# ESP32-S3 Synth MVP

## Goals
- Build a **standalone drone/experimental synth** on ESP32-S3 + PCM5102A DAC.
- Input via **voice-selection interface**: 3 voice buttons + rotary encoder for frequency control.
- Core sound: **3 continuously-running sine oscillators** with volume and active state control.
- Tone shaping: **Low-pass and High-pass filters** (shared across all voices) - *future phase*.
- Effects: **Drive, Delay, Reverb** - *future phase*.
- Output: **Stereo line-level** via PCM5102A (mono duplicated to both channels).
- Visualizer: **Oscilloscope-style** on CRT (via Pmod VGA → VGA-to-Composite converter) - *future phase*.

## Philosophy
- **Drone-focused**: voices sustain indefinitely, allowing hands-free sound sculpting.
- **Continuous frequency control**: rotary encoder provides smooth, unlimited frequency sweep (no fixed notes).
- **Sound design over performance**: optimized for exploring timbres, harmonics, and textures rather than playing melodies.
- **West Coast synthesis approach**: embrace continuous parameter control and experimental tuning.

## Non-Goals (for initial MVP)
- Traditional keyboard with fixed notes.
- USB MIDI host/device.
- Additional oscillator waveforms (square/triangle/saw).
- Envelopes (Attack/Release/Decay/Sustain) - deferred to later phase.
- Spectrum/special visualizations.
- Advanced FX (chorus, flanger, etc.).
- Filters and effects chain - building core audio engine first.

---

## Hardware

- **ESP32-S3 dev board** (with I²S for DAC).
- **PCM5102A DAC** (stereo line/headphone out).
- **Voice Buttons**: 3 × tactile switches (select which voice to edit).
- **Voice LEDs**: 3 × LEDs (indicate currently selected voice).
- **Encoders**: 2 × rotary encoders (frequency control, volume control for selected voice).
- **Pots**: 10 kΩ linear (×6 for cutoff, resonance, drive, delay time, delay mix, reverb mix).
- **Mode Buttons**: Octave −, Octave + (shift frequency range), Shift (future: mode switching).
- **Visualizer path**: Pmod VGA → VGA-to-Composite active converter → CRT TV.

---

## Voice Control Input

### Voice Selection
- **3 voice buttons** with corresponding LEDs.
- Press button → selects that voice for editing, LED indicates selection.
- Press same button again → toggles voice on/off (LED blinks or dims when off?).
- Voice continues playing at its set frequency/volume until toggled off.

### Frequency & Volume Control
- **Frequency encoder**: Controls frequency of currently selected voice.
  - Clockwise → frequency increases, counter-clockwise → decreases.
  - Range: 20 Hz – 2000 Hz (adjustable via Octave +/− buttons).
  - **Velocity-sensitive**: slow rotation = fine (±1-5 Hz), fast = coarse (±50-100 Hz).
- **Volume encoder**: Controls volume of currently selected voice (0.0 to 1.0).
  - Clockwise → louder, counter-clockwise → quieter.

### Frequency Range Shifting
- **Octave −** button: shifts frequency range down (e.g., 20–200 Hz sub-bass).
- **Octave +** button: shifts frequency range up (e.g., 200–2000 Hz mid-range).

### Debouncing
- **Buttons**: 10 ms software debounce.
- **Encoder**: hardware debounced via ESP32 PCNT (pulse counter) peripheral.

---

## Sound Engine

### Voices
- **3 independent voices**, each with:
  - Sine wave oscillator (wavetable-based).
  - Frequency & volume controlled by encoders (when that voice is selected).
  - Active state (toggled by voice button) - when inactive, voice outputs silence.
  - Volume setting persists when toggling active state on/off.

### Filters
- **Low-pass filter** (12 dB/oct, shared across all voices).
- **High-pass filter** (12 dB/oct, shared across all voices).
- Filters applied to the mixed voice signal.

### Effects
- **Drive**: waveshaper (soft clip/tanh), applied post-filter.
- **Delay**: mono, single tap, with feedback + mix controls.
- **Reverb**: Schroeder/Moorer style (4 combs + 2 all-pass), parameters: Mix, Time.

### Signal Flow (Current MVP)
```
Voice 1 Osc → Volume ┐
Voice 2 Osc → Volume ├→ Mix (normalize by 3) → PCM5102A
Voice 3 Osc → Volume ┘
```

### Signal Flow (Future)
```
Voice 1 Osc → Envelope → Volume ┐
Voice 2 Osc → Envelope → Volume ├→ Mix → LPF → HPF → Drive → Delay → Reverb → PCM5102A
Voice 3 Osc → Envelope → Volume ┘
```

---

## Controls

### Rotary Encoders
- **Frequency encoder**: Continuous, velocity-sensitive frequency adjustment for selected voice.
- **Volume encoder**: Continuous volume adjustment for selected voice (0.0 to 1.0).

### Pots (10 kΩ linear)
1. **Cutoff** (low-pass filter frequency).
2. **Resonance** (filter emphasis at cutoff frequency).
3. **Drive** (saturation/distortion amount).
4. **Delay Time** (delay tap length, ~50ms – 1000ms).
5. **Delay Mix** (dry/wet blend for delay).
6. **Reverb Mix** (dry/wet blend for reverb).

### Buttons
- **Voice 1 / Voice 2 / Voice 3**: Select voice, toggle on/off.
- **Octave −**: Shift frequency range down (sub-bass mode).
- **Octave +**: Shift frequency range up (mid/high mode).
- **Shift**: Reserved for future mode switching (e.g., keyboard mode, preset recall).

---

## Visualizer (Scope)
- Output via **Pmod VGA → Composite converter → CRT**.
- **Oscilloscope-style only** (waveform trace of mixed output).
- Aim for CRT aesthetic: persistence glow, soft edges.
- Shows the sum of all active voices after filtering and effects.

---

## Acceptance Checklist (Initial MVP)

1. **Voice Selection**: Press voice button → LED indicates selected voice.
2. **Voice Toggle**: Press selected voice button again → voice turns on/off (active state changes).
3. **Frequency Control**: Rotate frequency encoder → selected voice's frequency changes in real-time.
4. **Volume Control**: Rotate volume encoder → selected voice's volume changes in real-time.
5. **Polyphony**: All 3 voices playing simultaneously with different frequencies/volumes → audible harmony/chord.
6. **Audio Stability**: PCM5102A outputs clean 44.1 kHz stereo, no clicks/pops/dropouts.
7. **Volume Normalization**: 3 voices at full volume don't clip (sum divided by 3).
8. **Inactive Voices Silent**: Toggling voice off produces silence, toggling back on resumes at same frequency/volume.

## Future Acceptance Tests (Post-MVP)
9. **Frequency Range**: Press Octave +/− → frequency range shifts (sub-bass vs mid-range).
10. **Filters**: Cutoff/Resonance pots → smooth filter sweeps, audible timbral change.
11. **Drive**: Drive pot → saturation/grit increases.
12. **Delay**: Delay Time/Mix pots → rhythmic echo effect with feedback.
13. **Reverb**: Reverb Mix pot → spatial depth increases.
14. **No Zipper Noise**: Encoder/pot adjustments are smooth (parameter smoothing implemented).
15. **Envelopes**: Voice toggle produces smooth attack/release (no clicks/pops).
16. **Visualizer**: Oscilloscope displays stable waveform trace on CRT.

---

## Workflow Example

### Creating a 3-Voice Drone Chord:
1. Press **Voice 1** button → LED 1 lights up (voice selected).
2. Rotate **frequency encoder** to ~220 Hz, adjust **volume encoder** to taste.
3. Press **Voice 1** button again → voice becomes active and plays.
4. Press **Voice 2** button → LED 2 lights up (now selected).
5. Rotate **frequency encoder** to ~440 Hz, adjust **volume encoder** louder (fundamental).
6. Press **Voice 2** button again → voice becomes active.
7. Press **Voice 3** button → LED 3 lights up.
8. Rotate **frequency encoder** to ~660 Hz, adjust **volume encoder** softer (fifth interval).
9. Press **Voice 3** button again → voice becomes active.
10. Now all 3 voices drone together (A-A-E chord with balanced mix).
11. Press **Voice 2** button → select it, press again → middle voice turns off (silenced).
12. Rotate encoders → set new frequency/volume (e.g., 550 Hz, C#).
13. Press **Voice 2** button twice → new note plays (select then activate).

### Future: Sound Sculpting (Post-MVP with FX)
- Sweep **Cutoff** pot → filter opens/closes.
- Increase **Resonance** → emphasize filter peak.
- Add **Drive** → harmonic saturation.
- Tweak **Delay Mix** → rhythmic texture.
- Raise **Reverb Mix** → ambient space.

---

## Architecture Notes

### Message Passing (Lock-Free)
- **Audio task** (single task for MVP): owns I2S peripheral, receives messages via `heapless::spsc::Queue`.
- **Future control tasks** (buttons, encoders, ADC) will send messages to audio task.
- **Audio callback**: drains all pending messages, updates `Engine` state, renders audio buffer.
- **Why**: Real-time safety (no blocking), clean separation of control/audio paths.

### Module Structure
```
src/
  config.rs        // Centralized configuration constants
  oscillator.rs    // Wavetable oscillator (phase accumulator)
  voice.rs         // Voice = Oscillator + volume + active state
  engine.rs        // Engine = 3 voices + selected voice + (later: filters/FX)
  message.rs       // Message enum for lock-free communication
  lib.rs           // Re-exports
```

### Key Structs
```rust
// Oscillator: pure signal generation (already exists)
struct Oscillator {
    phase: f32,
    phase_increment: f32,
    wavetable: &'static [f32],
}

// Voice: instrument instance (current MVP, no envelope yet)
struct Voice {
    osc: Oscillator,
    volume: f32,              // User-set volume (0.0 to 1.0)
    active: bool,             // Voice on/off state
}

// Engine: synth state + rendering
struct Engine {
    voices: [Voice; VOICE_COUNT],
    selected_voice: Option<u8>,  // Some(0/1/2) or None (nothing selected)
    sample_rate: f32,
    // Later: filter state, effects state
}

// Message: control → audio communication
enum Message {
    SelectVoice(u8),
    ToggleVoice(u8),
    SetFrequency(f32),        // For selected voice
    SetVolume(f32),           // For selected voice
    // Later: SetCutoff, SetResonance, etc.
}
```

---

## Future Expansions

### Phase 1 (Post-MVP - Core Audio Features)
- **Envelopes**: Add Attack/Release (50ms/300ms) to eliminate clicks when toggling voices.
- **Filters**: Low-pass and high-pass (12 dB/oct, shared across voices).
- **Basic Effects**: Drive (waveshaper), Delay (mono single-tap), Reverb (Schroeder/Moorer).
- **Octave buttons**: Shift frequency range (sub-bass vs mid-range modes).

### Phase 2 (Extended Features)
- **Per-voice detuning**: Fine-tune each voice independently for chorus effect.
- **Frequency quantization mode**: Snap encoder to chromatic scale notes.
- **Delay feedback pot**: Control delay regeneration.
- **Reverb time pot**: Adjust reverb decay length.
- **Visualizer**: Oscilloscope on CRT via Pmod VGA.

### Phase 3 (Major Features)
- **Keyboard mode** (Shift button): Add 13-button keyboard (C→C octave) with traditional note-on/off.
- **Additional waveforms**: Square, triangle, sawtooth oscillators.
- **USB MIDI**: External keyboard input.
- **Preset system**: Save/recall voice + FX configurations.

### Phase 4 (Advanced)
- **More effects**: Chorus, flanger, phaser.
- **Spectrum analyzer** or Lissajous visualizer.
- **LFO modulation**: Auto-sweep cutoff, frequency, or delay time.
- **Envelope followers**: React to audio input (if adding mic/line-in).

---

## Design Philosophy Summary

This MVP prioritizes **exploration and experimentation** over traditional musicianship:
- **No "wrong notes"** – just frequencies and harmonics.
- **Hands-free operation** – set up a drone, then sculpt with both hands on pots.
- **Immediate feedback** – every encoder turn and pot twist is instantly audible.
- **Minimal hardware complexity** – no keyboard matrix, fewer buttons/wiring.
- **Foundation for expansion** – architecture supports adding keyboard mode, MIDI, presets later.

**Goal**: Build the simplest functional drone synth first, then expand once the core audio engine is solid.
