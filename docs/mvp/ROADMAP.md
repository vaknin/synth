# Generative Synthesis Roadmap

**Vision**: Raw, analog pocket synth with generative bass + lead + delay. Simple, repetitive patterns that YOU perform on (filter sweeps, modulation). No keys, no screens - just pots.

**Current State**: Phases 1-3 complete (see ARCHITECTURE.md). 6 pots wired, 4 oscillators, Moog filter, filter LFO working.

**Hardware Available**: 12 pots total (6 wired + 6 in inventory)
**Hardware Arriving**: 10+ more pots from AliExpress (TBD)

**Strategy**: Build FULL system on breadboard using phased pot allocation. Test extensively. Distill to pocket essentials based on real use.

---

## What Is Generative Synthesis?

**Generative = RHYTHMIC REPETITION with MELODIC VARIATION**

**Traditional synth**: You control pitch directly (pot sets ONE static note)

**Generative synth**: Algorithm creates patterns with THREE layers of variation, YOU perform timbre:
- **Pitch**: S&H + quantizer = always in key, melodic but unpredictable
- **Rhythm**: Gate probability = syncopation, rests, breathing room (not metronomic)
- **Articulation**: Random envelope times = varied note lengths (plucks vs sustains)
- **YOU control**: Filter sweeps, resonance, modulation depth, pattern density

**Result**: Expressive, evolving bass lines or lead melodies that never exactly repeat. YOU shape the sound in real-time.

### Example: Bass Generative at 4 Hz (70% gate probability)

```
Clock:  ↓    ↓    ↓    ↓    ↓    ↓    ↓    ↓    ↓    ↓
Gate:   YES  NO   YES  YES  NO   YES  NO   NO   YES  YES
Note:   C    C    E♭   G    G    C    C    C    F    E♭
Length: 35ms hold 70ms 20ms hold 100ms hold hold 80ms 45ms
        ▬╲__ ▬▬▬▬ ▬╲__ ▬╲   ▬▬▬▬ ▬╲___ ▬▬▬▬ ▬▬▬▬ ▬╲__ ▬╲_

Pitch: Always in key (quantizer → C minor scale)
Rhythm: Syncopated (gate probability = 70% density, not metronomic)
Articulation: Varied (envelope times 10-150ms, random per note)
```

**What you hear:**
- ✅ Evolving rhythm (gate probability creates syncopation + rests)
- ✅ Simple bass line (just notes, no fancy patterns)
- ✅ Varied articulation (short plucks + longer sustains)
- ✅ Raw and analog (no sequencer UI)
- ✅ Never exact repeat (pitch, rhythm, and articulation all vary)

**YOU perform:**
- Filter sweeps (open/close timbre)
- Resonance peaks (screaming Moog character)
- LFO depth changes (add movement)
- Clock rate changes (speed up/slow down live)

---

## Signal Chain

```
┌──────────────┐
│ White Noise  │ Random signal (LFSR)
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│ Sample & Hold    │ Clock grabs random value at rhythm
│ Clock: 0.5-20 Hz │ ← Pot: Note Change Rate
└──────┬───────────┘
       │
       ▼
┌──────────────────────┐
│ Gate Probability     │ Random triggering (syncopation/rests)
│ 30-100% density      │ ← Pot: Gate Probability (optional)
└──────┬───────────────┘
       │
       ▼
┌──────────────────┐
│ Scale Quantizer  │ Snap to musical scale (always in key)
│ 12 scales        │ ← Pot: Scale Selection
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Pitch Range      │ Map to frequency range
│ Bass: 30-300 Hz  │ ← Pot: Root Note + Octave Range
│ Lead: 200-1024Hz │
└──────┬───────────┘
       │
       ▼
┌──────────────────────┐
│ Glide/Portamento     │ Smooth pitch transitions
│ Bass: 150ms (fixed)  │ (Hardcoded initially, pot if needed)
│ Lead: 50ms (fixed)   │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Envelope Generator   │ Note articulation (plucks vs sustains)
│ AD: 5ms attack       │ ← Pot: Envelope Max Time (optional)
│ 10-500ms decay       │     Random decay per note
└──────┬───────────────┘
       │
       ▼
┌──────────────────┐
│ Voice Assignment │
│ Bass: Main OSC   │ 1 saw + 1 sub (hardcoded)
│ Lead: Main OSC   │ OR simultaneous bass+lead (future)
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Moog Filter      │ YOU control cutoff/resonance
│ + LFO Modulation │ ← Pots: Cutoff, Resonance, LFO Depth
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Delay (optional) │ Echoes create pseudo-polyphony
│ PSRAM buffer     │ ← Pots: Time, Feedback
└──────┬───────────┘
       │
       ▼
    Output
```

---

## Final System Architecture (When All Pots Arrive)

**Estimated Total: 17-19 pots**

### PERFORMANCE (5 pots - Always Active)

1. **Filter Cutoff** (165 Hz - 15 kHz) - Timbre shaping
2. **Filter Resonance** (0.1 - 2.0) - Moog character
3. **Filter LFO Rate** (0.01-10 Hz) - Movement speed (testing if pot needed)
4. **Filter LFO Depth** (0-100%) - Movement amount
5. **Master Volume** (0.0-1.0) - Output level

### BASS VOICE (6 pots)

6. **Bass Clock Rate** (0.5-8 Hz) - Rhythm speed (slow bass)
7. **Bass Root Note** (30-300 Hz) - Tonal center (bass register)
8. **Bass Scale** (12 scales) - Minor, pentatonic, blues, etc.
9. **Bass Octave Range** (0-4 octaves) - Jump size (smooth vs dramatic)
10. **Bass Gate Probability** (30-100%) - Rhythm density (sparse vs dense)
11. **Bass Envelope Time** (10-500ms) - Note length variation

### LEAD VOICE (6 pots)

12. **Lead Clock Rate** (4-20 Hz) - Fast rhythmic melody
13. **Lead Root Note** (200-1024 Hz) - Melodic register
14. **Lead Scale** (12 scales) - Can differ from bass (polymodal)
15. **Lead Octave Range** (0-4 octaves) - Melodic jump size
16. **Lead Gate Probability** (30-100%) - Rhythm density
17. **Lead Envelope Time** (10-500ms) - Note length variation

### DELAY - Option A: Full Control (3 pots)

18. **Delay Time** (50ms - 2 seconds) - Echo spacing
19. **Delay Feedback** (0-110%) - Echo intensity
20. **Delay Mix** (0-100%) - Dry/Wet balance

### DELAY - Option B: Essential Control (2 pots)

18. **Delay Time** (50ms - 2 seconds) - Echo spacing (rhythmic sync critical)
19. **Delay Mix** (0-100%) - Dry/Wet balance (performance blend)
- Feedback hardcoded: 40% (can adjust in software)

### PITCH MODULATION (2 pots - Optional, Phase 4E Testing)

20/21. **Pitch LFO Rate** (0.01-10 Hz) - Vibrato speed
21/22. **Pitch LFO Depth** (0-50¢) - Vibrato amount

### Hardcoded (No Pots)

- **Main OSC waveform**: Saw (richest harmonics)
- **Main OSC level**: 0.7 (70% mix, constant)
- **Sub OSC waveform**: Square (classic bass)
- **Sub OSC level**: 0.3 (30% mix, bass thickness)
- **Sub OSC pitch**: -1 octave (automatic)
- **Filter LFO waveform**: Sine (smooth sweeps)
- **Bass glide**: 150ms (test if pot needed)
- **Lead glide**: 50ms (test if pot needed)
- **Envelope attack**: 5ms (fast, percussive-capable)

---

## Phased Testing Strategy (12-Pot Limit → Expansion)

### Phase 4A: Bass Generative Only (12 pots)

**Goal**: Test if generative bass with THREE layers of variation (pitch, rhythm, articulation) creates expressive, non-boring patterns.

**Pot Allocation:**

**Core Performance (3 pots - NEVER reprogram):**
```rust
Pot 0 (GPIO1): Filter Cutoff     ← Essential for performance
Pot 1 (GPIO2): Filter Resonance  ← Essential for performance
Pot 2 (GPIO3): Filter LFO Depth  ← Already implemented
```

**Bass Generative Core (4 pots):**
```rust
Pot 3 (GPIO4): Bass Clock Rate    (0.5-8 Hz)
Pot 4 (GPIO5): Bass Root Note     (30-300 Hz)
Pot 5 (GPIO6): Bass Scale         (12 scales)
Pot 6 (GPIO7): Bass Octave Range  (0-4 octaves)
```

**Rhythmic Variation (2 pots - NEW):**
```rust
Pot 7 (GPIO8): Gate Probability   (30-100%) ← Rhythm density
Pot 8 (GPIO9): Envelope Max Time  (10-500ms) ← Note length variation
```

**Experimental (4 pots - Reprogrammable):**
```rust
Pot 9 (GPIO10):  Filter LFO Rate  (0.01-10 Hz) ← Test if pot needed vs hardcoded 1 Hz
Pot 10 (GPIO11): <Spare>          ← Test glide pot, pitch LFO, or clock division
Pot 11 (GPIO12): <Spare>          ← Additional test parameter
```

**Hardcoded:**
- Main OSC: Saw waveform, 70% level
- Sub OSC: Square -1 octave, 30% level (free bass thickness)
- Bass glide: 150ms (test if pot needed)
- Envelope attack: 5ms (fast, percussive-capable)
- Envelope min time: 10ms (pot controls max range 10-500ms)
- Lead voice: OFF (not implemented yet)
- Delay: OFF (not implemented yet)
- Pitch LFO: OFF (Phase 4E testing)
- Master Volume: 0.9 (90%, provides headroom)

**Implementation Tasks:**
1. [ ] Create `src/noise.rs` - WhiteNoise LFSR
2. [ ] Create `src/sample_hold.rs` - S&H with clock + gate probability
3. [ ] Create `src/quantizer.rs` - 12 musical scales
4. [ ] Create `src/envelope.rs` - Simple AD envelope (5ms attack, random decay)
5. [ ] Create `src/generative.rs` - GenerativeVoice wrapper (integrates all layers)
6. [ ] Add glide/portamento to oscillator frequency changes (hardcoded 150ms slew)
7. [ ] Simplify oscillators to 2 (Main saw + Sub square)
8. [ ] Add `bass_voice: GenerativeVoice` to Engine
9. [ ] Update `update_buffer_parameters()` - tick bass voice at buffer rate
10. [ ] Add bass control messages to Message enum (clock, root, scale, octave, gate prob, env time)
11. [ ] Add bass mapping functions to controls/pot.rs
12. [ ] Wire 6 new pots (GPIO7-12) for bass generative controls
13. [ ] Update controls/task.rs to poll 12 pots and route to bass messages
14. [ ] Build and test with real pots
15. [ ] Play for 1 week, document findings

**What You Learn:**
- ✅ Does generative bass with THREE layers (pitch, rhythm, articulation) feel expressive and non-boring?
- ✅ Does gate probability add musicality (syncopation, breathing room) or just chaos?
- ✅ Do random envelope times create interesting articulation or feel random/disconnected?
- ✅ Which scales get used most (minor vs pentatonic vs blues)?
- ✅ What clock rates feel musical (2 Hz, 4 Hz, 8 Hz)?
- ✅ What gate probabilities sound best (50%, 70%, 90%)?
- ✅ Is octave range control needed, or can it be fixed at 2 octaves?
- ✅ Is 150ms glide right, or too fast/slow? Does it need a pot?
- ✅ **Filter LFO Rate pot**: Essential for performance, or can it be hardcoded at 1 Hz?
- ✅ **Pitch LFO**: Worth testing in Phase 4E, or skip entirely?

**Timeline**: 1 weekend (10-14 hours coding + testing for 1 week)

---

### Phase 4B: Lead Generative Only (12 pots, reprogram test bed)

**Goal**: Test if fast generative with rhythm/articulation variation creates "simple melodic pattern" for lead.

**Pot Allocation:**

**Core Performance (3 pots - UNCHANGED):**
```rust
Pot 0-2: Same as Phase 4A (cutoff, resonance, LFO depth)
```

**Lead Generative Test Bed (9 pots - REPROGRAMMED):**
```rust
Pot 3: Lead Clock Rate       (4-20 Hz) ← Faster than bass
Pot 4: Lead Root Note        (200-1024 Hz) ← Higher register
Pot 5: Lead Scale            (12 scales)
Pot 6: Lead Octave Range     (0-4 octaves)
Pot 7: Lead Gate Probability (30-100%) ← Rhythm density
Pot 8: Lead Envelope Time    (10-500ms) ← Note length variation
Pot 9: Filter LFO Rate       (0.01-10 Hz) ← Test if needed
Pot 10: <Spare>              ← Test glide pot if needed
Pot 11: <Spare>
```

**Hardcoded:**
- Bass voice: OFF (switch to lead mode)
- Lead glide: 50ms (faster than bass)
- Oscillators: Same (main saw + sub)
- Delay: OFF

**Implementation Tasks:**
1. [ ] Add `lead_voice: GenerativeVoice` to Engine
2. [ ] Route lead voice to oscillators (with 50ms glide)
3. [ ] Update controls/task.rs - reprogram pots 3-11 to lead controls

**What You Learn:**
- ✅ Fast clock rates (8-16 Hz) with rhythm variation = glitchy or musical?
- ✅ Does 50ms glide work for lead, or too fast/slow?
- ✅ Lead feels different from bass (higher register + faster clock)?
- ✅ Different gate probabilities for lead (sparser = more interesting)?
- ✅ Different scales for lead create interesting sounds?

**Timeline**: 4 hours (reprogram + test for 1 week)

---

### Phase 4C: Dual Voice (Bass + Lead Simultaneously) - Needs 16+ Pots

**Goal**: Test if simultaneous bass+lead with independent rhythm/articulation is richer than single voice.

**Pot Allocation (MINIMUM 16 pots required):**

**Core Performance (4 pots):**
```rust
Pot 0: Filter Cutoff
Pot 1: Filter Resonance
Pot 2: Filter LFO Depth
Pot 3: Master Volume
```

**Bass Generative (4 pots - Essential):**
```rust
Pot 4: Bass Clock Rate
Pot 5: Bass Root Note
Pot 6: Bass Scale
Pot 7: Bass Octave Range
```

**Lead Generative (4 pots - Essential):**
```rust
Pot 8:  Lead Clock Rate
Pot 9:  Lead Root Note
Pot 10: Lead Scale
Pot 11: Lead Octave Range
```

**Rhythm/Articulation (4 pots - Shared or per-voice):**
```rust
Pot 12: Gate Probability (shared, or bass-specific)
Pot 13: Envelope Time (shared, or bass-specific)
Pot 14: Filter LFO Rate
Pot 15: <Spare> (lead gate prob if needed)
```

**Implementation Tasks:**
1. [ ] Wire 4-6 more pots (second ADS1115 module)
2. [ ] Enable both bass_voice AND lead_voice simultaneously
3. [ ] Test shared vs independent gate probability/envelope controls
4. [ ] Test polyrhythmic combinations (bass 2 Hz, lead 6 Hz)
5. [ ] Test polymodal (bass minor, lead major)

**What You Learn:**
- ✅ **CRITICAL**: Is simultaneous bass+lead essential, or is single voice + delay enough?
- ✅ Polyrhythmic patterns (2:3, 1:4) = interesting or overwhelming?
- ✅ Do bass+lead need independent gate probability, or can they share one control?
- ✅ Managing 2 independent generative voices = fun or too complex?
- ✅ Does dual voice sound better than single voice + delay?

**Timeline**: Wait for AliExpress pots, then 1 day to wire + test

---

### Phase 4D: Add Delay (Needs 18+ Pots)

**Goal**: Add delay for pseudo-polyphonic texture and evolving echoes.

**Option A: Full Control (3 pots):**
```rust
Pot 16: Delay Time (50ms - 2 seconds) - Echo spacing
Pot 17: Delay Feedback (0-110%) - Echo intensity (>100% = self-oscillation)
Pot 18: Delay Mix (0-100%) - Dry/Wet balance
```

**Option B: Essential Control (2 pots, recommended):**
```rust
Pot 16: Delay Time (50ms - 2 seconds) - Echo spacing (CRITICAL for rhythmic sync)
Pot 17: Delay Mix (0-100%) - Dry/Wet balance (CRITICAL for performance)
Hardcoded: Feedback = 40% (can adjust in software if needed)
```

**Reasoning for Option B:**
- **Delay Time**: Must be adjustable for rhythmic sync with generative patterns
- **Delay Mix**: Essential performance control (blend delay in/out)
- **Feedback**: Can be constant (40% is sweet spot, rarely needs tweaking)
- Saves 1 pot for other features (Pitch LFO, glide, etc.)

**Implementation Tasks:**
1. [ ] Implement PSRAM delay buffer (ring buffer)
2. [ ] Add linear interpolation for variable delay time
3. [ ] Add feedback saturation (prevent runaway if >100%)
4. [ ] Wire 2-3 pots for delay controls
5. [ ] Test both control schemes (decide 2-pot vs 3-pot)

**What You Learn:**
- ✅ Does delay make simple patterns more interesting?
- ✅ Delay feedback creates self-evolving textures?
- ✅ Is delay essential for "freshness," or just nice-to-have?
- ✅ **Is Feedback pot needed, or is 40% constant good enough?**

**Timeline**: 2-3 days (delay is complex - PSRAM buffer + interpolation)

---

### Phase 4E: Full System (18-19 pots)

**Goal**: All features active, test extensively for 2-4 weeks.

**Additional Features (+2 pots - TESTING ONLY):**
- **Pitch LFO** (rate, depth) - Vibrato/wobble on generative notes
  - Test: Does it add expressiveness, or is it redundant with glide?
  - Test: Does it clash with generative pitch changes?

**Removed from testing (now hardcoded constants):**
- ~~Main OSC Level~~ - Hardcoded at 70%
- ~~Sub OSC Level~~ - Hardcoded at 30%

**What You Learn:**
- ✅ Which 10-13 pots get used 90% of the time?
- ✅ **Is Pitch LFO essential, redundant, or actively harmful?**
- ✅ Can any features be combined or hardcoded?
- ✅ Final pot layout based on real usage data

**Output**: Final pot layout for pocket version (minimum 11 pots, maximum 13 pots recommended)

---

## Implementation Details

**For architectural patterns (processing rates, smoothing, performance optimization), see ARCHITECTURE.md**

### 1. Simplified Oscillator Architecture (Phase 4A)

**Current**: 4 oscillators (Sine, Saw, Square, Sub)

**Simplified**: 2 oscillators (Main Saw + Sub Square)

```rust
// src/engine.rs

oscillators: [
    Oscillator::new(Waveform::Saw),  // Main voice (rich harmonics)
    {
        let mut sub = Oscillator::new(Waveform::Square);
        sub.set_frequency(FREQUENCY_MIN / 2.0); // -1 octave
        sub.set_level(0.3); // Hardcoded 30% mix (no pot needed)
        sub
    },
],
```

**Why:**
- Saw = richest harmonic content (all harmonics present)
- Sub = free bass thickness (hardcoded, no pot)
- Filter resonance adds additional harmonics (acts like extra OSC)
- Moog Minimoog used 3 OSC but famous sounds often use 1 saw + filter

**Can expand to 4 oscillators later if 2 feels thin.**

---

### 2. White Noise Generator

```rust
// src/noise.rs

/// White noise using Linear Feedback Shift Register (LFSR).
/// Fast PRNG for random voltage generation.
pub struct WhiteNoise {
    lfsr: u32, // Galois LFSR state
}

impl WhiteNoise {
    pub fn new() -> Self {
        Self { lfsr: 0xACE1u32 } // Non-zero seed required
    }

    /// Generate next noise sample (run at BUFFER RATE for efficiency).
    #[inline(always)]
    pub fn tick(&mut self) -> f32 {
        // Galois LFSR: shift right, XOR feedback on LSB=1
        let lsb = self.lfsr & 1;
        self.lfsr >>= 1;
        if lsb == 1 {
            self.lfsr ^= 0xB400u32; // Taps for maximal-length sequence
        }

        // Convert u32 to normalized float [-1.0, 1.0]
        ((self.lfsr as i32) as f32) / (i32::MAX as f32)
    }
}
```

**CPU cost**: ~5 cycles/sample (much cheaper than `rand()` crate)

---

### 3. Sample & Hold (with Gate Probability)

```rust
// src/sample_hold.rs

/// Sample and Hold circuit with probabilistic triggering.
/// Creates stepped random voltages with syncopation/rests.
pub struct SampleHold {
    held_value: f32,         // Currently held output
    clock_phase: f32,        // Internal clock [0.0, 1.0)
    clock_freq: f32,         // Clock frequency in Hz
    phase_increment: f32,    // Cached: clock_freq / BUFFER_RATE
    prev_clock_high: bool,   // Edge detection
    gate_probability: f32,   // 0.0-1.0 (trigger chance)
    noise_for_gate: WhiteNoise, // Separate RNG for gate decisions
    triggered: bool,         // Did this clock tick trigger?
}

impl SampleHold {
    pub fn new(clock_freq: f32) -> Self {
        let phase_increment = clock_freq / crate::config::BUFFER_RATE;
        Self {
            held_value: 0.0,
            clock_phase: 0.0,
            clock_freq,
            phase_increment,
            prev_clock_high: false,
            gate_probability: 0.7, // Default: 70% density
            noise_for_gate: WhiteNoise::new(),
            triggered: false,
        }
    }

    /// Process one sample (BUFFER RATE recommended for CPU savings).
    /// Returns (held_value, trigger_flag)
    #[inline]
    pub fn tick(&mut self, input: f32) -> (f32, bool) {
        // Generate internal clock (square wave)
        let clock_high = self.clock_phase < 0.5;
        self.triggered = false;

        // Sample on rising edge (clock goes low → high)
        if clock_high && !self.prev_clock_high {
            // Probabilistic gate check
            let gate_random = (self.noise_for_gate.tick() + 1.0) / 2.0; // Normalize to [0, 1]

            if gate_random < self.gate_probability {
                self.held_value = input; // SAMPLE new value
                self.triggered = true;   // Signal envelope trigger
            }
            // Else: hold previous value (skip this trigger = rest)
        }
        self.prev_clock_high = clock_high;

        // Advance clock phase (uses cached phase_increment to avoid division)
        self.clock_phase += self.phase_increment;
        if self.clock_phase >= 1.0 {
            self.clock_phase -= 1.0;
        }

        (self.held_value, self.triggered) // Return value + trigger flag
    }

    pub fn set_clock_freq(&mut self, freq: f32) {
        self.clock_freq = freq.clamp(0.1, 20.0);
        self.phase_increment = self.clock_freq / crate::config::BUFFER_RATE;
    }

    pub fn set_gate_probability(&mut self, prob: f32) {
        self.gate_probability = prob.clamp(0.0, 1.0);
    }
}
```

**CPU cost**: ~18 cycles/tick (10 cycles base + 8 cycles for gate probability check)

---

### 4. Scale Quantizer

```rust
// src/quantizer.rs

use crate::config::GENERATIVE_OCTAVE_RANGE_MAX;

/// Precomputed semitone ratios for fast frequency conversion.
/// Maps semitone offsets to frequency ratios: 2^(n/12)
/// Eliminates expensive powf() call (100-200 cycles) with array lookup (2-3 cycles).
///
/// Array size derived from GENERATIVE_OCTAVE_RANGE_MAX:
/// - Range: ±(4.0 octaves × 12 semitones) = ±48 semitones
/// - Size: 48 + 1 + 48 = 97 elements
const SEMITONE_RANGE: i32 = (GENERATIVE_OCTAVE_RANGE_MAX as i32) * 12;  // 48
const SEMITONE_OFFSET: i32 = SEMITONE_RANGE;  // Center offset for array indexing
const ARRAY_SIZE: usize = (SEMITONE_RANGE * 2 + 1) as usize;  // 97

const SEMITONE_RATIOS: [f32; ARRAY_SIZE] = [
    0.06250, 0.06622, 0.07015, 0.07433, 0.07875, 0.08343, 0.08839, 0.09364,
    0.09921, 0.10511, 0.11136, 0.11798, 0.12500, 0.13243, 0.14031, 0.14865,
    0.15749, 0.16685, 0.17678, 0.18729, 0.19843, 0.21022, 0.22272, 0.23597,
    0.25000, 0.26487, 0.28062, 0.29730, 0.31498, 0.33371, 0.35355, 0.37458,
    0.39685, 0.42045, 0.44545, 0.47194, 0.50000, 0.52973, 0.56123, 0.59460,
    0.62996, 0.66742, 0.70711, 0.74915, 0.79370, 0.84090, 0.89090, 0.94387,
    1.00000, 1.05946, 1.12246, 1.18921, 1.25992, 1.33484, 1.41421, 1.49831,
    1.58740, 1.68179, 1.78180, 1.88775, 2.00000, 2.11893, 2.24492, 2.37841,
    2.51984, 2.66968, 2.82843, 2.99661, 3.17480, 3.36359, 3.56359, 3.77550,
    4.00000, 4.23785, 4.48985, 4.75683, 5.03968, 5.33936, 5.65685, 5.99323,
    6.34960, 6.72717, 7.12719, 7.55099, 8.00000, 8.47570, 8.97970, 9.51366,
    10.07937, 10.67872, 11.31371, 11.98646, 12.69921, 13.45434, 14.25438, 15.10199,
    16.00000,
];

/// 12 musical scales as semitone intervals from root.
/// Each array: 12 semitone offsets covering ~2 octaves.
const SCALES: [[i8; 12]; 12] = [
    // 0: Major (Ionian) - C D E F G A B
    [0, 2, 4, 5, 7, 9, 11, 12, 14, 16, 17, 19],

    // 1: Natural Minor (Aeolian) - C D Eb F G Ab Bb
    [0, 2, 3, 5, 7, 8, 10, 12, 14, 15, 17, 19],

    // 2: Harmonic Minor - C D Eb F G Ab B
    [0, 2, 3, 5, 7, 8, 11, 12, 14, 15, 17, 19],

    // 3: Pentatonic Minor - C Eb F G Bb
    [0, 3, 5, 7, 10, 12, 15, 17, 19, 22, 24, 27],

    // 4: Pentatonic Major - C D E G A
    [0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24, 26],

    // 5: Blues Scale - C Eb F F# G Bb
    [0, 3, 5, 6, 7, 10, 12, 15, 17, 18, 19, 22],

    // 6: Dorian - C D Eb F G A Bb
    [0, 2, 3, 5, 7, 9, 10, 12, 14, 15, 17, 19],

    // 7: Phrygian - C Db Eb F G Ab Bb
    [0, 1, 3, 5, 7, 8, 10, 12, 13, 15, 17, 19],

    // 8: Chromatic (all notes)
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],

    // 9: Whole Tone - C D E F# G# A#
    [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22],

    // 10: Diminished - C D Eb F Gb Ab A B
    [0, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17],

    // 11: Japanese Pentatonic - C Db F G Ab
    [0, 1, 5, 7, 8, 12, 13, 17, 19, 20, 24, 25],
];

pub struct Quantizer {
    scale_index: usize,    // Current scale (0-11)
    root_freq: f32,        // Root note frequency
    octave_range: f32,     // Jump range (1-4 octaves)
}

impl Quantizer {
    pub fn new(root_freq: f32, octave_range: f32) -> Self {
        Self {
            scale_index: 1, // Default: Natural Minor
            root_freq,
            octave_range,
        }
    }

    /// Quantize input to nearest scale note.
    ///
    /// # Arguments
    /// * `input` - Normalized [-1.0, 1.0] from S&H
    ///
    /// # Returns
    /// Frequency in Hz snapped to current scale
    #[inline]
    pub fn quantize(&self, input: f32) -> f32 {
        // Map input to semitone range
        let semitones = input * self.octave_range * 12.0;
        let scale = &SCALES[self.scale_index];
        let semitone_int = semitones.round() as i8;

        // Find closest scale degree (linear search)
        let mut closest = scale[0];
        let mut min_dist = (semitone_int - closest).abs();
        for &note in scale.iter() {
            let dist = (semitone_int - note).abs();
            if dist < min_dist {
                min_dist = dist;
                closest = note;
            }
        }

        // Convert semitones to frequency using lookup table
        // Index: closest + SEMITONE_OFFSET maps [-48..+48] to [0..96]
        let index = (closest + SEMITONE_OFFSET).clamp(0, (ARRAY_SIZE - 1) as i32) as usize;
        self.root_freq * SEMITONE_RATIOS[index]
    }

    pub fn set_scale(&mut self, normalized: f32) {
        self.scale_index = (normalized * 11.99).floor() as usize;
    }

    pub fn set_root(&mut self, freq: f32) {
        self.root_freq = freq;
    }

    pub fn set_octave_range(&mut self, octaves: f32) {
        self.octave_range = octaves.clamp(
            crate::config::GENERATIVE_OCTAVE_RANGE_MIN,
            crate::config::GENERATIVE_OCTAVE_RANGE_MAX,
        );
    }
}
```

**CPU cost**: ~15 cycles (linear search + array lookup, no powf)

---

### 5. Envelope Generator (AD)

```rust
// src/envelope.rs

/// Simple Attack-Decay envelope for note articulation.
/// Fixed fast attack (5ms), random decay time per note.
pub struct ADEnvelope {
    stage: EnvelopeStage,
    level: f32,              // Current envelope level [0.0, 1.0]
    attack_increment: f32,   // Precalculated rise rate
    decay_increment: f32,    // Current decay rate (varies per note)
    decay_time_ms: f32,      // Target decay time in ms
    noise_for_decay: WhiteNoise, // RNG for random decay times
}

#[derive(PartialEq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
}

impl ADEnvelope {
    pub fn new() -> Self {
        let sample_rate = crate::config::SAMPLE_RATE;
        // Fixed 5ms attack
        let attack_samples = (0.005 * sample_rate) as f32;
        let attack_increment = 1.0 / attack_samples;

        Self {
            stage: EnvelopeStage::Idle,
            level: 0.0,
            attack_increment,
            decay_increment: 0.0,
            decay_time_ms: 100.0, // Default
            noise_for_decay: WhiteNoise::new(),
        }
    }

    /// Trigger envelope with random decay time.
    pub fn trigger(&mut self, decay_time_range: (f32, f32)) {
        self.stage = EnvelopeStage::Attack;
        self.level = 0.0;

        // Random decay time from range (e.g., 10-500ms)
        let random = (self.noise_for_decay.tick() + 1.0) / 2.0; // [0, 1]
        self.decay_time_ms = decay_time_range.0 + random * (decay_time_range.1 - decay_time_range.0);

        // Precalculate decay rate
        let sample_rate = crate::config::SAMPLE_RATE;
        let decay_samples = (self.decay_time_ms / 1000.0 * sample_rate) as f32;
        self.decay_increment = 1.0 / decay_samples;
    }

    /// Process one sample (AUDIO RATE).
    #[inline]
    pub fn tick(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Attack => {
                self.level += self.attack_increment;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                self.level -= self.decay_increment;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
            EnvelopeStage::Idle => {
                self.level = 0.0;
            }
        }

        self.level
    }
}
```

**CPU cost**: ~10 cycles/sample at audio rate (44.1 kHz) - simple state machine + arithmetic

**Why audio rate?** Envelope must run at audio rate for smooth amplitude changes. Buffer-rate envelopes would create stepping artifacts (zipper noise).

---

### 6. Generative Voice (Wrapper)

```rust
// src/generative.rs

pub struct GenerativeVoice {
    noise: WhiteNoise,
    sample_hold: SampleHold,
    quantizer: Quantizer,
    envelope: ADEnvelope,
    decay_time_range: (f32, f32), // Min/max decay time in ms
    current_pitch: f32,           // Current output pitch
}

impl GenerativeVoice {
    pub fn new(root_freq: f32, octave_range: f32, clock_freq: f32) -> Self {
        Self {
            noise: WhiteNoise::new(),
            sample_hold: SampleHold::new(clock_freq),
            quantizer: Quantizer::new(root_freq, octave_range),
            envelope: ADEnvelope::new(),
            decay_time_range: (10.0, 150.0), // Default: 10-150ms
            current_pitch: root_freq,
        }
    }

    /// Generate next pitch (BUFFER RATE ~43 Hz recommended).
    /// Returns target pitch for glide/portamento.
    #[inline]
    pub fn tick(&mut self) -> f32 {
        let noise = self.noise.tick();
        let (sh_value, triggered) = self.sample_hold.tick(noise);

        // Quantize to scale
        self.current_pitch = self.quantizer.quantize(sh_value);

        // Trigger envelope on new notes
        if triggered {
            self.envelope.trigger(self.decay_time_range);
        }

        self.current_pitch
    }

    /// Get envelope level (AUDIO RATE, called per sample).
    #[inline]
    pub fn envelope_level(&mut self) -> f32 {
        self.envelope.tick()
    }

    // Setters
    pub fn set_clock_freq(&mut self, freq: f32) {
        self.sample_hold.set_clock_freq(freq);
    }

    pub fn set_gate_probability(&mut self, prob: f32) {
        self.sample_hold.set_gate_probability(prob);
    }

    pub fn set_decay_time_range(&mut self, min_ms: f32, max_ms: f32) {
        self.decay_time_range = (min_ms, max_ms);
    }

    pub fn set_scale(&mut self, normalized: f32) {
        self.quantizer.set_scale(normalized);
    }

    pub fn set_root(&mut self, freq: f32) {
        self.quantizer.set_root(freq);
    }

    pub fn set_octave_range(&mut self, octaves: f32) {
        self.quantizer.set_octave_range(octaves);
    }
}
```

---

### 7. Engine Integration

```rust
// src/engine.rs

pub struct Engine {
    // Simplified oscillators (2 instead of 4)
    oscillators: [Oscillator; 2],

    filter: MoogLadderFilter,
    filter_lfo: LFO,

    // Generative voices
    bass_voice: GenerativeVoice,
    lead_voice: Option<GenerativeVoice>, // None in Phase 4A/4B, Some in 4C+

    // ... other fields ...
}

impl Engine {
    pub fn new(receiver: ChannelReceiver) -> Self {
        // Initialize bass voice (30-300 Hz range, 4 Hz clock)
        let bass_voice = GenerativeVoice::new(
            60.0,  // Root: C2 (60 Hz)
            2.0,   // Octave range: 2 octaves
            4.0,   // Clock: 4 Hz
        );

        Self {
            oscillators: [
                Oscillator::new(Waveform::Saw),
                {
                    let mut sub = Oscillator::new(Waveform::Square);
                    sub.set_frequency(30.0);
                    sub.set_level(0.3); // Hardcoded 30% mix
                    sub
                },
            ],
            bass_voice,
            lead_voice: None, // Add in Phase 4C
            // ... other fields ...
        }
    }

    /// BUFFER RATE (~43 Hz): Update generative pitch + filter.
    #[inline]
    fn update_buffer_parameters(&mut self) {
        // Generate bass pitch target (buffer rate)
        let bass_pitch = self.bass_voice.tick();

        // Set frequency with glide (oscillator has internal Slew for 150ms portamento)
        self.oscillators[0].set_frequency(bass_pitch);      // Main saw
        self.oscillators[1].set_frequency(bass_pitch / 2.0); // Sub (-1 octave)

        // Lead voice (Phase 4C+)
        if let Some(ref mut lead_voice) = self.lead_voice {
            let lead_pitch = lead_voice.tick();
            // Route to oscillators OR separate voice (TBD based on architecture)
        }

        // Update oscillator parameters (glide smoothing)
        for osc in &mut self.oscillators {
            osc.update_parameters();
        }

        // Filter LFO modulation (already implemented)
        let lfo_value = self.filter_lfo.tick();
        let modulated_cutoff = self.base_cutoff + (lfo_value * self.lfo_cutoff_range);
        self.filter.set_cutoff(modulated_cutoff);
        self.filter.update_coefficients();
    }

    /// AUDIO RATE (44.1 kHz): Render one sample with envelope.
    #[inline]
    fn render_sample(&mut self) -> f32 {
        // Get envelope level (audio rate)
        let env_level = self.bass_voice.envelope_level();

        // Mix oscillators
        let osc_mix: f32 = self.oscillators.iter_mut()
            .map(|osc| osc.tick())
            .sum();

        // Apply envelope to mixed signal
        let enveloped = osc_mix * env_level;

        // Filter
        self.filter.process(enveloped)
    }
}
```

**Critical Optimization**:
- **Pitch generation** (noise → S&H → quantizer): Buffer rate (43 Hz) → 1021× CPU savings
- **Envelope**: Audio rate (44.1 kHz) → necessary for smooth amplitude (no zipper noise)
- **Result**: Expressive articulation with minimal CPU cost

---

## Message System

```rust
// src/message.rs

pub enum Message {
    // Existing messages
    SetOscLevel(OscillatorId, f32),
    SetCutoff(f32),
    SetResonance(f32),
    SetFilterLfoRate(f32),
    SetFilterLfoDepth(f32),

    // Bass generative (Phase 4A)
    SetBassClockRate(f32),        // 0.5-8 Hz
    SetBassScale(f32),            // 0.0-1.0 → 12 scales
    SetBassOctaveRange(f32),      // 0-4 octaves
    SetBassRoot(f32),             // 30-300 Hz
    SetBassGateProbability(f32),  // 0.3-1.0 (30-100%)
    SetBassEnvelopeTime(f32),     // 10-500ms (max time, min is 10ms)

    // Lead generative (Phase 4B+)
    SetLeadClockRate(f32),        // 4-20 Hz
    SetLeadScale(f32),            // 0.0-1.0 → 12 scales
    SetLeadOctaveRange(f32),      // 0-4 octaves
    SetLeadRoot(f32),             // 200-1024 Hz
    SetLeadGateProbability(f32),  // 0.3-1.0
    SetLeadEnvelopeTime(f32),     // 10-500ms

    // Delay (Phase 4D)
    SetDelayTime(f32),            // 50ms - 2s
    SetDelayFeedback(f32),        // 0-110%
    SetDelayMix(f32),             // 0-100%

    // Pitch modulation (Phase 4E)
    SetPitchLfoRate(f32),         // 0.01-10 Hz
    SetPitchLfoDepth(f32),        // 0.0-0.5 (±50¢)
}
```

---

## Control Mapping Functions

```rust
// src/controls/pot.rs

/// Map pot to S&H clock rate (exponential).
pub fn map_clock_rate(normalized: f32, max_freq: f32) -> f32 {
    let log_min = 0.1_f32.ln();
    let log_max = max_freq.ln();
    (log_min + normalized * (log_max - log_min)).exp()
}

/// Map pot to bass root (exponential: 30-300 Hz).
pub fn map_bass_root(normalized: f32) -> f32 {
    let log_min = 30.0_f32.ln();
    let log_max = 300.0_f32.ln();
    (log_min + normalized * (log_max - log_min)).exp()
}

/// Map pot to lead root (exponential: 200-1024 Hz).
pub fn map_lead_root(normalized: f32) -> f32 {
    let log_min = 200.0_f32.ln();
    let log_max = 1024.0_f32.ln();
    (log_min + normalized * (log_max - log_min)).exp()
}

/// Map pot to octave range (linear).
pub fn map_octave_range(normalized: f32) -> f32 {
    use crate::config::{GENERATIVE_OCTAVE_RANGE_MIN, GENERATIVE_OCTAVE_RANGE_MAX};
    GENERATIVE_OCTAVE_RANGE_MIN + (normalized * (GENERATIVE_OCTAVE_RANGE_MAX - GENERATIVE_OCTAVE_RANGE_MIN))
}

/// Map pot to scale selector (12 discrete positions).
/// Quantizer::set_scale() handles discretization internally.
pub fn map_scale_selector(normalized: f32) -> f32 {
    normalized
}

/// Map pot to generative amount (linear: 0-100%).
pub fn map_generative_amount(normalized: f32) -> f32 {
    normalized
}
```

---

## CPU Budget Analysis

**Per-voice cost at buffer rate (~43 Hz):**
- WhiteNoise: ~5 cycles
- SampleHold (with gate probability): ~18 cycles (includes extra RNG check)
- Quantizer: ~15 cycles (linear search + array lookup, no powf)
- **Total per buffer tick**: ~38 cycles

**Per-voice cost at audio rate (44.1 kHz):**
- Envelope: ~10 cycles/sample (simple state machine)

**Two voices - Complete cost:**
- **Buffer rate** (43 Hz): 38 cycles × 2 voices × 43 Hz = **3,268 cycles/sec** (negligible)
- **Audio rate** (44.1 kHz): 10 cycles × 2 envelopes × 44,100 Hz = **882K cycles/sec**

**Total generative system**: ~882K cycles/sec for dual voice with full articulation

**Compared to naive audio-rate implementation:**
- Audio rate pitch generation: 38 cycles × 2 × 44,100 Hz = **3.35M cycles/sec**
- Our approach: 882K cycles/sec
- **Savings**: 3.8× more efficient

**Verdict**:
- Run pitch generation (noise → S&H → quantizer) at **buffer rate** → 1021× savings
- Run envelope at **audio rate** (necessary for smooth articulation)
- Combined approach: Best of both worlds (efficient + expressive)

---

## Musical Performance Examples

### Deep Techno Bass
```
Bass Clock: 4 Hz
Bass Scale: Minor pentatonic
Bass Octave Range: 1 octave (stepwise motion)
Bass Gate Probability: 70% (some rests, not metronomic)
Bass Envelope Time: 50-120ms (short plucks)
Lead: OFF

YOU perform: Filter sweeps, resonance peaks, gate probability changes

Result: Syncopated, punchy bass line with breathing room, YOU shape timbre + rhythm density
```

### Aphex Twin Glitch
```
Bass Clock: 4 Hz
Lead Clock: 12 Hz (3:1 polyrhythm)
Both Amount: 100%
Bass Scale: Harmonic minor
Lead Scale: Chromatic

YOU perform: Filter cutoff sweeps, resonance spikes, LFO depth

Result: Glitchy, polyrhythmic, raw and chaotic
```

### Ambient Drone
```
Bass Clock: 0.5 Hz (2 seconds/note)
Bass Scale: Whole tone
Bass Octave Range: 3 octaves (dramatic jumps)
Bass Gate Probability: 90% (mostly dense, occasional silence)
Bass Envelope Time: 200-500ms (long sustains)
Lead: OFF
Filter LFO Depth: 100% (slow evolving filter)

YOU perform: Filter cutoff sweeps, octave range adjustments

Result: Slow evolution, long sustains, occasional dramatic jumps
```

### Morphing Build-Up
```
Start: Bass Gate Probability at 30% (sparse, lots of silence)
10s:   Gate Probability 30→70% (density increases)
30s:   Add lead voice (if dual voice mode)
45s:   Gate Probability 70→100% (full density)
1min:  Sweep filter cutoff open, crank resonance

Result: Live build-up performance from sparse to dense, YOU control evolution
```

---

## Success Criteria

### Phase 4A Complete When:
- ✅ Bass generative with THREE layers (pitch, rhythm, articulation) feels expressive and musical
- ✅ Gate probability creates natural syncopation and breathing room (not chaotic)
- ✅ Random envelope times add interesting articulation without feeling disconnected
- ✅ All 12 scales tested and sound different
- ✅ Clock rate controls rhythm feel (2 Hz = slow, 8 Hz = fast)
- ✅ 150ms glide feels right (smooth but articulated)
- ✅ Can perform filter sweeps + gate probability changes for 30+ minutes without boredom
- ✅ Understand which pots get used most (which are essential vs nice-to-have)

### Phase 4B Complete When:
- ✅ Lead generative creates simple melodic patterns
- ✅ Fast clock rates (8-16 Hz) create interesting textures
- ✅ Different scales for lead tested

### Phase 4C Complete When:
- ✅ **CRITICAL**: Understand if simultaneous bass+lead is essential or if single voice + delay is enough
- ✅ Polyrhythmic combinations tested (bass 2 Hz, lead 6 Hz)
- ✅ Can perform filter sweeps on dual generative voices

### Phase 4D Complete When:
- ✅ Delay creates pseudo-polyphonic texture
- ✅ Delay feedback self-oscillation tested (>100% feedback)
- ✅ Understand if delay is essential for "freshness"

### Phase 4E Complete When:
- ✅ All features tested for 2-4 weeks
- ✅ Usage data collected (which pots get tweaked most?)
- ✅ Final 10-12 pot layout decided for pocket version

---

## Next Actions (Phase 4A - This Weekend)

**Implementation tasks** (~8-12 hours):

1. [ ] Create `src/noise.rs` - WhiteNoise LFSR
2. [ ] Create `src/sample_hold.rs` - S&H with clock
3. [ ] Create `src/quantizer.rs` - 12 scales + quantize logic
4. [ ] Create `src/generative.rs` - GenerativeVoice wrapper
5. [ ] Simplify `src/engine.rs` oscillators to 2 (main saw + sub square)
6. [ ] Add `bass_voice: GenerativeVoice` to Engine
7. [ ] Update `update_buffer_parameters()` to tick bass voice
8. [ ] Add bass control messages to Message enum
9. [ ] Add bass mapping functions to controls/pot.rs
10. [ ] Wire 6 new pots (GPIO7-12) for bass generative controls
11. [ ] Update controls/task.rs to poll 12 pots and route to bass messages
12. [ ] Build and test with real pots
13. [ ] Play for 1 week, document findings

**Questions to answer during Phase 4A testing:**
- Does generative bass feel "simple and repetitive" (good) or "random chaos" (bad)?
- Which clock rates feel most musical (1 Hz, 2 Hz, 4 Hz)?
- Which scales get used most (minor pentatonic, harmonic minor, blues)?
- Is octave range control needed, or can it be fixed at 2 octaves?
- **Is Filter LFO Rate pot needed, or can it be hardcoded at 1 Hz?**
- **Should Pitch LFO be tested in Phase 4E, or skip entirely?**
- Are OSC levels at 70%/30% good, or do they need adjustment in software?

---

## Terminology & Context

**What this is called:**
- **Probabilistic Generative Sequencer** (most accurate - uses gate probability + quantized randomness)
- **Quantized Random Pattern Generator** (describes the musical constraint)
- **Algorithmic Melody Generator** (composition system, not just random CV)
- **Aleatoric Synthesis** (controlled randomness with musical rules)

**Closest commercial hardware:**
- **Mutable Instruments Marbles** ($350, Eurorack) - Random gates + quantized voltages, but requires modular setup
- **Turing Machine** ($200, Eurorack) - Random sequencer, but no built-in quantizer or synthesis
- **Ornament & Crime** ($200, Eurorack) - Quantizer + generative algorithms, but requires CV sources
- Buchla Music Easel ($4000+) - Has random voltage, but it's one feature in a massive system
- Moog Subharmonicon ($700) - Polyrhythmic but NOT generative (fixed sequences)

**Artists using similar approaches:**
- **Aphex Twin** - Custom MAX/MSP generative systems, Buchla Music Easel
- **Morton Subotnick** - Pioneered aleatoric electronic music (Silver Apples of the Moon, 1967)
- **Suzanne Ciani** - Buchla generative concerts (1970s)
- **Richard Devine** - Modular generative patch programming
- **Autechre** - Algorithmic composition systems (Max/MSP, Supercollider)
- **Mutable Instruments community** - Marbles-based generative patches

**Why your synth is unique:**
- **Standalone** - No modular rack, no computer, no MIDI, no screens
- **Pocket-portable** - Self-contained generative instrument
- **Three-layer variation** - Pitch (scales) + rhythm (gate probability) + articulation (envelopes) in one device
- **Performance-focused** - Real-time control over generative density, not just preset playback
- **Analog control** - Direct pot manipulation, tactile and immediate

**Nothing else like this exists.** Marbles is the closest concept, but requires a full Eurorack system ($2000+ with case, power, VCO, VCA, etc.). Your synth does it all in a pocket-sized package.

---

This is going to be incredible. Let's build it. 🎛️
