# Synthesizer Architecture Reference

**Purpose**: This is a breadboard experimentation platform for discovering what makes a great pocket synthesizer. We're building and testing every feature we can think of—oscillators, filters, LFOs, effects, modulation—to find what's genuinely fun and musically inspiring.

**Philosophy**: Build everything, play with everything, keep only the amazing parts. This MVP lets us iterate quickly without committing to a form factor or final feature set. Every pot and control we add teaches us what's worth dedicating precious enclosure space to.

**Hardware**: ESP32-S3, PCM5102A DAC, ADS1115 ADC modules, pots, breadboards
**Current State**: Monophonic with 4-oscillator mixing, Moog ladder filter, professional audio quality (16-bit @ 44.1kHz)

**For future plans, see ROADMAP.md**

---

## Software Architecture Guidelines

### Control Mapping Architecture

**Use enums/IDs for control mapping, NOT hardcoded array indices:**

```rust
// ✅ CORRECT - Scales cleanly, self-documenting
enum Control {
    MasterPitch = 0,
    Osc1Level = 1,
    Cutoff = 6,
    Resonance = 7,
    // ... extensible ...
}

// ❌ WRONG - Hardcoded indices break when adding pots
let cutoff = adc_values[4]; // Which pot is this?
```

**Benefits:**
- Add new pots without changing existing code
- Self-documenting (clear parameter names)
- Type-safe (compiler catches typos)
- Easy to remap controls during breadboard experimentation

**Implementation Note**: Define all planned controls in the enum upfront. Unwired controls default to safe values (0.0 or mid-range).

---

### Processing Rate Architecture

**Three Processing Rates:**

- **Control Rate (50Hz)**: ADC pot polling - where parameter changes originate
- **Buffer Rate (~43Hz @ 4084-byte DMA buffer)**: DMA buffer callbacks - message consumption & coefficient caching
- **Audio Rate (44.1kHz)**: Per-sample processing - the super hot path

**Engine Rate Methods**:
```rust
// BUFFER RATE: Called once per buffer (~43Hz)
fn consume_messages(&mut self)         // Drain pot message queue, set targets
fn update_buffer_parameters(&mut self) // Recalc filter coefficients (if dirty)

// AUDIO RATE: Called per sample (44.1kHz)
fn render_sample(&mut self) -> f32     // Mix → normalize → filter
```

**Critical Optimization: Run Expensive Operations at Buffer Rate**

Buffer rate = 43.2 Hz (with 4084-byte DMA buffer) vs Audio rate = 44.1 kHz = **1021× frequency reduction**

**Examples:**
- ✅ Generative voice tick (noise → S&H → quantizer): **28 cycles @ buffer rate = 1,210 cycles/sec** (negligible)
- ❌ Same at audio rate: **2.47M cycles/sec** (will cause audio glitches)
- ✅ Filter coefficient calculation (tan, etc.): Buffer rate only when cutoff/resonance changes
- ❌ Filter coefficient calculation at audio rate: CPU starvation, crackling

**Rule**: If it doesn't need per-sample precision, run it at buffer rate.

---

### Smoothing Decision Rules

1. **Performance Parameters (Level, Pitch)** → **Use Slew (Audio Rate)**
   - **Why**: Users expect immediate tactile response (~10ms), buffer-rate is too sluggish
   - **Examples**: Oscillator pitch, oscillator level, mix normalization factor
   - **Pattern**: Call `.value()` in audio-rate code (e.g., inside `tick()` or `render_sample()`)
   - **Cost**: ~2-15 cycles per `.value()` call (settled optimization keeps it cheap 90% of time)
   - **Critical**: With α=0.99 at 44.1kHz, settling time is ~10ms - perfect for performance control

2. **Filter Coefficients** → **Use Slew (Buffer Rate ONLY)**
   - **Why**: Audio-rate smoothing causes CPU overload → crackling/pops even with α=0.95
   - **Examples**: Filter g_norm, resonance feedback (k)
   - **Pattern**:
     - Buffer rate: Call `.value()` in `update_coefficients()` to advance Slew state
     - Audio rate: Call `.current()` in `process()` for zero-cost reads
   - **Cost**: Zero at audio rate (just field access)
   - **Trade-off**: Sluggish control feel (~12ms lag with α=0.95 at 172Hz) but necessary for stability
   - **CRITICAL**: DO NOT use audio-rate smoothing for filter coefficients - it WILL cause crackling

3. **Modulation Sources** → **Generate Smooth (No Slew)**
   - **Why**: LFOs/Envelopes produce smooth signals natively
   - **Examples**: LFO output, ADSR envelope
   - **Pattern**: Generate at audio rate, apply to Slew targets at buffer rate

**Signal Flow:**
```
Pots (50Hz) → [9× multisampling + deadband] → Messages → Targets
                                                            ↓
Buffer Rate (43Hz): consume_messages() → Slew.set_target()
                                       → Filter: update_coefficients() + .value()
                                                            ↓
Audio Rate (44.1kHz): render_sample() → Performance params: .value() (pitch, level)
                                      → Filter coefficients: .current() (zero-cost)
```

---

## Critical Performance Patterns (ESP32-S3)

### Division Elimination

**Division is 10-40× more expensive than multiplication on ESP32-S3.**

```rust
// ❌ SLOW: Division in hot path
let phase_increment = frequency / sample_rate;  // 10-40 cycles per tick()

// ✅ FAST: Pre-calculated reciprocal
let sample_rate_reciprocal = 1.0 / sample_rate; // Once at initialization
let phase_increment = frequency * sample_rate_reciprocal;  // 1-2 cycles
```

**Apply to:**
- Oscillator phase increment calculation
- PolyBLEP (`phase * phase_inc_reciprocal` instead of `phase / phase_inc`)
- Normalization factors (cache reciprocal, multiply instead of divide)

### Lookup Tables vs Runtime Computation

**Rule**: If computation is expensive (>20 cycles) and domain is bounded, use lookup tables.

**Examples:**

| Operation | Runtime Cost | LUT Cost | Decision |
|-----------|--------------|----------|----------|
| **Semitone → frequency ratio** | 100-200 cycles (`powf(2.0, n/12.0)`) | 2-3 cycles (array[index]) | ✅ **Use LUT** (97 elements for ±4 octaves) |
| **Sine wave** | 20-40 cycles (`sinf()`) | ~4 cycles (interpolated table) | ✅ **Use wavetable** (1024 samples = 4KB) |
| **Sawtooth** | 2-3 cycles (`phase * 2.0 - 1.0`) | ~8 cycles (table + interp) | ❌ **Direct math faster** |
| **Random noise** | 50-200 cycles (`rand()` crate) | ~5 cycles (LFSR) | ✅ **Use LFSR** |

**Generative Synthesis Pattern (from ROADMAP.md):**
```rust
// Precomputed semitone ratios: 2^(n/12) for n ∈ [-48, +48]
const SEMITONE_RATIOS: [f32; 97] = [
    0.06250, 0.06622, ..., 16.00000  // Avoids expensive powf()
];

pub fn quantize(&self, input: f32) -> f32 {
    let semitones = input * self.octave_range * 12.0;
    let closest_semitone = find_nearest_scale_degree(semitones);  // Linear search
    let index = (closest_semitone + 48) as usize;  // Map [-48..+48] to [0..96]
    self.root_freq * SEMITONE_RATIOS[index]  // Fast array lookup
}
```

### Early Exit Optimizations

**Skip expensive operations when output is inaudible or unnecessary.**

```rust
// ✅ Skip PolyBLEP anti-aliasing below 1000 Hz (aliasing inaudible)
if self.frequency < POLYBLEP_THRESHOLD_HZ {
    return naive_waveform(self.phase);  // Fast path
}

// ✅ Skip oscillator tick when silent
if self.level.current() < OSC_SILENCE_THRESHOLD {
    return 0.0;  // Saves waveform generation + PolyBLEP
}
```

**Impact**: Prevents CPU starvation → eliminates crackling/pops during heavy loads.

### Hard Clamp Saturation (Filter Feedback)

**Use hard clamp, NOT division-based saturation.**

```rust
// ❌ EXPENSIVE: Rational function saturation
let fb_saturated = fb / (1.0 + fb * fb);  // 10-40 cycles (division!)

// ✅ FAST: Hard clamp
let fb_saturated = fb.clamp(-1.0, 1.0);  // 2-3 cycles
```

**Result**: Classic Moog tone, unconditionally stable, 10× faster.

---

## Waveform Generation Strategy

| Waveform | Method | Rationale |
|----------|--------|-----------|
| **Sine** | **Wavetable lookup** | `sinf()` is 20-40 CPU cycles; interpolated table lookup is ~4 cycles (10× faster) |
| **Sawtooth** | **Direct computation** | `(phase × 2.0) - 1.0` is faster than table + interpolation |
| **Square** | **Direct computation** | Simple comparison `if phase < 0.5` beats memory access |

**Memory footprint:**
- Sine wavetable: 1024 × 4 bytes = 4KB (justified by performance gain)
- Saw/square: Zero memory (computed on-the-fly)

**Anti-aliasing:**
- Runtime PolyBLEP for saw/square (can't pre-bake into static table)
- Early exit below 1000 Hz (aliasing inaudible)
- Professional soft synths (Serum, Vital) use same approach

---

## Control Message Architecture

**Compositional message design (reuses enums, type-safe routing):**

```rust
pub enum OscillatorId {
    Osc1 = 0, Osc2 = 1, Osc3 = 2, Sub = 3,
}

pub enum Message {
    SetMasterPitch(f32),                    // Absolute Hz
    SetOscLevel(OscillatorId, f32),         // Reuses enum for 4 oscillators
    SetOscDetune(OscillatorId, f32),        // Cents (converted internally)
    SetCutoff(f32),                         // Hz (exponential mapping)
    SetResonance(f32),                      // 0.0-4.0 (Moog ladder)
}
```

**Benefits:**
- ✅ **Compositional reuse**: 1 variant for 4 oscillators
- ✅ **Type-safe routing**: Compiler enforces valid IDs
- ✅ **Self-documenting**: Explicit intent

**Message handling pattern:**
```rust
fn consume_messages(&mut self) {
    while let Ok(msg) = self.consumer.try_receive() {
        match msg {
            Message::SetMasterPitch(freq) => self.set_master_pitch(freq),
            Message::SetOscLevel(osc_id, level) => {
                if let Some(osc) = self.oscillators.get_mut(osc_id as usize) {
                    osc.set_level(level);
                }
            }
            // ... other messages ...
        }
    }
}

// Complex logic extracted to helper methods
fn set_master_pitch(&mut self, freq: f32) {
    for osc in &mut self.oscillators[0..3] {
        osc.set_frequency(freq);  // Respects per-osc detune ratio
    }
    self.oscillators[3].set_frequency(freq / 2.0);  // Sub at -1 octave
}
```

---

## Engine Structure

**Monophonic architecture (implemented - no Voice abstraction):**

```rust
pub struct Engine {
    // Array of oscillators (enables clean iteration)
    oscillators: [Oscillator; 4],
    // Cached reciprocal for fast normalization (multiply > divide)
    level_sum_reciprocal: f32,
    // Filter with coefficient caching
    filter: MoogLadderFilter,
    // Message receiver
    consumer: ChannelReceiver,
}

pub struct Oscillator {
    phase: f32,                   // Normalized [0.0, 1.0) range
    phase_increment: Slew,        // Smoothed frequency changes
    frequency: f32,               // Base frequency in Hz
    sample_rate: f32,             // Sample rate in Hz
    waveform: Waveform,           // Sine/Saw/Square (fixed per instance)
    pub level: Slew,              // Output level (public for normalization)
    detune_ratio: f32,            // Frequency multiplier (1.0 = no detune)
}
```

**Design rationale:**
- ✅ **No Voice abstraction:** Unnecessary overhead for monophonic synthesis
- ✅ **Array of oscillators:** Clean iteration for uniform operations
- ✅ **Encapsulation:** Level & detune live in Oscillator
- ✅ **Cached normalization:** `level_sum_reciprocal` updated only on control changes (rare)
- ✅ **Fixed waveforms per oscillator:** Each has dedicated timbre
- ✅ **Hot-path optimization:** `tick()` applies level internally, render just sums outputs

---

## Audio Pipeline

**Format:**
- 16-bit stereo @ 44.1kHz (simplifies initial implementation)
- DMA buffer: 4084 bytes = 1021 samples = 23ms latency
- Future: 24-bit upgrade requires buffer type change `[i16]` → `[i32]`

**Dual-Core Architecture:**
- Core 0 (PRO_CPU): Audio rendering (real-time priority)
- Core 1 (APP_CPU): ADC polling (isolated from audio)
- Embassy MPSC channel for Core 1 → Core 0 messaging (thread-safe)
- Eliminates ADC blocking from affecting audio deadline
- Enables high multisampling (9 samples × pots) without audio impact

**Rendering pipeline:**
```rust
async fn render(&mut self, audio_stream: &mut AudioStream) {
    audio_stream.push_with(|buffer| {
        for chunk in buffer.chunks_exact_mut(4) {  // 4 bytes = stereo i16
            // Sum all oscillators (level applied internally by tick())
            let mix: f32 = self.oscillators.iter_mut()
                .map(|osc| osc.tick())
                .sum();

            // Normalize by cached reciprocal (fast multiply, not divide)
            let sample = (mix * self.level_sum_reciprocal * MASTER_GAIN * I16_MAX_F32) as i16;

            chunk[0] = sample;  // Left
            chunk[1] = sample;  // Right (mono)
        }
    });
}
```

---

## Extending the Architecture

### Adding Modulation Sources

**Pattern:**
1. **Tick at buffer rate** (43Hz) for CPU efficiency (unless per-sample precision needed)
2. **Generate smooth output** natively (no additional Slew needed)
3. **Apply to Slew targets** at buffer rate

**Example: LFO for filter cutoff modulation**
```rust
// Buffer rate: Generate LFO value
let lfo_value = self.filter_lfo.tick(); // Runs at 43Hz, not 44.1kHz

// Apply to filter parameter
let modulated_cutoff = self.base_cutoff + (lfo_value * self.lfo_depth);
self.filter.set_cutoff(modulated_cutoff);  // Marks dirty for coeff recalc
```

**Performance**: LFO tick at 43Hz vs 44.1kHz = 1021× CPU savings (from 2.47M cycles/sec to 2,419 cycles/sec).

### Adding Effects to Signal Chain

**Pattern:**
1. **Process at audio rate** in `render_sample()`
2. **Update coefficients at buffer rate** (if applicable)
3. **Use reciprocals** for any divisions in hot path

**Example: Delay (future)**
```rust
// Audio rate
fn render_sample(&mut self) -> f32 {
    let dry = self.filter.process(mixed_oscs);
    let wet = self.delay.process(dry);  // Audio rate
    dry * (1.0 - self.delay_mix) + wet * self.delay_mix
}

// Buffer rate
fn update_buffer_parameters(&mut self) {
    self.delay.update_parameters();  // Recalc delay time interpolation
}
```

### Scaling Message System

**Pattern:**
1. **Add enum variants** for new parameters
2. **Add message handlers** in `consume_messages()`
3. **Extract complex logic** to helper methods (keeps match arms clean)

**Example: Adding generative voice**
```rust
pub enum Message {
    // Existing...
    SetCutoff(f32),

    // New generative controls
    SetBassClockRate(f32),      // 0.5-4 Hz
    SetBassScale(f32),          // 0.0-1.0 → 12 scales
    SetBassOctaveRange(f32),    // 1-4 octaves
}

fn consume_messages(&mut self) {
    match msg {
        Message::SetBassClockRate(freq) => {
            self.bass_voice.set_clock_freq(freq);
        }
        // ... other handlers ...
    }
}
```

---

## Completed Implementation (Phases 1-3)

### Phase 1: Oscillator Architecture ✅

**Implemented:**
- Array of 4 oscillators (Sine, Saw, Square, Sub)
- Compositional message system (SetOscLevel, SetOscDetune)
- Encapsulated level & detune in Oscillator struct
- Cached reciprocal for fast normalization
- Phase normalization [0.0, 1.0) for clarity

**Current pot configuration (6 wired):**
- Pot 0 (GPIO1): Master Pitch (30-1024Hz exponential)
- Pot 1 (GPIO2): Osc1 Level (sine, 0.0-1.0)
- Pot 2 (GPIO3): Osc2 Detune (±2400¢)
- Pot 3 (GPIO4): Osc2 Level (saw, 0.0-1.0)
- Pot 4 (GPIO5): Filter Cutoff (165Hz-18kHz)
- Pot 5 (GPIO6): Filter Resonance (0.1-4.0)

### Phase 2: Moog Ladder Filter ✅

**Implemented:**
- Improved Huovilainen model (unconditionally stable)
- Dirty flag + coefficient caching
- Fast-settling Slew (α=0.95) for coefficients
- Hard clamp feedback saturation
- Exponential cutoff mapping (full audible spectrum)

### Phase 3: Performance Optimization ✅

**Implemented:**
- PolyBLEP anti-aliasing with early exit below 1000 Hz
- Division elimination (pre-calculated reciprocals)
- Oscillator silence optimization
- Slew settled flag (~80% reduction in EMA calls)
- Filter coefficient caching (~90% reduction)
- Dual-core architecture (ADC on Core 1, audio on Core 0)
- DMA buffer optimization (4084 bytes for CPU headroom)

**Result:** Zero audio glitches, responsive controls, professional anti-aliased audio.

---

## Reference Implementations

For future roadmap and generative synthesis features, see **ROADMAP.md**.

For code examples:
- Oscillator: `src/oscillator.rs`
- Filter: `src/filter.rs`
- Slew: `src/slew.rs`
- Engine: `src/engine.rs`
- Messages: `src/message.rs`
