# ESP32-S3 Synth MVP

## Goals
- Build a **standalone playable synth** on ESP32-S3 + PCM5102A DAC.
- Input via **DIY one-octave keyboard (C→C)** built from buttons.
- Core sound: **polyphonic sine oscillators with ADSR envelopes**.
- Tone shaping: **Low-pass and High-pass filters**.
- Effects: **Drive, Delay, Reverb** (mandatory).
- Output: **Stereo line-level** via PCM5102A (no panning yet).
- Visualizer: **Oscilloscope-style** on CRT (via Pmod VGA → VGA-to-Composite converter).

## Non-Goals
- USB MIDI host/device (for now).
- Additional oscillator waveforms (square/triangle/saw).
- Spectrum/special visualizations.
- Advanced FX (chorus, flanger, etc.).

---

## Hardware

- **ESP32-S3 dev board** (with I²S, USB-OTG if needed later).
- **PCM5102A DAC** (stereo line/headphone out).
- **Keys**: 13 × tactile switches (arranged physically as piano-style octave, white + black keys).
- **Diodes**: 13 × 1N4148 (matrix ghosting protection).
- **Expander**: MCP23017 (I²C → 2 pins for the key matrix).
- **Pots**: 10 kΩ linear (x7 for cutoff, resonance, env amt, drive, delay time, delay mix, reverb mix).
- **Buttons**: Octave −, Octave +, Shift.
- **Visualizer path**: Pmod VGA → VGA-to-Composite active converter → CRT TV.

---

## Keyboard Input

- **Layout**: 13 switches, arranged C4–C5 including sharps/flats.
- **Matrix wiring**: MCP23017 expands I/O → fewer ESP32 pins used.
- **Debounce**: 5–10 ms software debounce; scan rate 1–2 kHz.
- **Diodes**: one per switch to prevent ghosting.
- **Events**: NoteOn/NoteOff emitted internally (MIDI-like struct).

---

## Sound Engine

### Oscillators
- **3-voice polyphony** (configurable).
- **Sine wave only** (expandable later).
- Per-voice **ADSR envelopes**.

### Filters
- **Low-pass filter** (12 dB/oct).
- **High-pass filter** (12 dB/oct).

### Effects
- **Drive**: waveshaper (soft clip/tanh).
- **Delay**: mono, single tap, feedback + mix.
- **Reverb**: Schroeder/Moorer style (4 combs + 2 all-pass), parameters: Mix, Time.

### Signal Flow
Voice mix → LPF → HPF → Drive → Delay → Reverb → PCM5102A.

---

## Controls

- **Pots (10 kΩ linear)**:  
  1. Cutoff  
  2. Resonance  
  3. Env Amount  
  4. Drive  
  5. Delay Time  
  6. Delay Mix  
  7. Reverb Mix  

- **Buttons**:  
  - Octave −  
  - Octave +  
  - Shift (secondary functions: Delay Feedback, Reverb Time/Damping, etc.)

---

## Visualizer (Scope)
- Output via **Pmod VGA → Composite converter → CRT**.
- **Oscilloscope-style only** (waveform trace).
- Aim for CRT aesthetic: persistence glow, soft edges.

---

## Acceptance Checklist

1. **Keys**: Press C→C → NoteOn/Off events trigger.  
2. **Polyphony**: 3 voices; proper voice allocation.  
3. **ADSR**: Envelope shaping audible.  
4. **Filters**: LPF + HPF respond smoothly.  
5. **FX**: Drive adds grit; Delay/Feedback/Mix work; Reverb Mix/Time audible.  
6. **Controls**: Pots + buttons mapped correctly; no zipper noise.  
7. **Audio**: PCM5102A stable 48 kHz out, stereo duplicated.  
8. **Visualizer**: Oscilloscope waveform stable on CRT.  

---

## Future Expansions
- Add waveforms (square, triangle, saw).
- USB MIDI (host or device).
- More FX (chorus, flanger, phaser).
- Spectrum analyzer or Lissajous visualizer.
- Larger keyboard or external MIDI keyboard.