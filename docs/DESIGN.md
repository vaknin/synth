# ESP32-S3 Drone Synth

## Philosophy

**West Coast-inspired experimental drone synthesizer** focused on sound exploration over traditional performance.

- **Drone-first**: Voices sustain indefinitely, hands-free sound sculpting
- **Continuous control**: Smooth frequency sweeps, no fixed notes
- **Sound design over melody**: Exploring timbres, harmonics, and evolving textures
- **Polyphonic**: 3 independent voices for harmonic interaction

---

## Hardware

- **ESP32-S3** + **PCM5102A DAC** (44.1 kHz stereo output)
- **Voice Control**: 3 buttons + LEDs (select/toggle voices)
- **Encoders**: 2× rotary (frequency, volume per-voice)
- **Pots**: 10 kΩ linear (cutoff, resonance, drive, delay time, delay mix, reverb mix, LFO rates/depths)
- **Mode Buttons**: Octave +/−, Shift (future features)
- **Visualizer** (future): Oscilloscope on CRT via Pmod VGA → composite converter

---

## Sound Engine

### Voices
- **3 polyphonic voices**, each with:
  - Sine wave oscillator (wavetable)
  - Independent frequency (20–2000 Hz, continuous)
  - Independent volume (0.0–1.0)
  - Active state (on/off toggle, retains settings when off)
  - Mixed and normalized to prevent clipping

### LFOs (Global Modulation)
**Architecture**: Global LFOs modulate all voices together (unified texture evolution, not chaos).

**Phase 1 - Essential LFOs** (implement first):
1. **Filter Cutoff LFO** (MUST-HAVE)
   - Creates auto-wah, breathing textures
   - Most dramatic timbral change
   - Controls: Rate (0.1–10 Hz), Depth (modulation amount)

2. **Pitch LFO / Vibrato** (MUST-HAVE)
   - Adds organic movement, prevents static drones
   - Slow = chorus shimmer, medium = vibrato, fast = FM-like timbral shift
   - Controls: Rate (0.5–15 Hz), Depth (cents/semitones)

**Phase 2 - Advanced LFOs** (add later):
3. **Resonance LFO**
   - Works with filter cutoff for wildly evolving harmonics
   - High resonance + low cutoff = filter self-oscillation (extra tone source)
   - Controls: Rate, Depth

4. **Wavefolder Depth LFO**
   - Rhythmic harmonic bursts (very West Coast!)
   - More aggressive than filter modulation
   - Controls: Rate, Depth

### Wavefolding
**What**: Mirrors waveform peaks back on themselves (unlike harsh clipping).
**Why**: Transforms sine waves into rich, bell-like/buzzy timbres that stay musical.
**Sound**: Low fold = brighter sine, medium = clarinet-like, high = metallic/bells.
**Classic West Coast technique** from Buchla 259 oscillator.

### Filters
- **Low-pass filter** (12 dB/oct, shared across voices)
- **High-pass filter** (12 dB/oct, shared across voices)
- Cutoff and resonance controls
- Applied to mixed voice signal

### Effects
- **Drive**: Waveshaper/soft clipping (post-filter harmonic saturation)
- **Delay**: Mono single-tap with feedback + mix
- **Reverb**: Schroeder/Moorer style (4 combs + 2 all-pass), mix + time controls

### Signal Flow
```
Voice 1 Osc ┐
Voice 2 Osc ├─→ Mix (÷3) ─→ Wavefolder ─→ LPF/HPF ─→ Drive ─→ Delay ─→ Reverb ─→ Output
Voice 3 Osc ┘                    ↑              ↑
                              LFO 4          LFO 1/3
                          (Fold Depth)    (Cutoff/Reso)

                          LFO 2 (Pitch) ──→ All Voices
```

---

## Controls

### Encoders
- **Frequency**: Adjust selected voice's pitch (velocity-sensitive: slow = fine, fast = coarse)
- **Volume**: Adjust selected voice's level (0.0–1.0)

### Voice Buttons
- **Press once**: Select voice (LED lights)
- **Press again**: Toggle on/off (active state)

### Pots (10 kΩ linear)
**Filter & Drive**:
1. Cutoff (LPF frequency)
2. Resonance (filter emphasis)
3. Drive (saturation amount)

**Effects**:
4. Delay Time (~50–1000 ms)
5. Delay Mix (dry/wet)
6. Reverb Mix (dry/wet)

**LFOs** (Phase 1):
7. LFO 1 Rate (Filter Cutoff)
8. LFO 1 Depth
9. LFO 2 Rate (Pitch)
10. LFO 2 Depth

### Mode Buttons
- **Octave +/−**: Shift frequency range (sub-bass vs mid-range)
- **Shift**: Reserved for future modes (keyboard, presets)

---

## Workflow Example

**Creating a 3-voice drone with evolving filter:**
1. Select Voice 1 → set to 220 Hz → activate
2. Select Voice 2 → set to 440 Hz → activate
3. Select Voice 3 → set to 660 Hz → activate
4. Adjust Filter Cutoff pot → sculpt timbre
5. Increase Resonance → emphasize filter peak
6. Set LFO 1 Rate to 0.5 Hz → slow filter sweep
7. Increase LFO 1 Depth → filter auto-sweeps
8. Set LFO 2 Rate to 0.3 Hz → gentle pitch wobble
9. Tweak Delay/Reverb Mix → add space
10. Hands-free evolving drone chord

---

## Development Phases

### ✅ Phase 0 (Complete)
- Basic 3-voice architecture
- Lock-free message passing (control → audio)
- Voice selection/toggle logic
- Audio output via PCM5102A

### 🔨 Phase 1 (Current Focus)
**Goal**: Complete core sound sculpting capabilities

- [ ] **LFO 1**: Filter Cutoff modulation
- [ ] **LFO 2**: Pitch modulation (vibrato)
- [ ] **Wavefolding**: Add wavefolder to signal chain
- [ ] **Filters**: Low-pass and high-pass (12 dB/oct)
- [ ] **Basic Effects**: Drive, Delay, Reverb
- [ ] **Envelopes**: Attack/Release (eliminate clicks on voice toggle)
- [ ] **Hardware Integration**: Wire up all pots, buttons, encoders

### Phase 2 (Extended Features)
- **LFO 3**: Resonance modulation
- **LFO 4**: Wavefolder depth modulation
- Per-voice fine detuning (chorus effect)
- Octave +/− button functionality
- Delay feedback control
- Reverb time control
- Frequency quantization mode (snap to chromatic scale)

### Phase 3 (Visualizer & Polish)
- Oscilloscope on CRT (Pmod VGA → composite)
- Parameter smoothing (eliminate zipper noise)
- LED animations (voice activity, LFO rates)

### Phase 4 (Major Features)
- Keyboard mode (13-button C→C octave)
- Additional waveforms (square, triangle, sawtooth)
- USB MIDI input
- Preset system (save/recall configurations)

### Phase 5 (Advanced)
- More effects (chorus, flanger, phaser)
- Spectrum analyzer / Lissajous visualizer
- Envelope followers (react to audio input)

---

## Design Priorities

1. **Audio quality first**: Clean signal path, no clicks/pops/dropouts
2. **Hands-free exploration**: Set up drones, sculpt with both hands on pots
3. **Immediate feedback**: Every control change instantly audible
4. **Modular architecture**: Easy to add features without breaking existing code
5. **West Coast aesthetic**: Continuous control, wavefolding, experimental tuning

**Goal**: Build a living, breathing instrument for exploring sound, not playing melodies.
