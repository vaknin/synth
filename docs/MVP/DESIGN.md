# Design Document - Monophonic Drone Synthesizer

## Philosophy

**Handheld electronic drone instrument** for exploratory sound creation.

- **Always-on continuous sound** - sculpt and morph a living, breathing tone
- **Monophonic triple-oscillator synthesis** - three detunable oscillators → filter → effects
- **Live improvisation** - real-time control, no sequencing or recording
- **Minimal and focused** - essential controls only, designed for completion
- **Aphex Twin aesthetic** - synthetic, electronic, warm but precise
- **Engineer's instrument** - full parameter control, no hidden behavior, precise and predictable
- **Physical and tactile** - quality knobs, immediate response, satisfying to use
- **Maximum flexibility** - blend waveforms, mix filter modes, modulate everything

---

## Core Sound Design

### Signal Flow

```
Oscillator 1 (Sine)     ┐
Oscillator 2 (Saw)      ├─→ [Mixer] ─→ [Multi-Mode Filter] ─→ [Drive] ─→
Oscillator 3 (Square)   │              (LP/HP/BP/Notch blend)
Sub-Oscillator          │
White Noise             ┘

─→ [Delay] ─→ [Reverb] ─→ [Master Volume] ─→ Headphone Out (3.5mm)
```

### Oscillators

- **3 oscillators** with independent level and detune control
- **OSC1 (Sine):** Reference oscillator, cleanest tone
- **OSC2 (Sawtooth):** Bright, harmonic-rich, detunable
- **OSC3 (Square/PWM):** Hollow tone with pulse width modulation
- **Master Pitch:** All oscillators track together (monophonic)
- **Detuning:** OSC2 and OSC3 can detune ±50 cents for beating/chorus effect
- **Sub-oscillator:** OSC1 pitched down one octave for bass weight
- **White noise:** Filtered noise for air and texture

### Filter

- **Multi-mode State Variable Filter** with blend capability
- **Four modes:** Low-Pass, High-Pass, Band-Pass, Notch
- **Independent level controls** for each mode (blend any combination)
- **Resonance can self-oscillate** (>1.0) for additional tonal voice
- **Cutoff range:** 20Hz - 20kHz (exponential scaling)

### Modulation

- **Filter Cutoff LFO** (sine wave, rate 0.01-20Hz + depth control)
- **Pitch LFO** (sine wave, rate 0.01-20Hz + depth control)
- **PWM LFO** (sine wave, rate 0.01-20Hz + depth control, modulates OSC3 pulse width)
- **Extended LFO ranges** for glacial ambient evolution (down to 100-second cycles)

### Effects

- **Delay:** Time (50ms - 5sec), Feedback (0-110%, can self-oscillate), Mix
- **Reverb:** Mix control (Freeverb algorithm)
- **Drive:** Saturation/waveshaping for harmonic richness and warmth

---

## Physical Layout

```
┌──────────────────────────────────────────────────┐
│              DRONE SYNTHESIZER                   │
│                                                  │
│     ┌────────────────────────────────────┐      │
│     │   3.5" TFT LCD (Landscape)         │      │
│     │   Oscilloscope + Status Bar        │      │
│     │   (Battery %, Param Values)        │      │
│     └────────────────────────────────────┘      │
│                                          [Scrn]  │  ← Screen toggle
│                                                  │
│  ┌─────────── PERFORMANCE CONTROLS ───────────┐ │
│  │ [Cutoff] [Reso] [Drive] [DlMix] [RvMix]    │ │
│  │ [LPLvl] [HPLvl]                             │ │
│  └─────────────────────────────────────────────┘ │
│                                                  │
│  ┌───────────── OSCILLATORS ──────────────────┐ │
│  │ [MstrPitch] [O1Lvl] [O2Lvl] [O2Det]        │ │
│  │ [O3Lvl] [O3Det] [PWM]                      │ │
│  └─────────────────────────────────────────────┘ │
│                                                  │
│  ┌──────────── MODULATION ────────────────────┐ │
│  │ [FltLFORate] [FltLFODpth] [PitLFORate]     │ │
│  │ [PitLFODpth] [PWMLFORate] [PWMLFODpth]     │ │
│  └─────────────────────────────────────────────┘ │
│                                                  │
│  ┌─────── EFFECTS & MIXER ─────────────────────┐│
│  │ [DelTime] [DelFB] [BPLvl] [NotchLvl]       │ │
│  │ [SubLvl] [NoiseLvl]                        │ │
│  └─────────────────────────────────────────────┘ │
│                                                  │
│              [Volume]                            │
│              (log taper)                         │
│                                                  │
│  [Power]          [USB-C]        [3.5mm Jack]   │
└──────────────────────────────────────────────────┘
```

**Form Factor:**
- Handheld, portrait orientation (taller than wide)
- **~178mm (W) × 276mm (H) × ~50-70mm (D)** (similar to Roland SP-404)
- Screen at top for visibility
- Controls grouped by function (performance → oscillators → modulation → effects)
- 16mm potentiometers with comfortable spacing (22mm center-to-center)
- Minimal aesthetic with clear labeling

---

## Complete Control List

### Oscillators (7 pots)
1. **Master Pitch** - Controls base frequency for all oscillators
2. **OSC1 Level** - Sine wave level (0-100%)
3. **OSC2 Level** - Sawtooth level (0-100%)
4. **OSC2 Detune** - Detune amount for OSC2 (±50 cents)
5. **OSC3 Level** - Square/PWM wave level (0-100%)
6. **OSC3 Detune** - Detune amount for OSC3 (±50 cents)
7. **PWM** - Pulse width for OSC3 (10-90% duty cycle)

### Filter (6 pots)
8. **Cutoff** - Filter cutoff frequency (20Hz - 20kHz, exponential)
9. **Resonance** - Filter emphasis/Q (0.0 - 1.2, can self-oscillate)
10. **LP Level** - Low-pass filter output level (0-100%)
11. **HP Level** - High-pass filter output level (0-100%)
12. **BP Level** - Band-pass filter output level (0-100%)
13. **Notch Level** - Notch filter output level (0-100%)

### LFOs (6 pots)

**Filter Cutoff LFO** (sine wave modulation)
14. **Rate** - LFO frequency (0.01-20 Hz)
15. **Depth** - Modulation amount

**Pitch LFO** (sine wave modulation)
16. **Rate** - LFO frequency (0.01-20 Hz)
17. **Depth** - Vibrato/drift amount

**PWM LFO** (sine wave modulation)
18. **Rate** - LFO frequency (0.01-20 Hz)
19. **Depth** - Pulse width modulation amount

### Effects (5 pots)

**Delay**
20. **Time** - Delay time (50ms - 5 seconds)
21. **Feedback** - Number of repeats (0-110%, can self-oscillate)
22. **Mix** - Dry/wet blend

**Reverb**
23. **Mix** - Dry/wet blend

**Drive**
24. **Amount** - Saturation/distortion intensity

### Mixer (2 pots)
25. **Sub Level** - Sub-oscillator level (OSC1 -1 octave)
26. **Noise Level** - White noise level

### Master (1 pot)
27. **Volume** - Master output level (logarithmic taper)

### Buttons (1 button)
28. **Screen Toggle** - Turn display on/off to save battery

**Total: 27 potentiometers + 1 button**

---

## Display (3.5" ILI9341 LCD - Landscape Orientation)

### Visualization
- **Oscilloscope waveform display** (classic retro scope aesthetic)
- Shows real-time output waveform
- Color: Green/cyan on black background
- Runs continuously while device is on

### Status Bar (Top)
- **Battery level** (percentage + icon)
- **Current patch/mode** (if applicable)

### Parameter Overlay
- When any control is adjusted, overlay appears showing:
  ```
  Cutoff: 800 Hz
  ```
  or
  ```
  LFO Rate: 2.3 Hz
  ```
- Overlay fades after 2 seconds, returns to pure waveform view

### Power Management
- **Screen toggle button** allows turning display off to conserve battery
- Pressing button again or adjusting any control wakes display
- Display consumes ~25-30mA, turning off extends battery life by ~1 hour

---

## Hardware Architecture

### Microcontroller
- **ESP32-S3** (dual-core Xtensa LX7, 240 MHz)
- Core 0: UI, display, control reading
- Core 1: Audio processing (dedicated, maximum priority)
- ~35 GPIO pins required (adequate headroom)

### Audio Output
- **PCM5102A DAC** (I2S interface, 44.1 kHz, 16-bit)
  - **22Ω series resistors** on I2S lines (BCLK, LRCK, DIN) to prevent crackling
  - **SCK pin grounded** to prevent digital noise pickup
- **TPA6130A2 headphone amplifier** (low noise, high quality, drives 16-600Ω)
- **3.5mm TRS headphone jack**

### Display
- **ILI9341 3.5" TFT LCD** (320×240 pixels landscape, SPI interface)
- No touch capability needed
- ~25-30mA power draw

### Power System
- **18650 Li-Ion battery** (3.7V, 3000mAh protected cell, ~4-6 hours runtime)
- **TP4056 USB-C charging module** (with protection)
  - **Cannot play while charging** (load-sharing limitation)
- **Separate analog/digital power supplies:**
  - **Digital rail:** HT7833 3.3V regulator for ESP32 + Display
    - 1000µF bulk capacitor for WiFi transient handling
  - **Analog rail:** RC filter (220Ω + 220µF) from digital rail to DAC/Amp
    - Alternative: Dedicated TPS7A4700 ultra-low-noise regulator (~4µVrms)
- **1N5819 Schottky diode** (prevents backfeed)
- **SPST power switch**
- **Battery level monitoring** via ESP32 ADC (displayed on screen)

### Input Controls
- **27× potentiometers** (10kΩ linear, 16mm panel-mount)
  - All linear taper except Volume (logarithmic)
  - Connected directly to ESP32-S3 ADC pins
  - Software applies exponential scaling for musical response (filter, LFO)
- **1× tactile button** (screen on/off toggle)
- **27× knob caps** (Davies 1900H style or similar, black with pointer, 15-20mm diameter)

### GPIO Pin Allocation

| Component          | Pins Required | Interface |
|--------------------|---------------|-----------|
| 27× Potentiometers | 27            | ADC       |
| 1× Button          | 1             | GPIO      |
| ILI9341 Display    | 5             | SPI       |
| PCM5102A DAC       | 3             | I2S       |
| Battery Monitor    | 1             | ADC       |
| **Total**          | **37**        |           |

ESP32-S3 has ~45 usable GPIO, providing adequate headroom.

---

## Audio Specifications

- **Sample Rate**: 44.1 kHz (CD quality)
- **Bit Depth**: 16-bit (PCM5102A native)
- **Buffer Size**: 64-128 samples (~1.5-3ms latency)
- **CPU Clock**: 80-160 MHz (optimize for lowest speed that maintains glitch-free audio)
- **Filter Type**: State Variable Filter (multi-mode with blend capability)
- **LFO Waveform**: Sine wave (smooth, musical)
- **Parameter Smoothing**: One-pole low-pass on all pots (5-30ms, prevents zipper noise)
- **Anti-aliasing**: PolyBLEP for sawtooth and square waves (prevents harsh digital sound)
- **Dual-core audio**: Core 1 dedicated to audio processing, Core 0 for UI/display

### DSP Best Practices
- Single-precision floats exclusively (`440.0f`, `sinf()`, `cosf()`)
- Pre-calculate reciprocals (multiply instead of divide)
- Exponential parameter scaling for musical response
- Extended parameter ranges (resonance/feedback can self-oscillate)
- All modulation depths are user-controllable (no hidden automation)

---

## User Experience

### Session Flow
1. Power on device (SPST switch)
2. Sound begins **immediately** (always-on drone, all oscillators active but OSC2/3 at 0% level)
3. **Start simple:** OSC1 Level up, adjust Master Pitch and Filter
4. **Add complexity when ready:** Turn up OSC2 Level, adjust Detune for beating
5. Add OSC3, Sub, Noise as desired
6. Sculpt timbre with filter blend (LP/HP/BP/Notch levels)
7. Add modulation (LFOs), space (delay/reverb), and character (drive)
8. Explore, improvise, meditate with headphones

### Interaction Model
- **Continuous control** - all pots, no menus
- **Immediate feedback** - hear and see changes in real-time
- **Tactile satisfaction** - quality 16mm pots, smooth rotation
- **Visual confirmation** - screen shows what you're adjusting
- **No modes** - everything always accessible
- **Engineer's instrument** - precise, predictable, full control, no surprises
- **Progressive complexity** - start with one oscillator, add layers when ready

### Design Goals
- **Completion over perfection** - MVP is buildable and finishable
- **Quality over features** - 27 well-chosen controls, no bloat
- **Experience over specs** - spiritual, exploratory, creative connection
- **Engineer's pride** - full control, no surprises, own your sound
- **Maximum flexibility** - blend waveforms, mix filters, modulate everything

---

## MVP Scope

### Included
- ✅ Monophonic always-on drone
- ✅ 3 oscillators (Sine, Saw, Square/PWM) with independent level and detune
- ✅ Sub-oscillator (OSC1 -1 octave)
- ✅ White noise generator
- ✅ Multi-mode filter (LP/HP/BP/Notch with blend capability)
- ✅ Filter resonance can self-oscillate (acts as additional voice)
- ✅ 3 LFOs (filter cutoff, pitch, PWM) with rate/depth control each
- ✅ Extended LFO range (0.01-20Hz for glacial ambient evolution)
- ✅ Delay effect (time 50ms-5sec, feedback 0-110%, mix)
- ✅ Reverb effect (mix)
- ✅ Drive/saturation (waveshaping for warmth)
- ✅ Live oscilloscope visualization (landscape 3.5" display)
- ✅ Parameter display overlay with battery level
- ✅ Screen toggle button for battery saving
- ✅ USB-C charging + firmware updates
- ✅ Headphone output (high quality PCM5102A DAC + TPA6130A2 amp)
- ✅ Battery powered (3000mAh 18650, 4-6 hours runtime)
- ✅ Handheld portrait form factor (~SP-404 size)
- ✅ Clean audio (separate analog/digital power, I2S series resistors)
- ✅ Professional DSP (PolyBLEP anti-aliasing, parameter smoothing, dual-core)

### Explicitly Excluded (Post-MVP)
- Polyphony (multiple independent voices)
- Note triggering / keyboard input
- Sequencing or patterns
- Recording or saving patches
- Additional oscillator waveforms (triangle, etc.)
- LFO waveform options (square, random, etc.)
- Additional LFO targets (amplitude, filter resonance, etc.)
- Cross-modulation between LFOs
- Additional effects (chorus, phaser, flanger, etc.)
- Preset management / patch storage
- Touch screen interaction
- Accelerometer/gyro input
- Bluetooth/WiFi connectivity
- MIDI input (USB or DIN)
- Stereo output
- Line output (1/4" jack)
- External audio input
- CV/Gate inputs

---

## Design Rationale

**Why monophonic always-on?**
- Simplest DSP, lowest CPU overhead
- Meditative, focused experience
- No timing/triggering complexity
- Pure sound sculpting
- Aligns with drone synthesis philosophy (Lyra-8, ambient instruments)

**Why 3 oscillators with detuning?**
- Single oscillator cannot create beating/chorus essential to drone music
- Detuning creates "breathing," evolving, organic quality
- Independent level controls allow progressive complexity (start simple, add layers)
- Fixed waveforms per oscillator simplify panel (no waveform switches)

**Why multi-mode filter with blend capability?**
- LP/HP/BP/Notch cover all essential filter types
- Blending (e.g., 50% LP + 50% HP) creates unique timbres impossible with switches
- State Variable Filter provides all outputs simultaneously (efficient)
- Self-oscillating resonance adds additional tonal voice

**Why 3 dedicated LFOs?**
- Filter cutoff modulation: Essential for movement (most common)
- Pitch modulation: Vibrato, drift, detuning effects
- PWM modulation: Chorusing, phasing, classic analog character
- Independent rate/depth per LFO allows complex interactions
- Slow rates (0.01Hz) enable glacial ambient evolution

**Why these effects?**
- Delay: Essential for ambient music, long times (2-5sec) create reverb-like spaces
- Reverb: Spatial depth, standard for drone/ambient
- Drive: Warmth, harmonic richness, "glue" for sterile digital tones
- Delay feedback >100% can self-oscillate (additional voice)

**Why these components?**
- ESP32-S3: Powerful dual-core, I2S audio, plenty of GPIO, affordable
- PCM5102A: Excellent DAC, simple I2S, 112dB SNR, proven compatibility
- TPA6130A2: Industry-standard headphone amp, low noise, high PSRR
- ILI9341 3.5": Common, well-supported, readable size for visualization
- 18650 battery: Standard, replaceable, safe with TP4056, 4-6 hour runtime
- 16mm pots: Quality tactile feel, comfortable spacing, precise control
- Separate power supplies: Clean audio, eliminates ESP32 digital noise

**Why no presets/patch storage?**
- Pot position mismatch creates UX complexity ("pickup mode" needed)
- Emphasizes live exploration over recall
- Simpler code, fewer failure modes
- Can be added post-MVP if demand exists

**Why engineer's instrument philosophy?**
- Full parameter control (no hidden automation or fixed routings)
- Predictable behavior (turn knob, hear expected result)
- Extended ranges allowed (self-oscillation, extremes embraced)
- Visual feedback (display confirms every adjustment)
- Quality components (satisfying tactile feel)
- Maximum flexibility (blend, mix, modulate everything)

---

## Response to Expert Feedback

### What We Accepted

**Critical additions (high impact):**
- ✅ Multiple oscillators (implemented 3 with independent detune)
- ✅ PWM waveform with dedicated LFO (classic analog chorusing)
- ✅ Multi-mode filter (LP/HP/BP/Notch, went further with blend capability)
- ✅ Drive/saturation (warmth and harmonic richness)
- ✅ Sub-oscillator and white noise (bass weight and texture)
- ✅ Extended LFO ranges (0.01-20Hz for glacial evolution)
- ✅ Longer delay times (50ms-5sec for ambient spaces)
- ✅ Extended parameter ranges (resonance/feedback self-oscillation)

**Hardware quality (essential):**
- ✅ 22Ω series resistors on I2S lines
- ✅ Ground SCK pin on PCM5102A
- ✅ Separate analog/digital power supplies (RC filter or dedicated regulator)
- ✅ Massive decoupling (1000µF on ESP32 rail)
- ✅ Protected 18650 cell
- ✅ Logarithmic volume pot

**DSP implementation (non-negotiable):**
- ✅ PolyBLEP anti-aliasing for saw/square
- ✅ Parameter smoothing (one-pole low-pass on all pots)
- ✅ State Variable Filter for multi-mode capability
- ✅ Dual-core architecture (Core 1 audio, Core 0 UI)
- ✅ Single-precision floats exclusively
- ✅ Exponential parameter scaling

### What We Rejected (and Why)

**Rejected for MVP (may revisit post-launch):**
- ❌ LFO cross-modulation - Adds complexity, marginal gain vs. 3 independent LFOs
- ❌ Random/stepped modulation - Extra pot/switch, niche use case
- ❌ Accelerometer/touch input - Doesn't fit "engineer's instrument" (imprecise)
- ❌ Spectrum analyzer - Oscilloscope sufficient for MVP, can add as toggle later
- ❌ Granular processing - CPU-intensive, needs 4-5 controls, major feature creep
- ❌ Preset storage - UX complexity (pot mismatch), emphasizes recall over exploration

**Rejected permanently (conflicts with design philosophy):**
- ❌ Hidden/invisible automation - Every parameter must have explicit user control
- ❌ Fixed LFO routings - User must control depth via dedicated pots
- ❌ Feature beyond scope - MIDI, WiFi, recording (bloat, delays completion)

### What We Enhanced Beyond Recommendations

**Went further than expert suggested:**
- 🚀 Filter mode **blending** (4 level pots vs. 4-position switch) - Total flexibility
- 🚀 **Dedicated PWM LFO** (vs. routing existing LFO) - Independent rate/depth control
- 🚀 **3.5" display** (vs. 2.8") - Better readability for oscilloscope/parameters
- 🚀 **Screen toggle button** - Battery optimization not mentioned by expert

---

## Next Steps (Implementation Priority)

### Phase 1: Core Audio Engine
1. Triple oscillator implementation (sine, saw with PolyBLEP, square/PWM)
2. State Variable Filter with 4-mode blend
3. Basic LFO system (filter, pitch, PWM)
4. Parameter smoothing (slew limiting)
5. I2S audio output with proper resistors/grounding

### Phase 2: Effects & Modulation
6. Delay (with PSRAM for long buffer)
7. Reverb (Freeverb algorithm)
8. Drive/saturation (soft clipping)
9. Sub-oscillator and white noise

### Phase 3: Hardware Integration
10. 27× ADC pot reading with debouncing
11. Display driver (oscilloscope waveform)
12. Parameter overlay on display
13. Battery level monitoring
14. Screen toggle button

### Phase 4: Power & Enclosure
15. Dual power supply implementation (digital + analog)
16. Battery charging circuit (TP4056 + protection)
17. PCB or perfboard layout
18. Enclosure design (3D print or project box)
19. Panel layout and drilling
20. Final assembly and testing

---

**This design represents a complete, buildable MVP for a professional-quality handheld drone synthesizer.**
