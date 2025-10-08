# Drone Synthesizer MVP Build Plan

Refactoring roadmap: migrating from polyphonic voice-based architecture to monophonic drone synthesizer with professional audio quality.

**Hardware Status**: ✅ All components available (ESP32-S3, PCM5102A, ADS1115 modules, pots, breadboards)

---

## Software Architecture Guidelines (Critical for Scalability)

**Goal**: Ensure code written in Phase 1-4 scales to all 22 controls without major refactoring.

### Control Mapping Architecture

**Use enums/IDs for control mapping, NOT hardcoded array indices:**

```rust
// ✅ CORRECT - Scales cleanly from 10 to 22 controls
enum Control {
    MasterPitch = 0,
    Osc1Level = 1,
    Osc2Level = 2,
    Osc2Detune = 3,
    Osc3Level = 4,
    Osc3Detune = 5,
    Cutoff = 6,
    Resonance = 7,
    FilterLfoRate = 8,
    FilterLfoDepth = 9,
    PitchLfoRate = 10,
    PitchLfoDepth = 11,
    DelayTime = 12,
    DelayFeedback = 13,
    ReverbMix = 14,
    Drive = 15,
    SubLevel = 16,
    NoiseLevel = 17,
    Volume = 18,
}

// ❌ WRONG - Hardcoded indices require refactoring later
let cutoff = adc_values[4]; // Which pot is this? Will break when adding more pots
```

**Benefits:**
- Add new pots without changing existing code
- Self-documenting (clear parameter names)
- Type-safe (compiler catches typos)
- Easy to remap controls during breadboard experimentation

**Implementation Note**: Even if only wiring 10 pots in Phase 1, define all 22 controls in the enum. Unwired controls default to safe values (0.0 or mid-range).

---

## Software Architecture Decisions (Phase 1)

**Goal**: Document technical implementation choices based on DSP best practices and ESP32-S3 hardware constraints.

### Oscillator Design

**Waveform Generation Strategy:**

| Waveform | Method | Rationale |
|----------|--------|-----------|
| **Sine** | **Wavetable lookup** | `sinf()` is 20-40 CPU cycles; interpolated table lookup is ~4 cycles (10× faster) |
| **Sawtooth** | **Direct computation** | `(phase × 2.0) - 1.0` is faster than table + interpolation |
| **Square** | **Direct computation** | Simple comparison `if phase < 0.5` beats memory access |

**Implementation (Phase 1 complete):**
```rust
pub enum Waveform {
    Sine,
    Saw,
    Square,
}

pub struct Oscillator {
    phase: f32,                // Normalized [0.0, 1.0) range
    phase_increment: Slew,     // Smoothed frequency
    frequency: f32,            // Base frequency in Hz
    sample_rate: f32,          // Sample rate in Hz
    waveform: Waveform,        // Per-oscillator waveform selection
    pub level: Slew,           // Output level (integrated)
    detune_ratio: f32,         // Frequency multiplier
}

// In tick() method (applies level internally):
let sample = match self.waveform {
    Waveform::Sine => {
        // Wavetable lookup with linear interpolation
        let table_pos = self.phase * WAVETABLE_SIZE_F32;
        let index = table_pos as usize;
        let frac = table_pos - index as f32;
        let s1 = SINE[index];
        let s2 = SINE[(index + 1) & WAVETABLE_MASK];
        s1 + (s2 - s1) * frac  // FMA instruction on XTensa LX7
    }
    Waveform::Saw => (self.phase * 2.0) - 1.0,   // Direct math (faster than table)
    Waveform::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },  // Branch predictor wins
};

sample * self.level.value()  // Apply integrated level
```

**Memory footprint:**
- Sine wavetable: 1024 × 4 bytes = 4KB (justified by performance gain)
- Saw/square: Zero memory (computed on-the-fly)
- Total: 4KB vs. 12KB if all three used tables

**Anti-aliasing strategy:**
- **Phase 1-3:** Naive waveforms (will alias above ~2kHz, acceptable for testing)
- **Phase 4:** Add PolyBLEP to `tick()` method for saw/square (runtime computation, can't pre-bake into static table)
- **Precedent:** Professional soft synths (Serum, Vital) use runtime PolyBLEP, not pre-computed band-limited tables

### Control Message Architecture

**Compositional message design (implemented):**
```rust
pub enum OscillatorId {
    Osc1 = 0, Osc2 = 1, Osc3 = 2, Sub = 3,
}

pub enum Message {
    // Phase 1: Oscillator Controls
    SetMasterPitch(f32),                    // Absolute Hz
    SetOscLevel(OscillatorId, f32),         // Reuses enum for 4 oscillators
    SetOscDetune(OscillatorId, f32),        // Cents (converted internally)

    // Phase 2: Filter Controls
    SetCutoff(f32),                         // Hz (60-6668Hz exponential)
    SetResonance(f32),                      // 0.0-10.0 (Moog ladder resonance)
}
```

**Benefits:**
- ✅ **Compositional reuse**: `SetOscLevel(OscillatorId, f32)` = 1 variant for 4 oscillators
- ✅ **Type-safe routing**: Compiler enforces valid oscillator/filter IDs
- ✅ **Clean message handlers**: Pattern match on variant, route by enum value
- ✅ **Self-documenting**: Explicit intent (SetMasterPitch vs SetControl(0, val))

**Message handling pattern:**
```rust
fn consume_messages(&mut self) {
    while let Ok(msg) = self.consumer.try_receive() {
        match msg {
            Message::SetMasterPitch(freq) => self.set_master_pitch(freq),

            Message::SetOscLevel(osc_id, level) => {
                if let Some(osc) = self.oscillators.get_mut(osc_id as usize) {
                    let old_level = osc.level.target;
                    osc.set_level(level);
                    // Update cached normalization reciprocal (O(1))
                    self.update_level_sum_reciprocal(old_level, level);
                }
            }

            Message::SetOscDetune(osc_id, cents) => {
                if let Some(osc) = self.oscillators.get_mut(osc_id as usize) {
                    osc.set_detune_cents(cents);  // Converts cents→ratio internally
                }
            }

            // Phase 2 filter controls...
        }
    }
}

// Complex logic extracted to helper methods
fn set_master_pitch(&mut self, freq: f32) {
    // Update Osc1, Osc2, Osc3 with base frequency
    for osc in &mut self.oscillators[0..3] {
        osc.set_frequency(freq);  // Respects per-osc detune ratio
    }
    // Sub-oscillator at -1 octave
    self.oscillators[3].set_frequency(freq / 2.0);
}
```

### Engine Structure

**Monophonic architecture (implemented - replaces polyphonic Voice abstraction):**
```rust
pub struct Engine {
    // Array of 4 oscillators with fixed waveforms (enables clean iteration)
    oscillators: [Oscillator; 4],
    // [0] = Osc1 (Sine), [1] = Osc2 (Saw), [2] = Osc3 (Square), [3] = Sub (Sine @ -1 oct)

    // Cached reciprocal for fast normalization (multiply > divide in render loop)
    level_sum_reciprocal: f32,

    // Message receiver
    consumer: ChannelReceiver,
}

pub struct Oscillator {
    phase: f32,                   // Normalized [0.0, 1.0) range
    phase_increment: Slew,        // Smoothed frequency changes
    frequency: f32,               // Base frequency in Hz
    sample_rate: f32,             // Sample rate in Hz
    waveform: Waveform,           // Sine/Saw/Square (fixed per instance)
    pub level: Slew,              // Output level (public for Engine normalization)
    detune_ratio: f32,            // Frequency multiplier (1.0 = no detune)
}
```

**Design rationale:**
- ✅ **No Voice abstraction:** Unnecessary overhead for monophonic synthesis
- ✅ **Array of oscillators:** Clean iteration for uniform operations (vs 4 separate fields)
- ✅ **Encapsulation:** Level & detune live in Oscillator (component owns its data)
- ✅ **Cached normalization:** `level_sum_reciprocal` updated only on control changes (rare)
- ✅ **Fixed waveforms per oscillator:** Each oscillator has dedicated timbre (Sine/Saw/Square/Sub)
- ✅ **Master pitch tracking:** `set_master_pitch()` updates all 4 oscillators simultaneously
- ✅ **Hot-path optimization:** `tick()` applies level internally, render just sums outputs
- ✅ **Slew limiters:** Integrated into Oscillator for parameter smoothing (prevents clicks)

**Audio Format (Phase 1-3):**
- 16-bit stereo @ 44.1kHz (simplifies initial implementation)
- DMA buffer: 4084 bytes = 1021 samples = 23ms latency (increased during performance debugging)
- 24-bit upgrade deferred to Phase 4 (requires buffer type change `[i16]` → `[i32]`)

**Dual-Core Architecture (Implemented):**
- Core 0 (PRO_CPU): Audio rendering (real-time priority)
- Core 1 (APP_CPU): ADC polling (isolated from audio)
- Embassy MPSC channel for Core 1 → Core 0 messaging (thread-safe)
- Eliminates ADC blocking from affecting audio deadline
- Enables high multisampling (9 samples × 5 pots) without audio impact

**Rendering pipeline (Phase 1-3, implemented):**
```rust
async fn render(&mut self, audio_stream: &mut AudioStream) {
    audio_stream.push_with(|buffer| {
        for chunk in buffer.chunks_exact_mut(4) {  // 4 bytes = stereo i16
            // Sum all oscillators (level applied internally by tick())
            let mix: f32 = self.oscillators.iter_mut()
                .map(|osc| osc.tick())  // tick() returns waveform * level
                .sum();

            // Normalize by cached reciprocal (fast multiply, not divide)
            // Apply master gain and convert to i16
            let sample = (mix * self.level_sum_reciprocal * MASTER_GAIN * I16_MAX_F32) as i16;

        // Phase 4+: Add filter/drive/effects + upgrade to 24-bit
        // let filtered = self.filter.process(mixed);
        // let driven = self.drive.process(filtered);
        // let output = (driven * self.master_volume.value() * I24_MAX_F32) as i32;

        sample[0] = output;  // Left
        sample[1] = output;  // Right (mono)
    }
}
```

---

## Phase 1: Architecture Refactor - Polyphonic → Monophonic

**Status**: ✅ **COMPLETE** (with Phase 2 filter + Phase 3 optimizations integrated)
**Goal**: Migrate from Voice-based polyphonic to direct Oscillator-based monophonic drone architecture

**Critical Performance Optimizations Implemented:**
- ✅ **Slew settled optimization**: Added `settled` flag to skip EMA computation when parameter stable (~80% reduction)
- ✅ **Filter coefficient caching**: Only recalculate `f` and `q` when cutoff/resonance changes (~90% reduction)
- ✅ **Direct loop iteration**: Replaced `.iter_mut().map().sum()` with explicit loop (eliminated closure overhead)
- ✅ **Dual-core architecture**: ADC on Core 1, audio on Core 0 (eliminates blocking contention)
- ✅ **DMA buffer optimization**: Increased to 4084 bytes for CPU deadline headroom

### Baseline (commit d9c3b67 "pre-refactor"):
- ✅ I2S audio output @ 44.1kHz (PCM5102A DAC)
- ✅ Parameter smoothing via Slew limiter
- ✅ ADC pot reading with EMA filtering
- ✅ Embassy async runtime with message passing
- ✅ Polyphonic engine with 3 Voice instances
- ✅ Build verified: compiles in 1.61s

### Implemented Changes:

**Architecture Refactor:**
- ✅ Removed `Voice` struct abstraction (src/voice.rs deleted)
- ✅ Refactored `Engine` to hold **array of 4 Oscillators** (cleaner iteration vs individual fields)
- ✅ **Encapsulated level into Oscillator**: `pub level: Slew` field (self-contained tick())
- ✅ **Encapsulated detune into Oscillator**: `detune_ratio: f32` field (internal storage)
- ✅ Implemented `set_master_pitch()` helper (updates all 4 oscillators simultaneously)
- ✅ **Performance optimization**: Cached `level_sum_reciprocal` in Engine (multiply > divide in hot path)

**Oscillator Improvements:**
- ✅ Added `Waveform` enum (Sine/Saw/Square)
- ✅ Added `waveform` field to Oscillator struct
- ✅ Added `pub level: Slew` and `detune_ratio: f32` fields (encapsulation)
- ✅ Implemented `set_detune_cents(cents)` with micromath `powf()` (cents → ratio conversion)
- ✅ **Phase normalization**: Migrated from [0, 1024) to [0.0, 1.0) for clarity
- ✅ Removed wavetable field (static SINE constant reference is faster)
- ✅ Implemented waveform switching in `tick()` with integrated level application
- ✅ Direct sawtooth: `(phase * 2.0) - 1.0`
- ✅ Direct square: `if phase < 0.5 { 1.0 } else { -1.0 }`
- ✅ Wavetable sine with linear interpolation (performance critical)

**Message System (Compositional Design):**
- ✅ **Compositional messages** (cleaner than unified SetControl):
  - `Message::SetMasterPitch(f32)` - absolute Hz
  - `Message::SetOscLevel(OscillatorId, f32)` - reuses enum, not separate variants
  - `Message::SetOscDetune(OscillatorId, f32)` - cents (converted internally)
- ✅ `OscillatorId` enum (Osc1/Osc2/Osc3/Sub) for type-safe routing
- ✅ Phase 2 filter message stubs (SetCutoff/SetResonance)

**Control Mapping:**
- ✅ `map_master_pitch()`: normalized → 30-1024Hz exponential
- ✅ `map_osc_level()`: normalized → 0.0-1.0 direct
- ✅ `map_osc_detune()`: normalized → -50¢ to +50¢ (centered at 0.5)
- ✅ **Current test setup**: 2 pots (GPIO1=pitch, GPIO2=osc1_level)

**Key Design Decisions Made:**
- **Array vs separate fields**: Array enables clean iteration, uniform operations
- **Encapsulation**: Level/detune live in Oscillator (component owns its data)
- **Reciprocal caching**: O(1) update on control change, fast multiply in render
- **Public fields**: `level: Slew` public for Engine normalization reads
- **Phase space**: [0.0, 1.0) normalized (clearer than wavetable index space)
- **Detune storage**: Internal ratio (fast), public API in cents (intuitive)

**Phase 1 Full Pot Mapping (Ready to Wire):**
- Pot 0: `MasterPitch` (30Hz - 1024Hz exponential) ✅
- Pot 1: `Osc1Level` (sine, 0.0 - 1.0) ✅
- Pot 2: `Osc2Detune` (±2400¢ = 4 octaves) ✅
- Pot 3: `Osc2Level` (saw, 0.0 - 1.0) ✅
- Pot 4: `Osc3Level` (square, 0.0 - 1.0) 🔄
- Pot 5: `Osc3Detune` (±2400¢) 🔄
- Pot 6: `SubLevel` (sine -1 octave, 0.0 - 1.0) 🔄

**Current Build Status**: ✅ Clean compilation, 6 pots wired and functional

**Current Pot Configuration (src/controls/task.rs):**
- Pot 0 (GPIO1): Master Pitch ✅
- Pot 1 (GPIO2): Osc1 Level ✅
- Pot 2 (GPIO3): Osc2 Detune ✅
- Pot 3 (GPIO4): Osc2 Level ✅
- Pot 4 (GPIO5): Filter Cutoff ✅ (Phase 2)
- Pot 5 (GPIO6): Filter Resonance ✅ (Phase 2)

---

## Phase 2: Filter & Effects Implementation

**Status**: ✅ **FILTER COMPLETE** (Effects deferred to focus on modulation)
**Goal**: Complete signal chain with Moog Ladder Filter and saturation (add 4 pots: Cutoff, Resonance, Drive, Volume)

**Filter Implementation:**
- ✅ Implement Moog Ladder Filter (4-pole lowpass with resonance feedback)
- ✅ **Improved Huovilainen model**: Unconditional stability at all cutoff/resonance combinations
- ✅ **Performance optimization**: Dirty flag + coefficient caching (only recalc when parameters change)
- ✅ **Coefficient smoothing**: Fast-settling Slew (α=0.95) eliminates zipper noise without CPU overhead
- ✅ Add Cutoff pot control with exponential mapping (165Hz - 18kHz, full audible spectrum)
- ✅ Add Resonance pot control (0.1 - 4.0, sweet spot for self-oscillation without harshness)
- ✅ Hard clamp feedback saturation (fast, stable, classic Moog tone)

**Effects & Finalization (Deferred):**
- ⏸️ Implement Drive/saturation (soft clipping waveshaper) - *deferred to prioritize LFO modulation*
- ⏸️ Calibrate Drive curve for warmth without harshness
- ⏸️ Add master Volume control with logarithmic pot scaling
- ⏸️ Tune all parameter ranges for musicality
- ⏸️ Document optimal control positions for various drone textures

**Current Pot Mapping (6 pots wired):**
- Pot 4 (GPIO5): `Cutoff` (165Hz - 18kHz exponential) ✅
- Pot 5 (GPIO6): `Resonance` (0.1 - 4.0) ✅

**Checkpoint**: OSC → Filter(LP) → Output chain working beautifully, LFO modulation now highest priority

---

## Phase 3: Performance Optimization & Refinement

**Status**: ✅ **COMPLETE** (All critical optimizations implemented)
**Goal**: Stable 44.1kHz audio with minimal CPU overhead, responsive controls, professional DSP quality

**DSP Quality Improvements:**
- ✅ Add PolyBLEP anti-aliasing to sawtooth/square oscillators (commit: "optimized polyBLEP")
- ✅ Early exit optimization: Skip PolyBLEP below 1000 Hz (POLYBLEP_THRESHOLD_HZ) where aliasing inaudible
- ✅ Division elimination: Pre-calculated reciprocals (phase_inc_reciprocal) for hot-path performance
- ✅ Oscillator silence optimization: Skip waveform generation when level < OSC_SILENCE_THRESHOLD
- ✅ Test filter response at extreme cutoff/resonance settings - **Unconditionally stable**

**Performance Profiling:**
- ✅ Profile audio callback CPU usage - **Identified Slew and filter as bottlenecks**
- ✅ Add `#[inline]` attributes to hot-path functions (Slew, filter, oscillator tick)
- ✅ Optimize Moog Ladder Filter process() - **Coefficient caching + fast-settling Slew (α=0.95)**
- ✅ **Slew optimization**: Settled flag skips computation when stable (~80% reduction in EMA calls)
- ✅ Verify DMA buffer size - **4084 bytes (23ms @ 44.1kHz) provides CPU deadline headroom**

**System Integration:**
- ✅ Verify dual-core task distribution - **Core 0 = audio (PRO_CPU), Core 1 = ADC (APP_CPU)**
- ✅ Embassy MPSC channel for thread-safe messaging between cores
- ✅ Test ADC scan rate with all pots - **6 pots @ 9× multisampling, no audio impact**
- ✅ Soft-start gain prevents speaker pop on initialization
- ✅ Crackling and pops eliminated via division elimination and early exit optimizations

**Checkpoint**: ✅ Zero audio glitches during continuous operation, responsive controls, professional anti-aliased audio

---

## Phase 3.5: Filter Cutoff LFO (Highest Priority Modulation)

**Status**: 🚀 **READY TO START**
**Goal**: Add filter cutoff modulation for evolving drone textures (adds 2 pots: LFO Rate, LFO Depth)

**Rationale - Why Filter Cutoff LFO is Most Impactful:**
1. **Core of drone synthesis**: Timbral evolution over time creates psychoacoustic interest
2. **Classic modular technique**: Filter sweeps are the essence of vintage drone/ambient synthesis
3. **Musical range**: 0.01-10Hz covers slow pads (30s cycle) to rhythmic pulses (0.1s cycle)
4. **Works with existing architecture**: Modulates cutoff parameter at buffer rate (no new signal chain elements)
5. **Immediate sonic impact**: More dramatic than pitch LFO for sustained drones

**Implementation Architecture:**

```rust
// src/lfo.rs - New file
pub struct Lfo {
    /// Current phase [0.0, 1.0)
    phase: f32,
    /// Phase increment per sample (frequency / sample_rate)
    phase_increment: f32,
    /// LFO frequency in Hz (smoothed via Slew)
    frequency: Slew,
    /// LFO depth 0.0-1.0 (smoothed via Slew)
    depth: Slew,
    /// Sample rate
    sample_rate: f32,
}

impl Lfo {
    /// Generate next LFO sample (AUDIO RATE - called per sample)
    /// Returns: bipolar output [-1.0, 1.0] scaled by depth
    #[inline(always)]
    pub fn tick(&mut self) -> f32 {
        // Use existing SINE wavetable (share with oscillator, no extra memory)
        let sine = wavetable::sine_at_phase(self.phase);

        // Advance phase (wraps automatically via modulo in 0.0-1.0 space)
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Scale by smoothed depth (prevents zipper noise on depth changes)
        sine * self.depth.value()
    }

    /// Update LFO parameters (BUFFER RATE - called once per buffer)
    #[inline]
    pub fn update_parameters(&mut self) {
        // Recalculate phase increment only when frequency changed (dirty flag pattern)
        let freq = self.frequency.value();
        self.phase_increment = freq / self.sample_rate;
    }
}
```

**Engine Integration (src/engine.rs):**

```rust
pub struct Engine {
    // ... existing fields ...

    /// Filter cutoff LFO (generates modulation signal)
    filter_lfo: Lfo,

    /// Base filter cutoff from pot (before modulation)
    base_cutoff: f32,

    /// LFO modulation depth in Hz (mapped from pot, exponential scale)
    lfo_cutoff_range: f32,
}

// BUFFER RATE: Update LFO and calculate modulated cutoff
fn update_buffer_parameters(&mut self) {
    // Update LFO parameters (frequency Slew)
    self.filter_lfo.update_parameters();

    // Generate LFO value for this buffer (audio rate will use cached value)
    // Future: Move to audio rate if per-sample modulation sounds better
    let lfo_value = self.filter_lfo.tick(); // -1.0 to +1.0

    // Apply LFO to base cutoff: cutoff = base + (lfo × range)
    // Range example: pot at 1000Hz, depth at max (±2 octaves) = 250Hz to 4000Hz
    let modulated_cutoff = self.base_cutoff + (lfo_value * self.lfo_cutoff_range);

    // Clamp to valid filter range, set filter cutoff (marks dirty for coefficient recalc)
    self.filter.set_cutoff(modulated_cutoff.clamp(FILTER_CUTOFF_MIN, FILTER_CUTOFF_MAX));

    // ... existing oscillator/filter parameter updates ...
}
```

**Message System Updates:**

```rust
// src/message.rs
pub enum Message {
    // ... existing messages ...

    /// Set Filter Cutoff LFO rate in Hz (0.01-10.0 Hz exponential)
    SetFilterLfoRate(f32),

    /// Set Filter Cutoff LFO depth (0.0-1.0, maps to ±N octaves)
    SetFilterLfoDepth(f32),
}
```

**Control Mapping Functions:**

```rust
// src/controls/pot.rs

/// Map pot to LFO rate (exponential: 0.01 Hz to 10 Hz)
/// 0.01 Hz = 100 second cycle (ultra-slow pad evolution)
/// 10 Hz = 0.1 second cycle (rhythmic tremolo effect)
pub fn map_lfo_rate(normalized: f32) -> f32 {
    // Exponential curve: 10^(normalized × 3 - 2) = 0.01 to 10.0
    // normalized 0.0 → 10^-2 = 0.01 Hz
    // normalized 0.5 → 10^-0.5 ≈ 0.316 Hz (sweet spot for drones)
    // normalized 1.0 → 10^1 = 10 Hz
    let exponent = normalized * 3.0 - 2.0;
    micromath::F32Ext::powf(10.0, exponent).clamp(0.01, 10.0)
}

/// Map pot to LFO depth (0.0 to 1.0, internally maps to ±octave range)
/// Engine will multiply by octave range (e.g., ±2 octaves = 4× cutoff swing)
pub fn map_lfo_depth(normalized: f32) -> f32 {
    normalized  // Linear 0.0-1.0
}
```

**Pot Wiring Plan (Adds 2 Pots):**
- Pot 6 (GPIO7): `FilterLfoRate` (0.01 - 10 Hz exponential) 🔧
- Pot 7 (GPIO8): `FilterLfoDepth` (0.0 - 1.0, ±2 octaves) 🔧

**Implementation Steps:**
1. ✅ Design LFO architecture (above)
2. [ ] Create `src/lfo.rs` with Lfo struct
3. [ ] Add LFO to Engine struct with base_cutoff tracking
4. [ ] Implement buffer-rate modulation in `update_buffer_parameters()`
5. [ ] Add Message::SetFilterLfoRate/Depth variants
6. [ ] Add pot mapping functions (map_lfo_rate, map_lfo_depth)
7. [ ] Wire pots 6-7 in controls/task.rs
8. [ ] Test LFO range: verify smooth sweeps at 0.1 Hz, 1 Hz, 10 Hz
9. [ ] Tune depth scaling for musical range (start with ±2 octaves)
10. [ ] Document sweet spots (e.g., 0.5 Hz rate + 50% depth = classic drone texture)

**Performance Considerations:**
- ✅ Reuses existing SINE wavetable (no extra memory)
- ✅ LFO runs at audio rate, modulation applied at buffer rate (efficient)
- ✅ Slew on frequency/depth prevents zipper noise
- ✅ No expensive math in tick() - just wavetable lookup + multiply
- ✅ Leverages existing filter coefficient caching (dirty flag optimization)

**Testing Plan:**
1. Static LFO test: Set rate=1Hz, depth=100%, verify 1-second cycle
2. Range test: Pot sweep from 0.01 Hz (slow) to 10 Hz (fast), verify exponential response
3. Depth test: Fixed rate, sweep depth 0→100%, verify cutoff modulation range
4. Musical test: Find sweet spots for pad textures, rhythmic pulses, subtle movement
5. CPU test: Verify no audio glitches with LFO active (should be negligible overhead)

**Success Criteria:**
- ✅ LFO generates clean sine wave at all rates (0.01-10 Hz)
- ✅ Filter cutoff sweeps smoothly without clicks or zipper noise
- ✅ Depth control provides intuitive scaling (0% = static, 100% = dramatic sweep)
- ✅ Can create classic drone textures: slow pad evolution, rhythmic filter pulses
- ✅ No CPU overhead or audio glitches with LFO running
- ✅ 8 pots total wired and functional

---

## Phase 4: Extended Features & Display - OPTIONAL

**Status**: Not Started (Phase 3.5 Filter LFO takes priority)
**Goal**: Add pitch modulation, time-based effects, and visual feedback (requires scaling to 22 controls)

**Modulation (Lower Priority than Filter LFO):**
- [ ] Implement Pitch LFO (sine wave, 0.01-20Hz rate + depth controls) - *vibrato effects*
- [ ] Add noise generator with level control - *organic texture layer*

**Time-Based Effects:**
- [ ] Implement Delay using PSRAM buffer (time: 50ms-5sec, feedback: 0-110%)
- [ ] Implement Reverb (4-tap FDN algorithm with mix control)

**Display Integration:**
- [ ] Wire SSD1351 OLED to SPI bus (GPIO pins TBD based on I2S/ADC assignments)
- [ ] Integrate embedded-graphics crate for display rendering
- [ ] Implement oscilloscope waveform visualization (128 pixels wide)
- [ ] Implement parameter overlay display (shows control name + value)
- [ ] Add 2-second fade timer for overlay (returns to pure waveform view)
- [ ] Optimize SPI refresh rate for 30+ FPS without audio glitches

**Control Expansion:**
- [ ] Wire additional ADS1115 modules for remaining pots (controls 8-18)
- [ ] Map remaining pots: Filter LFO Rate/Depth, Pitch LFO Rate/Depth, Delay Time/Feedback, Reverb Mix, Drive, Sub Level, Noise Level, Volume

**Decision Point**: Only pursue if Phase 1-3 delivers satisfying core drone synthesis

---

## Phase 5: Permanent Build Transition - FUTURE

**Status**: Not Started
**Goal**: Portable enclosure with battery power, professional build quality

**Prerequisites**: Phase 1-4 fully validated on breadboard

**Power System Design:**
- [ ] Design 18650 Li-Ion battery system with USB-C charging
- [ ] Implement buck-boost regulation for stable 3.3V (battery: 2.5-4.2V)
- [ ] Add low-dropout LDO for clean analog power
- [ ] Include power switch and battery protection

**Mechanical Design:**
- [ ] Design/source enclosure (~178×257×72mm, pocket-portable)
- [ ] Plan control panel layout for 22 pots + display
- [ ] Consider 3D print vs. off-shelf enclosure options

**Perfboard Construction:**
- [ ] Design modular board layout (power, controller, ADC bank, audio)
- [ ] Build and validate power board standalone
- [ ] Build ESP32 board with I2C routing for 6× ADS1115
- [ ] Build audio output board (DAC + optional headphone amp)
- [ ] Integrate all subsystems with proper wiring

**Final Assembly:**
- [ ] Mount components in enclosure
- [ ] Wire controls to ADC modules
- [ ] Test battery runtime (target: 20+ hours)
- [ ] Final sound validation and calibration

---

## Success Criteria

### Phase 1 (Refactor - Oscillators Only) Complete When:
- ✅ Voice abstraction removed, direct oscillator architecture implemented
- ✅ `WaveformType` enum added, each oscillator has fixed waveform (sine/saw/square)
- ✅ 4 oscillators (osc1/osc2/osc3/sub) blend creates rich timbres
- ✅ Unified `Control` enum message system handles all 22 parameters (defined, only 7 wired)
- ✅ Master pitch control updates all 4 oscillators simultaneously
- ✅ Detuning (osc2/osc3) creates chorus/beating effects
- ✅ All 7 pot controls respond smoothly with parameter smoothing
- ✅ Clean audio output with no clicks or zipper noise
- ✅ 16-bit audio @ 44.1kHz with 5.8ms latency (1024-byte DMA buffer)

### Phase 2 (Filter/Effects - Add 4 Pots) Complete When:
- ✅ Moog Ladder Filter shapes tone musically with classic analog character
- ✅ Cutoff and Resonance controls respond smoothly
- ✅ Self-oscillation at high resonance creates additional tonal voice
- ✅ Drive/saturation adds warmth without harshness
- ✅ Master volume control provides proper output scaling
- ✅ Full signal chain OSC→Moog Filter→Drive→Output sounds professional
- ✅ All 11 pots wired and functional
- ✅ Can create evolving drone textures continuously
- ✅ 16-bit audio @ 44.1kHz with 5.8ms latency (1024-byte buffer)

### Phase 3 (Performance Optimization) Complete When:
- ✅ PolyBLEP anti-aliasing eliminates harsh aliasing artifacts
- ✅ Filter stable at all cutoff/resonance combinations (no crashes or NaN)
- ✅ Audio callback uses <95% CPU utilization
- ✅ Zero audio glitches during continuous 10+ minute operation
- ✅ ADC scan completes in <25ms for all wired pots
- ✅ All controls respond within 30ms latency
- ✅ DMA buffer shows no underruns in profiling (1024 bytes = 5.8ms)
- ✅ Dual-core task distribution verified (audio priority on Core 1)

### Phase 4 (Extended Features & Display - Optional) Complete When:
- ✅ Filter Cutoff LFO and Pitch LFO add musical movement
- ✅ Delay and Reverb create spatial depth
- ✅ Noise generator adds organic texture
- ✅ All ~19 controls wired and functional
- ✅ Real-time oscilloscope waveform renders at 30+ FPS
- ✅ Display updates don't interfere with audio rendering

### Phase 5 (Permanent Build) Complete When:
- ✅ All 22 controls functional in enclosure
- ✅ Battery provides 20+ hours runtime
- ✅ Pocket-portable, professional build quality

---

## Risk Mitigation

**Risk**: Architecture refactor breaks working audio
**Mitigation**: Work on feature branch `monophony`; keep stable polyphonic version on `main`

**Risk**: Monophonic design feels limiting after polyphonic experience
**Mitigation**: Focus on rich timbral control (4 oscillators, filter modes, effects); drones don't need polyphony

**Risk**: DMA buffer underruns during refactor
**Mitigation**: Current 2044-byte buffer is proven stable; maintain same buffer size

**Risk**: Oscillator phase accumulation drift over long sessions
**Mitigation**: Use f32 phase with proper wrapping; test 10+ minute continuous operation

**Risk**: Filter becomes unstable at high resonance
**Mitigation**: Clamp internal state variables; use reference SVF implementations (Hal Chamberlin, Andy Simper)

---

## Current Status: Phase 1 Ready to Start

**Pre-refactor State**: Commit d9c3b67 "pre-refactor" - working polyphonic synth with frequency/volume pots

**Audio Format Decisions**:
- **Phase 1-3:** 16-bit stereo @ 44.1kHz (simplifies implementation)
- **DMA Buffer:** 1024 bytes = 256 samples = 5.8ms latency (reduced from 2044 bytes/11.6ms)
- **Phase 4:** Upgrade to 24-bit if needed for effects/final polish

**Phase Scope Refinements**:
- **Phase 2:** Moog Ladder lowpass filter (classic analog sound)
- **Phase 3:** Add PolyBLEP anti-aliasing to oscillators
- **Phase 4:** Optional extended features (LFOs, effects, display)

**Next Action**: Begin architecture refactor on `monophony` branch

**Active Focus**:
1. Add `WaveformType` enum to Oscillator (Sine/Saw/Square)
2. Implement waveform switching in `tick()` method
3. Define `Control` enum for all 22 parameters
4. Refactor Engine to hold 4 direct Oscillator instances with fixed waveforms
5. Remove Voice abstraction (src/voice.rs)
6. Update message handling to unified `SetControl(Control, f32)` system
7. Wire and map **7 initial pots**: MasterPitch, Osc1/2/3 Levels, Osc2/3 Detune, SubLevel

**Branch Strategy**: Work on `monophony` branch, merge to `main` when Phase 1 complete
