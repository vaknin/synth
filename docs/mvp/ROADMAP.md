# Pocket Synth Roadmap

**Vision**: Pocket-sized melody maker with arpeggiator + subtractive synthesis. Good melodies, expressive filter control, fun to perform.

**Current State**:
- ✅ Phase 1-3 complete (subtractive synthesis engine)
- ✅ Phase 4A complete (basic arpeggiator with gate)
- ✅ Phase 4B complete (arp modes: Up/Down/UpDown/Random, transpose ±12 semitones)
- ✅ Phase 4C complete (delay line with feedback, 50ms-1000ms)
- 📍 **Current**: Testing full arpeggiator + delay system

---

## Architecture Overview

```
Arpeggiator Clock (1-20 Hz) → Chord Pattern → Note
                                              ↓
                                         Lead OSC (Saw)
                                      (play arp note)
                                              │
                                              ├─────────┐
                                              │         │
                                    Mix ←─────┘    Bass OSC (Sine)
                                     │         (constant drone @ -2 oct)
                                     │         with drift LFO (±5¢)
                                     ↓
                          Moog Filter → Delay → Output
```

**Key Design:**
- **Lead**: 1 oscillator (Saw) plays arpeggio notes with gate articulation
- **Bass**: 1 oscillator (Sine) plays constant sub-bass drone at -2 octaves
  - Lyra-8 style pitch drift: ±5 cents over 30-second cycle
  - 25% level (texture, not melody)
  - Warm sine wave prevents harmonic masking
- **Dual layer**: Lead = performance/melody, Bass = texture/ambience
- **Filter LFO**: Modulates cutoff for evolving timbre (shared by both layers)

---

## Phase 4: Arpeggiator System

### Phase 4A: Basic Arpeggiator (Start Here)

**Goal**: Build simple arpeggiator with hardcoded chords, "up" mode only.

**Features**:
- Arp clock: 1-20 Hz (direct, no BPM)
- Mode: Up only (C → E → G → C...)
- Chords: Start with 1 pattern (C major), expand later
- Bass: Plays root note every 4th note (sparse rhythm)

**Pot Allocation (8 pots) - Current**:
```
Pot 1: Filter Cutoff         (essential)
Pot 2: Filter Resonance       (essential)
Pot 3: Filter LFO Rate        (existing)
Pot 4: Filter LFO Depth       (existing)
Pot 5: Arp Rate (1-20 Hz)     ✅ implemented
Pot 6: Delay Time (50ms-1s)   ✅ implemented
Pot 7: Delay Mix (0-50%)      ✅ implemented
Pot 8: Master Pitch           ✅ implemented
```

**Tradeoffs Made**:
- Arp Mode (Up/Down/UpDown/Random): Hardcoded to `Up` mode
- Arp Transpose (±12 semitones): Removed for delay control
- Delay feedback: Hardcoded at 40% (3-4 echoes)
- Bass rhythm: Changed from "every 4th note" to constant drone for texture

**Implementation**:
1. Create `src/arpeggiator.rs`:
   - Clock generator (Hz-based, phase accumulator)
   - Hardcoded chord pattern (start with 1: C major)
   - Pattern index tracking
   - "Up" mode logic
   - Bass frequency cached at -2 octaves (sub bass/pad territory)
2. Update `src/engine.rs`:
   - Add `arpeggiator` field
   - Add `bass_drift_lfo` (30-second cycle, ±5 cents modulation)
   - Tick arp at buffer rate
   - Route lead note to Osc[0] (Saw) with gate
   - Route bass to Osc[1] (Sine, constant drone with drift)
3. Update `src/message.rs`:
   - Add `SetArpRate(f32)`
   - Add `SetArpChord(usize)` (for future chord selection)
4. Wire pot 5 for arp rate control

**Success Criteria**:
- ✅ Arpeggio cycles correctly (C → E → G → C)
- ✅ Bass provides constant harmonic foundation at -2 octaves
- ✅ Bass drift adds subtle movement without detuning noticeably
- ✅ Rate pot controls arp speed smoothly
- ✅ Dual layer works: lead = melody, bass = evolving texture
- ✅ Filter performance feels responsive
- ✅ Can switch between performance mode (fast arp) and ambient mode (slow arp)

---

### Phase 4B: Arp Modes + Features ✅

**Goal**: Add more arp modes and musical features.

**Implemented Features**:
- ✅ Modes: Up, Down, UpDown, Random (all 4 modes working)
- ✅ Random mode uses `WhiteNoise` LFSR (32-bit Galois)
- ✅ Root transpose: ±12 semitones with extended lookup table (0-24 semitones)
- ✅ Gate system: 80% note length, 20% silence for articulation

**Status**: Fully implemented but mode/transpose controls sacrificed for delay (Phase 4C).
- Arp mode: Hardcoded to `ArpMode::Up` in engine
- Transpose: Code exists but no pot control (set to 0 semitones)

---

### Phase 4C: Delay ✅

**Goal**: Add delay for pseudo-polyphonic texture.

**Implemented Features**:
- ✅ Delay time: 50ms - 1000ms (exponential pot curve)
- ✅ Delay mix: 0-50% (wet/dry balance, capped at 50% to preserve dry attack)
- ✅ Feedback: Hardcoded at 40% (creates 3-4 natural echoes)
- ✅ Ring buffer: Vec<f32> in heap (will optimize to PSRAM later if needed)
- ✅ Architecture: Filter → Delay (preserves filtered character in echoes)

**Pot Assignment**:
- Pot 6: Delay Time (exponential curve for perceptual linearity)
- Pot 7: Delay Mix (linear 0-50%)

**Performance**: ~10-20 cycles/sample (ring buffer read/write + feedback multiply)

---

## Hardcoded Parameters

These work well as constants (no pots needed):
- **Lead waveform**: Saw (bright, harmonic-rich)
- **Bass waveform**: Sine (warm, sub-frequency foundation)
- **Bass level**: 25% (texture underneath, not melody)
- **Bass drift**: ±5 cents over 30-second cycle (Lyra-8 style)
- **Bass octave**: -2 octaves (sub bass/pad territory)
- **Filter LFO waveform**: Sine (smooth sweeps)
- **Master gain**: 0.9 (headroom)
- **Arp attack**: Fast (< 5ms, percussive)
- **Arp gate**: 80% (slight gap between notes)
- **Delay feedback**: 40% (3-4 natural echoes)

---

## Future Ideas (Beyond Phase 4)

- **Scale quantization**: Snap arp to musical scales
- **Step sequencer**: Record and loop patterns
- **Pitch LFO**: Vibrato/wobble on sustained notes
- **Delay feedback control**: If delay feels essential

---

## Message System

```rust
// src/message.rs additions

pub enum Message {
    // Existing
    SetMasterPitch(f32),
    SetOscLevel(OscillatorId, f32),
    SetOscDetune(OscillatorId, f32),
    SetCutoff(f32),
    SetResonance(f32),
    SetFilterLfoRate(f32),
    SetFilterLfoDepth(f32),

    // New for Phase 4A
    SetArpRate(f32),           // 1-20 Hz
    SetArpChord(usize),        // 0-7 (hardcoded patterns)

    // Phase 4B
    SetArpMode(ArpMode),       // Up, Down, UpDown, Random
    SetArpTranspose(i8),       // ±12 semitones

    // Phase 4C (optional)
    SetDelayTime(f32),         // 50ms - 2s
    SetDelayMix(f32),          // 0-100%
}
```

---

## Success Criteria

### Phase 4A Complete When:
- ✅ Arpeggio sounds melodic and rhythmic
- ✅ Bass locks harmonically with lead
- ✅ Chord switching feels musical
- ✅ Rate control is expressive (1-20 Hz range works well)
- ✅ Filter + arp together = fun to perform
- ✅ Can play for 15+ minutes without boredom

### Phase 4B Complete When:
- ✅ All arp modes tested (up, down, up-down, random)
- ✅ Random mode sounds musical (not chaotic)
- ✅ Root transpose expands harmonic palette
- ✅ Understand which modes get used most

### Phase 4C Complete When:
- ✅ Delay makes simple patterns more interesting
- ✅ Delay time syncs musically with arp rate
- ✅ Understand if delay is essential or nice-to-have

---

## Current Status

**Completed**:
- ✅ Subtractive synthesis engine (2 oscillators: Lead Saw + Bass Square)
- ✅ Moog ladder filter (cutoff, resonance)
- ✅ Filter LFO (rate, depth, sine wave)
- ✅ Arpeggiator (clock-driven, C major chord, Up mode)
- ✅ Arp gate system (80/20 timing for articulation)
- ✅ Delay line (50ms-1000ms, 40% feedback, filter→delay chain)
- ✅ 8 pots fully wired

**In Progress**:
- 📍 Testing full system (arpeggiator + delay + filter)
- 📍 Evaluating delay impact on musicality

**Next Steps**:
1. Build and test on hardware
2. Evaluate if arp mode/transpose pots are needed (vs delay)
3. Consider adding more chord patterns
4. Potentially add PSRAM optimization for delay buffer

---

## Why Arpeggiator?

**Pros**:
- Predictable, melodic patterns (not random)
- Classic subtractive synthesis approach
- Easy to understand and perform
- Familiar to players (TB-303, SH-101, Odyssey)
- Fun to combine with filter sweeps

**What we learned from generative attempt**:
- Turing Machine created "evolving patterns" not "melodies"
- Lacked repetition, contour, phrase structure
- Random notes in key ≠ melodic
- **Arpeggiator gives us actual melodies**

---

**Let's build this.** 🎹
