# Generative Synthesis System - Implementation Plan

**Vision**: Raw, analog pocket synth with generative bass + lead + delay. Simple, repetitive patterns that YOU perform on (filter sweeps, modulation). No keys, no screens - just pots.

**Current State**: 6 pots wired, Phase 3.5 complete (4 oscillators, Moog filter, filter LFO)

**Hardware Available Now**: 12 pots total (6 wired + 5 B10K + 1 B5K in inventory)

**Hardware Arriving**: 10+ more pots from AliExpress (timeline TBD)

**Strategy**: Build FULL system on breadboard using phased pot allocation. Test extensively. Distill to pocket essentials based on real use.

---

## What Is Generative Synthesis?

**Generative = RHYTHMIC REPETITION with MELODIC VARIATION**

**Traditional synth**: You control pitch directly (pot sets ONE static note)

**Generative synth**: Algorithm creates simple, repetitive patterns, YOU perform timbre:
- Clock rate = steady rhythm (like drum machine)
- S&H = which notes get played (random but musical)
- Quantizer = always in key (snaps to scale)
- YOU control = filter sweeps, resonance, modulation depth

**Result**: Simple, raw, repetitive bass line or lead melody that never exactly repeats. YOU shape the sound in real-time.

### **Example: Bass Generative at 2 Hz (120 BPM 8th notes)**

```
Time:   0s   0.5s  1.0s  1.5s  2.0s  2.5s  3.0s  3.5s
Notes:  C    E♭    G     C     E♭    F     C     G
        └────┴─────┴─────┴─────┴─────┴─────┴─────┴──→
        Steady rhythm (2 Hz clock = metronomic)
        Notes change (S&H grabs random values)
        Always in key (quantizer → C minor scale)
```

**What you hear:**
- ✅ Repetitive rhythm (steady clock = predictable timing)
- ✅ Simple bass line (just notes, no fancy patterns)
- ✅ Raw and analog (no sequencer UI)
- ✅ Never exact repeat (S&H varies which notes)

**YOU perform:**
- Filter sweeps (open/close timbre)
- Resonance peaks (screaming Moog character)
- LFO depth changes (add movement)
- Clock rate changes (speed up/slow down live)

---

## Signal Chain Architecture

```
┌──────────────┐
│ White Noise  │ Random signal (LFSR, ~5 cycles/sample)
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│ Sample & Hold    │ Clock grabs random value at rhythm
│ Clock: 0.5-20 Hz │ ← Pot: Note Change Rate
└──────┬───────────┘
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

**Estimated Total: 19-21 pots**

### **PERFORMANCE (5 pots - Always Active)**

1. **Filter Cutoff** (165 Hz - 15 kHz) - Timbre shaping
2. **Filter Resonance** (0.1 - 2.0) - Moog character
3. **Filter LFO Rate** (0.01-10 Hz) - Movement speed
4. **Filter LFO Depth** (0-100%) - Movement amount
5. **Master Volume** (0.0-1.0) - Output level

### **BASS VOICE (5 pots)**

6. **Bass Clock Rate** (0.5-4 Hz) - Rhythm speed (slow bass)
7. **Bass Root Note** (30-300 Hz) - Tonal center (bass register)
8. **Bass Amount** (0-100%) - Manual ↔ Generative blend
9. **Bass Scale** (12 scales) - Minor, pentatonic, blues, etc.
10. **Bass Octave Range** (1-4 octaves) - Jump size (smooth vs dramatic)

### **LEAD VOICE (5 pots)**

11. **Lead Clock Rate** (4-20 Hz) - Fast rhythmic melody
12. **Lead Root Note** (200-1024 Hz) - Melodic register
13. **Lead Amount** (0-100%) - Manual ↔ Generative blend
14. **Lead Scale** (12 scales) - Can differ from bass (polymodal)
15. **Lead Octave Range** (1-4 octaves) - Melodic jump size

### **DELAY (3 pots)**

16. **Delay Time** (50ms - 2 seconds) - Echo spacing
17. **Delay Feedback** (0-110%) - Echo intensity
18. **Delay Mix** (0-100%) - Dry/Wet balance

### **PITCH MODULATION (2 pots - Optional)**

19. **Pitch LFO Rate** (0.01-10 Hz) - Vibrato speed
20. **Pitch LFO Depth** (0-50¢) - Vibrato amount

### **VOICE LEVELS (2 pots - Optional)**

21. **Main OSC Level** (0.0-1.0) - Can fade to silence
22. **Sub OSC Level** (0.0-1.0) - Bass thickness (currently hardcoded 30%)

### **Hardcoded (No Pots)**

- Main OSC waveform: Saw (richest harmonics)
- Sub OSC waveform: Square (classic bass)
- Sub OSC pitch: -1 octave (automatic)
- Filter LFO waveform: Sine (smooth sweeps)

---

## Phased Testing Strategy (12-Pot Limit → Expansion)

### **Phase 4A: Bass Generative Only (12 pots)**

**Goal**: Test if generative bass creates "simple, repetitive pattern" you want.

**Pot Allocation:**

**Core Performance (3 pots - NEVER reprogram):**
```rust
Pot 0 (GPIO1): Filter Cutoff     ← Essential for performance
Pot 1 (GPIO2): Filter Resonance  ← Essential for performance
Pot 2 (GPIO3): Filter LFO Depth  ← Already implemented
```

**Bass Generative Test Bed (9 pots - Reprogrammable):**
```rust
Pot 3 (GPIO4):  Bass Clock Rate   (0.5-4 Hz)
Pot 4 (GPIO5):  Bass Root Note    (30-300 Hz)
Pot 5 (GPIO6):  Bass Amount       (0-100%)
Pot 6 (GPIO7):  Bass Scale        (12 scales)
Pot 7 (GPIO8):  Bass Octave Range (1-4 octaves)
Pot 8 (GPIO9):  Filter LFO Rate   (0.01-10 Hz) ← Test if rate control needed
Pot 9 (GPIO10): <Spare>           ← Additional test parameter
Pot 10 (GPIO11): <Spare>          ← Additional test parameter
Pot 11 (GPIO12): <Spare>          ← Additional test parameter
```

**Hardcoded:**
- Main OSC: Saw waveform
- Sub OSC: Square -1 octave, 30% level (free bass thickness)
- Lead voice: OFF (not implemented yet)
- Delay: OFF (not implemented yet)
- Master Volume: 0.9 (90%, provides headroom)

**Implementation Tasks:**
1. [ ] Create `src/noise.rs` - WhiteNoise LFSR
2. [ ] Create `src/sample_hold.rs` - S&H with clock
3. [ ] Create `src/quantizer.rs` - 12 musical scales
4. [ ] Create `src/generative.rs` - GenerativeVoice wrapper
5. [ ] Add `bass_voice: GenerativeVoice` to Engine
6. [ ] Update `update_buffer_parameters()` - tick bass voice
7. [ ] Add bass messages to Message enum
8. [ ] Update controls/task.rs - wire 12 pots (6 new)
9. [ ] Simplify oscillators to 2 (Main saw + Sub square)

**What You Learn:**
- ✅ Does generative bass feel "simple and repetitive" enough?
- ✅ Which scales get used most (minor vs pentatonic vs blues)?
- ✅ What clock rates feel musical (1 Hz vs 4 Hz)?
- ✅ Is octave range control needed, or can it be fixed at 2?
- ✅ Is Filter LFO Rate pot needed, or can it be hardcoded at 1 Hz?
- ✅ Does Amount pot get used (blend mode), or is it binary 0%/100%?

**Timeline**: 1 weekend (8-12 hours coding + testing for 1 week)

---

### **Phase 4B: Lead Generative Only (12 pots, reprogram test bed)**

**Goal**: Test if fast generative creates "simple melodic pattern" for lead.

**Pot Allocation:**

**Core Performance (3 pots - UNCHANGED):**
```rust
Pot 0-2: Same as Phase 4A (cutoff, resonance, LFO depth)
```

**Lead Generative Test Bed (9 pots - REPROGRAMMED):**
```rust
Pot 3:  Lead Clock Rate    (4-20 Hz) ← Faster than bass
Pot 4:  Lead Root Note     (200-1024 Hz) ← Higher register
Pot 5:  Lead Amount        (0-100%)
Pot 6:  Lead Scale         (12 scales)
Pot 7:  Lead Octave Range  (1-4 octaves)
Pot 8:  Filter LFO Rate    (0.01-10 Hz)
Pot 9:  <Spare>
Pot 10: <Spare>
Pot 11: <Spare>
```

**Hardcoded:**
- Bass voice: OFF (switch to lead mode)
- Oscillators: Same (main saw + sub)
- Delay: OFF

**Implementation Tasks:**
1. [ ] Add `lead_voice: GenerativeVoice` to Engine
2. [ ] Route lead voice to oscillators
3. [ ] Update controls/task.rs - reprogram pots 6-11 to lead

**What You Learn:**
- ✅ Fast clock rates (8-16 Hz) = glitchy or musical?
- ✅ Lead feels different from bass (higher register matters)?
- ✅ Different scales for lead create interesting sounds?

**Timeline**: 4 hours (reprogram + test for 1 week)

---

### **Phase 4C: Dual Voice (Bass + Lead Simultaneously) - Needs 16+ Pots**

**Goal**: Test if simultaneous bass+lead is richer than single voice.

**Pot Allocation (MINIMUM 16 pots required):**

**Core Performance (4 pots):**
```rust
Pot 0: Filter Cutoff
Pot 1: Filter Resonance
Pot 2: Filter LFO Depth
Pot 3: Master Volume
```

**Bass Generative (4 pots):**
```rust
Pot 4: Bass Clock Rate
Pot 5: Bass Root Note
Pot 6: Bass Amount
Pot 7: Bass Scale
```

**Lead Generative (4 pots):**
```rust
Pot 8:  Lead Clock Rate
Pot 9:  Lead Root Note
Pot 10: Lead Amount
Pot 11: Lead Scale
```

**Extras (4 pots):**
```rust
Pot 12: Filter LFO Rate
Pot 13: Bass Octave Range
Pot 14: Lead Octave Range
Pot 15: <Spare>
```

**Implementation Tasks:**
1. [ ] Wire 4-6 more pots (second ADS1115 module)
2. [ ] Enable both bass_voice AND lead_voice simultaneously
3. [ ] Test polyrhythmic combinations (bass 2 Hz, lead 6 Hz)
4. [ ] Test polymodal (bass minor, lead major)

**What You Learn:**
- ✅ **CRITICAL**: Is simultaneous bass+lead essential, or is single voice + delay enough?
- ✅ Polyrhythmic patterns (2:3, 1:4) = interesting or overwhelming?
- ✅ Managing 2 independent generative voices = fun or too complex?

**Timeline**: Wait for AliExpress pots, then 1 day to wire + test

---

### **Phase 4D: Add Delay (Needs 18+ Pots)**

**Goal**: Add delay for pseudo-polyphonic texture and evolving echoes.

**Additional Pots (+2):**
```rust
Pot 16: Delay Time (50ms - 2 seconds)
Pot 17: Delay Feedback (0-110%, self-oscillates >100%)
```

**Optional (+1):**
```rust
Pot 18: Delay Mix (0-100% dry/wet)
```

**Implementation Tasks:**
1. [ ] Implement PSRAM delay buffer (ring buffer)
2. [ ] Add linear interpolation for variable delay time
3. [ ] Add feedback saturation (prevent runaway)
4. [ ] Wire 2-3 more pots for delay controls

**What You Learn:**
- ✅ Does delay make simple patterns more interesting?
- ✅ Delay feedback creates self-evolving textures?
- ✅ Is delay essential for "freshness," or just nice-to-have?

**Timeline**: 2-3 days (delay is complex - PSRAM buffer + interpolation)

---

### **Phase 4E: Full System (20-22 pots)**

**Goal**: All features active, test extensively for 2-4 weeks.

**Additional Features (+2-4 pots):**
- Pitch LFO (rate, depth) - vibrato/wobble on generative notes
- Main OSC Level - can fade to silence
- Sub OSC Level - control bass thickness
- Drive/Saturation - warmth/character

**What You Learn:**
- ✅ Which 10-12 pots get used 90% of the time?
- ✅ Are 4 oscillators better than 2 (main + sub)?
- ✅ Is pitch LFO essential or redundant?
- ✅ Can any features be combined or hardcoded?

**Output**: Final pot layout for pocket version (10-12 essential pots)

---

## Implementation Details

### **1. Simplified Oscillator Architecture (Phase 4A)**

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

### **2. White Noise Generator**

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

    /// Generate next noise sample (AUDIO RATE or BUFFER RATE).
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

### **3. Sample & Hold**

```rust
// src/sample_hold.rs

/// Sample and Hold circuit: samples input at clock rate.
/// Creates stepped random voltages (stairstep waveform).
pub struct SampleHold {
    held_value: f32,         // Currently held output
    clock_phase: f32,        // Internal clock [0.0, 1.0)
    clock_freq: f32,         // Clock frequency in Hz
    prev_clock_high: bool,   // Edge detection
}

impl SampleHold {
    pub fn new(clock_freq: f32) -> Self {
        Self {
            held_value: 0.0,
            clock_phase: 0.0,
            clock_freq,
            prev_clock_high: false,
        }
    }

    /// Process one sample (BUFFER RATE recommended for CPU savings).
    #[inline]
    pub fn tick(&mut self, input: f32) -> f32 {
        // Generate internal clock (square wave)
        let clock_high = self.clock_phase < 0.5;

        // Sample on rising edge (clock goes low → high)
        if clock_high && !self.prev_clock_high {
            self.held_value = input; // SAMPLE new value
        }
        self.prev_clock_high = clock_high;

        // Advance clock phase
        self.clock_phase += self.clock_freq / crate::config::SAMPLE_RATE as f32;
        if self.clock_phase >= 1.0 {
            self.clock_phase -= 1.0;
        }

        self.held_value // HOLD current value
    }

    pub fn set_clock_freq(&mut self, freq: f32) {
        self.clock_freq = freq.clamp(0.1, 20.0);
    }
}
```

**CPU cost**: ~10 cycles/sample

---

### **4. Scale Quantizer**

```rust
// src/quantizer.rs

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

        // Convert semitones to frequency: freq = root × 2^(semitones/12)
        let ratio = micromath::F32Ext::powf(2.0, closest as f32 / 12.0);
        self.root_freq * ratio
    }

    pub fn set_scale(&mut self, normalized: f32) {
        self.scale_index = (normalized * 11.99).floor() as usize;
    }

    pub fn set_root(&mut self, freq: f32) {
        self.root_freq = freq;
    }

    pub fn set_octave_range(&mut self, octaves: f32) {
        self.octave_range = octaves.clamp(1.0, 4.0);
    }
}
```

**CPU cost**: ~50 cycles (linear search + powf)

---

### **5. Generative Voice (Wrapper)**

```rust
// src/generative.rs

pub struct GenerativeVoice {
    noise: WhiteNoise,
    sample_hold: SampleHold,
    quantizer: Quantizer,
    amount: f32,           // 0.0 = manual, 1.0 = generative
    manual_pitch: f32,     // Fallback when amount < 1.0
}

impl GenerativeVoice {
    pub fn new(root_freq: f32, octave_range: f32, clock_freq: f32) -> Self {
        Self {
            noise: WhiteNoise::new(),
            sample_hold: SampleHold::new(clock_freq),
            quantizer: Quantizer::new(root_freq, octave_range),
            amount: 0.0, // Start with manual control
            manual_pitch: root_freq,
        }
    }

    /// Generate next pitch (BUFFER RATE ~172 Hz recommended).
    #[inline]
    pub fn tick(&mut self) -> f32 {
        let noise = self.noise.tick();
        let sh_value = self.sample_hold.tick(noise);
        let gen_pitch = self.quantizer.quantize(sh_value);

        // Blend generative + manual
        (gen_pitch * self.amount) + (self.manual_pitch * (1.0 - self.amount))
    }

    // Setters
    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
    }

    pub fn set_manual_pitch(&mut self, freq: f32) {
        self.manual_pitch = freq;
    }

    pub fn set_clock_freq(&mut self, freq: f32) {
        self.sample_hold.set_clock_freq(freq);
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

### **6. Engine Integration**

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
        // Initialize bass voice (30-300 Hz range, 2 Hz clock)
        let bass_voice = GenerativeVoice::new(
            60.0,  // Root: C2 (60 Hz)
            2.0,   // Octave range: 2 octaves
            2.0,   // Clock: 2 Hz (120 BPM 8th notes)
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

    /// BUFFER RATE (~172 Hz): Update generative voices.
    #[inline]
    fn update_buffer_parameters(&mut self) {
        // Generate bass pitch (controls both oscillators)
        let bass_pitch = self.bass_voice.tick();
        self.oscillators[0].set_frequency(bass_pitch);      // Main saw
        self.oscillators[1].set_frequency(bass_pitch / 2.0); // Sub (-1 octave)

        // Lead voice (Phase 4C+)
        if let Some(ref mut lead_voice) = self.lead_voice {
            let lead_pitch = lead_voice.tick();
            // Route to oscillators OR separate voice (TBD based on architecture)
        }

        // Update oscillator parameters (detune, level smoothing)
        for osc in &mut self.oscillators {
            osc.update_parameters();
        }

        // Filter LFO modulation (already implemented)
        let lfo_value = self.filter_lfo.tick();
        let modulated_cutoff = self.base_cutoff + (lfo_value * self.lfo_cutoff_range);
        self.filter.set_cutoff(modulated_cutoff);
        self.filter.update_coefficients();
    }
}
```

**Optimization**: Run generative tick() at **buffer rate** (172 Hz), not audio rate (44.1 kHz) → 256× CPU savings.

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
    SetBassClockRate(f32),      // 0.5-4 Hz
    SetBassScale(f32),          // 0.0-1.0 → 12 scales
    SetBassOctaveRange(f32),    // 1-4 octaves
    SetBassRoot(f32),           // 30-300 Hz
    SetBassAmount(f32),         // 0.0-1.0 (manual ↔ generative)

    // Lead generative (Phase 4B+)
    SetLeadClockRate(f32),      // 4-20 Hz
    SetLeadScale(f32),          // 0.0-1.0 → 12 scales
    SetLeadOctaveRange(f32),    // 1-4 octaves
    SetLeadRoot(f32),           // 200-1024 Hz
    SetLeadAmount(f32),         // 0.0-1.0

    // Delay (Phase 4D)
    SetDelayTime(f32),          // 50ms - 2s
    SetDelayFeedback(f32),      // 0-110%
    SetDelayMix(f32),           // 0-100%

    // Pitch modulation (Phase 4E)
    SetPitchLfoRate(f32),       // 0.01-10 Hz
    SetPitchLfoDepth(f32),      // 0.0-0.5 (±50¢)
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

/// Map pot to octave range (linear: 1-4 octaves).
pub fn map_octave_range(normalized: f32) -> f32 {
    1.0 + (normalized * 3.0)
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

**Per-voice cost (worst case):**
- WhiteNoise: ~5 cycles
- SampleHold: ~10 cycles
- Quantizer: ~50 cycles
- **Total per voice**: ~65 cycles

**Two voices at audio rate (44.1 kHz):**
- 130 cycles × 44,100 samples/sec = **5.73M cycles/sec**

**Optimization - Run at buffer rate (172 Hz):**
- 130 cycles × 172 buffers/sec = **22,360 cycles/sec**
- **256× reduction** in CPU usage

**Verdict**: Run generative voice tick() at **buffer rate**, pitch changes 172×/sec = imperceptible stepping.

---

## Musical Performance Examples

### **Deep Techno Bass**
```
Bass Clock: 2 Hz (120 BPM 8th notes)
Bass Scale: Minor pentatonic
Bass Range: 1 octave (stepwise motion)
Bass Amount: 100% (full generative)
Lead Amount: 0% (off)

YOU perform: Filter sweeps, resonance peaks

Result: Simple, repetitive bass line, YOU shape timbre
```

### **Aphex Twin Glitch**
```
Bass Clock: 4 Hz
Lead Clock: 12 Hz (3:1 polyrhythm)
Both Amount: 100%
Bass Scale: Harmonic minor
Lead Scale: Chromatic

YOU perform: Filter cutoff sweeps, resonance spikes, LFO depth

Result: Glitchy, polyrhythmic, raw and chaotic
```

### **Ambient Drone**
```
Bass Clock: 0.5 Hz (2 seconds/note)
Bass Scale: Whole tone
Bass Amount: 50% (blend with manual root pot)
Lead Amount: 0% (off)
Filter LFO Depth: 100% (slow evolving filter)

YOU perform: Filter cutoff sweeps, manual root note changes

Result: Slow evolution, guided randomness
```

### **Morphing Build-Up**
```
Start: Both Amount at 0% (silent)
10s:   Bass Amount 0→100% (bass enters)
30s:   Lead Amount 0→100% (lead joins)
1min:  Sweep filter cutoff open, crank resonance

Result: Live build-up performance, YOU control evolution
```

---

## Success Criteria

### **Phase 4A Complete When:**
- ✅ Bass generative creates simple, repetitive bass lines (always in key)
- ✅ All 12 scales tested and sound different
- ✅ Clock rate controls rhythm feel (0.5 Hz = slow, 4 Hz = fast)
- ✅ Amount pot blends manual ↔ generative smoothly
- ✅ Can perform filter sweeps on generative bass for 30+ minutes without boredom
- ✅ Understand if octave range pot is needed or can be fixed at 2

### **Phase 4B Complete When:**
- ✅ Lead generative creates simple melodic patterns
- ✅ Fast clock rates (8-16 Hz) create interesting textures
- ✅ Different scales for lead tested

### **Phase 4C Complete When:**
- ✅ **CRITICAL**: Understand if simultaneous bass+lead is essential or if single voice + delay is enough
- ✅ Polyrhythmic combinations tested (bass 2 Hz, lead 6 Hz)
- ✅ Can perform filter sweeps on dual generative voices

### **Phase 4D Complete When:**
- ✅ Delay creates pseudo-polyphonic texture
- ✅ Delay feedback self-oscillation tested (>100% feedback)
- ✅ Understand if delay is essential for "freshness"

### **Phase 4E Complete When:**
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
- Is Filter LFO Rate pot needed, or can it be hardcoded at 1 Hz?
- Does Amount pot get used (blend 0-100%), or is it always at 0% or 100%?

---

## Terminology & Context

**What this is called:**
- **Generative Synthesis** (algorithmic composition)
- **West Coast Synthesis** (Buchla tradition, 1960s+)
- **Random Voltage Generator** (modular synth term)
- **Quantized Random Sequencer** (describes the full chain)

**Artists using this:**
- Aphex Twin (Buchla Music Easel, custom MAX/MSP)
- Morton Subotnick (Silver Apples of the Moon, 1967)
- Suzanne Ciani (Buchla concerts, 1970s)
- Alessandro Cortini (Nine Inch Nails, Buchla/Serge)

**Commercial hardware:**
- Buchla Music Easel ($4000+) - has built-in random voltage + quantizer
- Make Noise 0-Coast ($500) - West Coast semi-modular
- Moog Subharmonicon ($700) - polyrhythmic sequencer

**Your synth is unique**: Pocket-portable, all analog pots, dual generative voices, no MIDI, no screens. Nothing else like this exists.

---

This is going to be incredible. Let's build it. 🎛️
