# Pocket Synth Roadmap

**Vision**: Pocket-sized melody maker with arpeggiator + subtractive synthesis. Good melodies, expressive filter control, fun to perform.

**Current State**:
- ✅ Phase 1-3 complete (4 oscillators, Moog filter, filter LFO)
- ✅ 8 pots wired and working
- 📍 **Next**: Build arpeggiator system

---

## Architecture Overview

```
Arpeggiator Clock (1-20 Hz) → Chord Pattern → Note
                                              ↓
                                    ┌─────────┴─────────┐
                                    │                   │
                              Lead OSCs             Bass OSC
                          (play arp note)      (play root note)
                                    │                   │
                                    └─────────┬─────────┘
                                              ↓
                                    Mix → Moog Filter → Output
```

**Key Design:**
- **Lead**: 1 oscillator (Osc2 Saw) plays arpeggio notes (cycles through chord)
- **Bass**: 1 oscillator (Sub Square) plays root note every 4th note (sparse rhythm)
- **Same clock**: Both use arp clock, harmonically locked
- **Filter LFO**: Already working, keep it for evolving timbre

---

## Phase 4: Arpeggiator System

### Phase 4A: Basic Arpeggiator (Start Here)

**Goal**: Build simple arpeggiator with hardcoded chords, "up" mode only.

**Features**:
- Arp clock: 1-20 Hz (direct, no BPM)
- Mode: Up only (C → E → G → C...)
- Chords: Start with 1 pattern (C major), expand later
- Bass: Plays root note every 4th note (sparse rhythm)

**Pot Allocation (8 pots)**:
```
Pot 0: Filter Cutoff         (essential)
Pot 1: Filter Resonance       (essential)
Pot 2: Filter LFO Depth       (existing)
Pot 3: Filter LFO Rate        (existing)
Pot 4: Arp Rate (1-20 Hz)     (new)
Pot 5: <Spare>                (future: chord select when multiple chords added)
Pot 6: <Spare>                (future: arp mode, root note transpose)
Pot 7: <Spare>                (future: delay time, delay mix)
```

**Implementation**:
1. Create `src/arpeggiator.rs`:
   - Clock generator (Hz-based, phase accumulator)
   - Hardcoded chord pattern (start with 1: C major)
   - Pattern index tracking
   - "Up" mode logic
   - Bass trigger (every 4th note, derived from chord length)
2. Update `src/engine.rs`:
   - Add `arpeggiator` field
   - Tick arp at buffer rate
   - Route lead note to Osc2 (Saw)
   - Route bass note to Sub (Square, conditional on bass_active)
3. Update `src/message.rs`:
   - Add `SetArpRate(f32)`
   - Add `SetArpChord(usize)` (for future chord selection)
4. Wire pot 4 for arp rate control

**Success Criteria**:
- ✅ Arpeggio cycles correctly (C → E → G → C)
- ✅ Bass plays root note every 4th note (sparse, grooving rhythm)
- ✅ Rate pot controls speed smoothly
- ✅ Chord selection switches patterns (future)
- ✅ Filter performance feels responsive

---

### Phase 4B: Arp Modes + Features

**Goal**: Add more arp modes and musical features.

**New Features**:
- Modes: Up, Down, Up-Down, Random
- Root transpose: Shift pitch up/down
- Octave range: 1-3 octaves

**Pot Reallocation**:
```
Pot 6: Arp Mode (4 modes)     (repurpose spare)
Pot 7: Root Transpose         (repurpose spare, ±12 semitones)
```

**Implementation**:
- Add mode enum to arpeggiator
- Add random mode (use `WhiteNoise` LFSR)
- Add root transpose logic
- Test all modes extensively

---

### Phase 4C: Delay (Optional)

**Goal**: Add delay for pseudo-polyphonic texture.

**Features**:
- Delay time: 50ms - 2s
- Delay mix: 0-100%
- Feedback: Hardcoded at 40% (test if pot needed)

**Requires**: More pots (currently 8, would need 10)

**Implementation**:
- PSRAM ring buffer
- Linear interpolation for time changes
- Feedback saturation (prevent runaway)

---

## Hardcoded Parameters

These work well as constants (no pots needed):
- Main OSC waveform: Saw (Osc1), Saw (Osc2), Square (Osc3)
- Main OSC levels: Equal mix (will normalize)
- Sub OSC: -1 octave below lead, plays bass root
- Filter LFO waveform: Sine (smooth sweeps)
- Master gain: 0.9 (headroom)
- Arp attack: Fast (< 5ms, percussive)
- Arp gate: 80% (slight gap between notes)

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
- ✅ Subtractive synthesis engine (4 oscillators)
- ✅ Moog ladder filter (cutoff, resonance)
- ✅ Filter LFO (rate, depth)
- ✅ 8 pots wired and polling

**In Progress**:
- 📍 Salvaging WhiteNoise from euclidean branch (for random arp mode)

**Next Steps**:
1. Design arpeggiator architecture (hardcoded chords)
2. Implement basic "up" mode
3. Wire pots 4-5 for arp control
4. Test and iterate

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
