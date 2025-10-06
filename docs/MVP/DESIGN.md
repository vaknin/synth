# Design Document - Monophonic Drone Synthesizer

## Philosophy

**Pocket-portable electronic drone instrument** for exploratory sound creation.

- **Always-on continuous sound** - sculpt and morph a living, breathing tone
- **Monophonic triple-oscillator synthesis** - three detunable oscillators → filter → effects
- **Live improvisation** - real-time control, no sequencing or recording
- **Minimal and focused** - essential controls only, designed for completion
- **Pocket-portable form factor** - SP-404 size, portrait orientation, fits in hand or pocket
- **Aphex Twin aesthetic** - synthetic, electronic, warm but precise
- **Engineer's instrument** - full parameter control, no hidden behavior, precise and predictable
- **Physical and tactile** - quality knobs, immediate response, satisfying to use
- **Maximum flexibility** - blend waveforms, mix filter modes, modulate everything
- **Flawless precision** - professional-grade ADC for every control, no dead zones or jitter

---

## Core Sound Design

### Signal Flow

```
Oscillator 1 (Sine)           ┐
Oscillator 2 (Saw)            ├─→ [Mixer] ─→ [Multi-Mode Filter] ─→ [Drive] ─→
Oscillator 3 (Square, 50%)    │              (LP/HP/BP blend)
Sub-Oscillator                │
White Noise                   ┘

─→ [Delay] ─→ [Reverb] ─→ [Master Volume] ─→ Headphone Out (3.5mm)
```

### Oscillators

- **3 oscillators** with independent level and detune control
- **OSC1 (Sine):** Reference oscillator, cleanest tone
- **OSC2 (Sawtooth):** Bright, harmonic-rich, detunable
- **OSC3 (Square):** Hollow tone, fixed 50% duty cycle (pure square wave)
- **Master Pitch:** All oscillators track together (monophonic)
- **Detuning:** OSC2 and OSC3 can detune ±50 cents for beating/chorus effect
- **Sub-oscillator:** OSC1 pitched down one octave for bass weight and thickness
- **White noise:** Filtered noise for air, texture, and organic quality

### Filter

- **Multi-mode State Variable Filter** with blend capability
- **Three modes:** Low-Pass, High-Pass, Band-Pass (Notch removed to save panel space)
- **Independent level controls** for each mode (blend any combination)
- **Resonance can self-oscillate** (>1.0) for additional tonal voice
- **Cutoff range:** 20Hz - 20kHz (exponential scaling)

### Modulation

- **Filter Cutoff LFO** (sine wave, rate 0.01-20Hz + depth control)
- **Pitch LFO** (sine wave, rate 0.01-20Hz + depth control)
- **Extended LFO ranges** for glacial ambient evolution (down to 100-second cycles)
- **Note:** PWM LFO removed to reduce control count for pocket-portable form factor

### Effects

- **Delay:** Time (50ms - 5sec), Feedback (0-110%, can self-oscillate)
  - Always 100% wet when Time/Feedback > 0 (no separate Mix control for space saving)
- **Reverb:** Mix control (4-tap FDN algorithm, 5-8% CPU vs Freeverb's 15-25%)
- **Drive:** Saturation/waveshaping for harmonic richness and warmth

---

## Physical Layout

### Front Panel (Portrait Orientation)

```
┌─────────────────────────────────────┐
│       DRONE SYNTHESIZER             │
│                                     │
│    ┌─────────────────────────┐     │
│    │  1.5" OLED (128×128)    │     │
│    │  Oscilloscope Display   │     │
│    │  Battery % + Params     │     │
│    └─────────────────────────┘     │
│                                     │
│  Primary ─────────────→ Secondary   │
│                                     │
│  [Cutoff]   [Reso]    [LPLvl] [HPLvl]  ← FILTER
│                                     │
│  [MstrPit]  [O1Lvl]   [O2Lvl] [O3Lvl]  ← OSCILLATORS
│                                     │
│  [O2Det]    [O3Det]   [SubLvl][NoiseLv] ← DETUNE/MIX
│                                     │
│  [Drive]    [BPLvl]   [DelTime][DelFB]  ← EFFECTS
│                                     │
│  [FltLFOR]  [FltLFOD] [PitLFOR][PitLFOD]← MODULATION
│                                     │
│  [RvMix]    [Volume]  [FUTURE][FUTURE]  ← MASTER
│                                     │
└─────────────────────────────────────┘
```

### Edge-Mounted Connectors

```
TOP EDGE (178mm width):
    [3.5mm Jack]                      [Power Switch]
    (top-left)                        (top-right)

BOTTOM EDGE (178mm width):
    [3.5mm Jack]              [USB-C Charging Port]
    (bottom-left)             (bottom-right)
```

**Form Factor:**
- **Pocket-portable, portrait orientation** (taller than wide, fits in hand/pocket)
- **178mm (W) × 257mm (H) × 72mm (D)** (Roland SP-404 size)
- Screen at top for visibility during performance
- **4 columns × 6 rows** pot layout with importance-weighted positioning:
  - Left columns (1-2): Primary performance controls (most frequently adjusted)
  - Right columns (3-4): Secondary/set-and-forget controls
- **16mm potentiometers** (Bourns PTV09A or Alpha RD901F) with **25mm horizontal, 23mm vertical spacing**
  - Horizontal: 4 columns = 75mm span + 51mm side margins (comfortable clearance)
  - Vertical: 6 rows = 115mm span + 21mm top/bottom margins
- **15-18mm knob caps** (Davies 1900H with brass insert) provide 7-10mm finger clearance
- **2 spare pot positions** (bottom-right) reserved for future expansion
- Minimal aesthetic with clear labeling and color-coded knobs

---

## Complete Control List

### Oscillators (6 pots)
1. **Master Pitch** - Controls base frequency for all oscillators
2. **OSC1 Level** - Sine wave level (0-100%)
3. **OSC2 Level** - Sawtooth level (0-100%)
4. **OSC2 Detune** - Detune amount for OSC2 (±50 cents)
5. **OSC3 Level** - Square wave level (0-100%, fixed 50% duty cycle)
6. **OSC3 Detune** - Detune amount for OSC3 (±50 cents)

### Filter (5 pots)
7. **Cutoff** - Filter cutoff frequency (20Hz - 20kHz, exponential)
8. **Resonance** - Filter emphasis/Q (0.0 - 1.2, can self-oscillate)
9. **LP Level** - Low-pass filter output level (0-100%)
10. **HP Level** - High-pass filter output level (0-100%)
11. **BP Level** - Band-pass filter output level (0-100%)

### LFOs (4 pots - 2 LFOs)

**Filter Cutoff LFO** (sine wave modulation)
12. **Rate** - LFO frequency (0.01-20 Hz)
13. **Depth** - Modulation amount

**Pitch LFO** (sine wave modulation)
14. **Rate** - LFO frequency (0.01-20 Hz)
15. **Depth** - Vibrato/drift amount

### Effects (4 pots)

**Delay**
16. **Time** - Delay time (50ms - 5 seconds)
17. **Feedback** - Number of repeats (0-110%, can self-oscillate)

**Reverb**
18. **Mix** - Dry/wet blend

**Drive**
19. **Amount** - Saturation/distortion intensity

### Mixer (2 pots)
20. **Sub Level** - Sub-oscillator level (OSC1 -1 octave)
21. **Noise Level** - White noise level

### Master (1 pot)
22. **Volume** - Master output level (linear taper, software log scaling)

**Total: 22 potentiometers + 0 buttons**

**Removed from original 27-pot design:**
- ❌ PWM pot (OSC3 is fixed 50% square)
- ❌ Notch Level (LP/HP/BP sufficient, least-used filter mode)
- ❌ PWM LFO Rate/Depth (manual PWM sweep if needed in future)
- ❌ Delay Mix (always 100% wet when Time/Feedback > 0)
- ❌ Screen Toggle button (always-on display, 20+ hour battery accommodates continuous operation)

**Rationale:** Reducing from 27 to 22 pots and eliminating the screen button enables pocket-portable SP-404 form factor while preserving essential drone synthesis controls. Always-on oscilloscope visualization is a core feature.

---

## Display (1.5" SSD1351 OLED - 128×128 RGB)

### Visualization
- **Oscilloscope waveform display** (classic retro scope aesthetic with true blacks)
- Shows real-time output waveform (128 pixels wide, ~1-2 wavelengths visible)
- Color: Vibrant green on perfect black background (infinite contrast ratio)
- Runs continuously while device is on (no sleep/dim modes - visualization is the point)
- OLED advantages: instant pixel response (0.1ms), no ghosting, stunning visual impact

### Status Bar (Top)
- **Battery level** (percentage via MAX17043 fuel gauge + icon)
- **Current parameter being adjusted** (dynamic overlay)

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

### Power Management & EMI Mitigation
- OLED power: ~10-15mA (scope display, mostly black content)
- Always-on display design (20+ hour battery life accommodates continuous operation)
- **SPI EMI mitigation strategies** (critical for audio quality):
  - **Reduced GPIO drive strength:** `gpio_set_drive_capability(GPIO_DRIVE_CAP_0)` (weakest drive, reduces RF emissions)
  - **Lower SPI clock frequency:** 10-15MHz instead of 40MHz (still plenty fast for 60+ FPS, ~40dB less EMI)
  - **Physical separation:** ≥20mm distance between display and audio components (PCM5102A, OPA1622)
  - **PCB ground guard traces:** Flank audio I2S traces with GND traces on both sides
  - **Short, direct SPI wiring:** Minimize trace lengths to reduce antenna effect
  - **Separate ground zones:** OLED on digital ground (DGND), audio on analog ground (AGND), single-point connection

---

## Hardware Architecture

### Microcontroller
- **ESP32-S3** (dual-core Xtensa LX7, 240 MHz)
- Core 0: UI, display, control reading
- Core 1: Audio processing (dedicated, maximum priority)
- ~35 GPIO pins required (adequate headroom)

### Audio Output
- **PCM5102A DAC** (I2S interface, 44.1 kHz, 24-bit)
  - **22Ω series resistors** on I2S lines (BCLK, LRCK, DIN) to prevent crackling
  - **SCK pin grounded** to prevent digital noise pickup
  - 112dB SNR, proven compatibility with ESP32-S3
- **OPA1622 headphone amplifier** (audiophile-grade, 119dB SNR, 2.6mA per channel)
  - Chosen over TPA6130A2 for 21dB better SNR, fully utilizes 24-bit dynamic range
  - Drives 16-600Ω headphones with 119mW output power
- **3.5mm TRS headphone jack** (panel-mount)
- **Hardware pop suppression:** 100kΩ + 1µF RC delay on amp enable (prevents power-on clicks)

### Display
- **SSD1351 1.5" OLED** (128×128 pixels RGB, SPI interface)
- No touch capability needed
- ~10-15mA power draw (content-dependent, mostly-black scope display)
- Infinite contrast ratio (true blacks), perfect for classic oscilloscope aesthetic
- 0.1ms pixel response time (no ghosting)

### Power System
- **18650 Li-Ion battery** (3.7V, 3000mAh protected cell, **20+ hours runtime**)
- **BQ24075 USB-C charging IC** (true load-sharing, play-while-charging capability)
  - Upgraded from TP4056 to enable continuous use during charging
  - 1A charging current, full protection circuitry
- **Buck-boost + LDO regulation architecture** (optimized for maximum battery utilization):
  - **TPS63070 buck-boost converter** (3.6V output, 85-95% efficiency)
    - Maintains stable 3.6V from 4.2V (full charge) to 3.0V (depleted battery)
    - Unlocks 95% of battery capacity (2850mAh usable vs 600mAh with dual LDO cascade)
    - Critical upgrade: dual LDO cascade would shut down at 3.8V (only 4hr runtime)
  - **AP2112K-3.3V LDO** (single stage, final regulation)
    - Fed from 3.6V buck-boost output
    - 250mV dropout, excellent transient response
    - Provides 70dB PSRR at audio frequencies for clean analog power
    - 1000µF bulk capacitor for transient current handling
  - **Separate analog/digital power filtering**:
    - Local 10Ω + 100µF RC filters at PCM5102A DAC and OPA1622 amp power pins
    - Provides >65dB PSRR, <-100dB noise floor
- **1N5819 Schottky diode** (prevents backfeed)
- **SPST power switch** (main power control, panel-mount on enclosure top-right edge)
- **MAX17043 fuel gauge** (I2C, accurate battery percentage with ModelGauge algorithm)
  - Replaces simple voltage divider for ±1% accuracy vs ±15%

### Input Controls
- **22× potentiometers** (10kΩ linear, **16mm body diameter**, panel-mount)
  - **Bourns PTV09A** (recommended, $1.50-2 each) or **Alpha RD901F** (budget, $0.80-1.20 each)
  - 16mm body provides professional tactile feel, industry standard for serious synth control surfaces
  - All linear taper (including Volume - logarithmic scaling done in software)
  - Connected to **6× ADS1115 16-bit ADCs** via dual I2C buses (professional-grade precision)
  - Software applies exponential scaling for musical response (filter, LFO, pitch)
- **22× knob caps** (Davies 1900H style with brass insert, black with pointer, **15-18mm diameter**)
  - Brass insert prevents stripping (plastic-only versions fail after 20-30 uses)
  - At 25mm pot spacing, 15-18mm knobs provide 7-10mm finger clearance

### ADC Architecture (Critical for Precision)
- **6× ADS1115 16-bit I2C ADCs** (4 channels each = 24 channels, using 22)
  - **I2C Bus 0:** 4× ADS1115 (addresses 0x48, 0x49, 0x4A, 0x4B) → 16 pots
  - **I2C Bus 1:** 2× ADS1115 (addresses 0x48, 0x49) → 6 pots
- **Why ADS1115 16-bit?**
  - ESP32-S3 internal ADC has ±7 LSB DNL with dead zones at 0-0.1V and 3.0-3.16V
  - ADS1115 provides **±0.5 LSB DNL across entire 0-3.3V range** (true 16-bit linearity)
  - **65,536 resolution steps** vs 4,096 for 12-bit (16× smoother parameter sweeps)
  - Eliminates audible stepping artifacts on fast filter cutoff sweeps
  - Professional audio standard - pro control surfaces use 16-bit minimum
- **Scan time:** ~25.5ms for all 22 pots @ 860 SPS (39Hz update rate)
  - Total control-to-audio latency: ~61ms (ADC + smoothing + buffer)
  - Well within acceptable range for manual potentiometer control (<100ms threshold)
  - Professional synths (Moog, Prophet) operate in 10-30ms range for manual controls
- **I2C pull-up resistors:**
  - Bus 0: 2× 2.2kΩ resistors (SDA, SCL) - optimized for 4 ADCs + MAX17043
  - Bus 1: 2× 3.3kΩ or 4.7kΩ resistors (SDA, SCL) - optimized for 2 ADCs
  - Both buses operate at 400kHz Fast-mode I2C
- **No jitter, no dead zones, silky-smooth control** - professional studio-grade precision

### GPIO Pin Allocation

| Component                | Pins Required | Interface | Notes                          |
|--------------------------|---------------|-----------|--------------------------------|
| 22× Pots (via ADS1115)   | 4             | I2C       | 2× I2C buses (SDA+SCL each)    |
| MAX17043 Fuel Gauge      | 0             | I2C       | Shares I2C Bus 0 with ADS1115  |
| SSD1351 OLED Display     | 7             | SPI       | MOSI, CLK, CS, DC, RST         |
| PCM5102A DAC             | 3             | I2S       | BCLK, LRCK, DIN                |
| OPA1622 Amp Enable       | 1             | GPIO      | Pop suppression RC delay       |
| **Total**                | **15**        |           | **30 pins spare for expansion**|

ESP32-S3 has ~45 usable GPIO, providing substantial headroom for future features.

---

## Audio Specifications

- **Sample Rate**: 44.1 kHz (CD quality, industry standard for music production)
- **Bit Depth**: 24-bit in 32-bit frames (144dB dynamic range, fully utilizes OPA1622's 119dB SNR)
- **Buffer Size**: 256 samples (5.8ms latency @ 44.1kHz, circular DMA)
  - Reduced from 512 samples (11.6ms) for improved control responsiveness
  - Safe on dual-core ESP32-S3 with Core 1 dedicated to audio
- **CPU Clock**: 240 MHz (dual-core Xtensa LX7)
- **Filter Type**: State Variable Filter (3-mode blend: LP/HP/BP, simultaneous outputs)
- **LFO Waveform**: Sine wave (smooth, musical modulation)
- **Parameter Smoothing**: One-pole low-pass on all pots (5-30ms, prevents zipper noise)
- **Anti-aliasing**: PolyBLEP for sawtooth and square waves (prevents harsh aliasing artifacts)
- **Dual-core audio**: Core 1 dedicated to audio processing (priority), Core 0 for UI/display/ADC
- **Reverb**: 4-tap FDN (Feedback Delay Network) algorithm - 5-8% CPU vs Freeverb's 15-25%
- **Control-to-audio latency**: ~61ms total (25.5ms ADC scan + 30ms parameter smoothing + 5.8ms audio buffer)
  - Well within acceptable range for manual potentiometer control (<100ms perceptual threshold)
  - Professional synthesizers (Moog, Prophet) operate in 10-30ms range for manual controls
  - 16-bit ADC resolution (65,536 steps) eliminates audible stepping artifacts on fast sweeps

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
- ✅ Monophonic always-on drone synthesis
- ✅ 3 oscillators (Sine, Saw, Square-50%) with independent level and detune
- ✅ Sub-oscillator (OSC1 -1 octave) for bass weight
- ✅ White noise generator for organic texture
- ✅ Multi-mode filter (LP/HP/BP with blend capability, Notch removed)
- ✅ Filter resonance can self-oscillate (>1.0, acts as additional voice)
- ✅ 2 LFOs (filter cutoff, pitch) with rate/depth control each
- ✅ Extended LFO range (0.01-20Hz for glacial ambient evolution)
- ✅ Delay effect (time 50ms-5sec, feedback 0-110%, always 100% wet)
- ✅ Reverb effect (4-tap FDN algorithm, mix control)
- ✅ Drive/saturation (waveshaping for harmonic warmth)
- ✅ Live oscilloscope visualization (1.5" OLED with true blacks, infinite contrast, always-on design)
- ✅ Parameter display overlay with accurate battery % (MAX17043 fuel gauge)
- ✅ SPI EMI mitigation (reduced drive strength, lowered clock, physical separation, guard traces)
- ✅ USB-C charging + firmware updates (BQ24075, play-while-charging)
- ✅ Audiophile headphone output (PCM5102A DAC + OPA1622 amp, 119dB SNR, 24-bit)
- ✅ Battery powered (3000mAh 18650, **20+ hours runtime**)
- ✅ **Pocket-portable portrait form factor** (SP-404 size: 178×257×72mm)
- ✅ Clean audio (TPS63070 buck-boost + AP2112K LDO, isolated analog/digital rails, I2S series resistors)
- ✅ Professional DSP (PolyBLEP anti-aliasing, parameter smoothing, dual-core, 24-bit/44.1kHz @ 256 samples)
- ✅ **Flawless control precision** (6× ADS1115 16-bit ADCs, 65,536 steps, no dead zones, silky-smooth sweeps)
- ✅ **4-layer PCB with ground plane** (continuous AGND/DGND split, 1.6mm thickness, studio-grade construction)
- ✅ **16mm potentiometers** (Bourns PTV09A/Alpha RD901F, 25mm spacing, 4×6 importance-weighted layout)
- ✅ **Panel-mount 3.5mm TRS jack** (edge-mounted bottom-left, proper headphone-level amplification)

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

## PCB and Assembly Requirements

### PCB Specifications (Mandatory)
- **4-layer PCB required** - perfboard construction impossible for this complexity
  - **Layer 1 (Top):** Components and primary signal routing
  - **Layer 2 (Inner):** Solid ground plane (no gaps except analog/digital boundary)
  - **Layer 3 (Inner):** Power distribution plus secondary signals
  - **Layer 4 (Bottom):** Additional components and routing
- **Thickness:** 1.6mm (standard PCB fab thickness, optimal rigidity for panel-mount pots)
- **Why 4-layer is critical:**
  - Continuous ground plane essential for 240MHz ESP32-S3 RF containment
  - Dual I2C buses (22 pots + 6 ADCs) need controlled impedance
  - I2S audio at 2.8MHz requires proper trace routing with guard traces
  - AGND/DGND split requires separate ground zones with single-point connection
  - High-frequency SPI OLED (10-15MHz) needs isolation from audio path
- **Cost:** $50-100 for 5 prototype boards from JLCPCB ($10-20 each), production volumes reduce to $5-8/board

### Grounding Architecture
- **Split ground planes:** DGND (digital) and AGND (analog) connected via single 0Ω resistor at power input
- **DGND components:** ESP32, I2C ADCs, OLED display, digital power bypass
- **AGND components:** PCM5102A DAC, OPA1622 amp, audio jack, analog power bypass
- **Star grounding:** All ground points converge at single point near battery input

### Passive Components Detail

**Critical components (~75-90 total):**

**Decoupling capacitors (32-36 total):**
- ESP32-S3: 10-12 capacitors (1× 10µF at main power, 1× 10µF at VDD3P3_CPU, 6-8× 0.1µF distributed)
- 6× ADS1115 ADCs: 7-8 capacitors (1× 0.1µF per chip within 10mm of VDD pin, 1-2× 10µF shared bulk)
- PCM5102A DAC: 3-4 capacitors (2-3× 0.1µF, 1× 10µF bulk on analog supply)
- SSD1351 OLED: 1× 0.1µF
- MAX17043 fuel gauge: 1× 0.1µF
- TPS63070 buck-boost: 2-4 capacitors per datasheet
- AP2112K LDO: 2 capacitors (1× 1µF input, 1× 2.2µF output)
- All 0.1µF caps must be X5R or X7R ceramic (never Y5V/Z5U which lose 80% capacitance under DC bias)

**I2C pull-up resistors (4 total):**
- Bus 0: 2× 2.2kΩ (1% tolerance, 1/8W) for SDA and SCL
- Bus 1: 2× 3.3kΩ or 4.7kΩ (1% tolerance, 1/8W) for SDA and SCL

**I2S signal conditioning (3 total):**
- 3× 22Ω resistors (1/4W, 5%) on BCLK, LRCK, DIN lines (placed at ESP32 output pins)

**OPA1622 amplifier circuit (8 total):**
- 2× 499Ω input resistors (0.1% metal film)
- 2× 768Ω feedback resistors (0.1% metal film)
- 2× 470pF bandwidth limiting capacitors (film or C0G ceramic)
- 4× 0.1µF power supply bypass capacitors (X7R ceramic)

**ESD protection & safety:**
- 1× USBLC6-2SC6 integrated ESD suppressor (USB-C port, placed within 5mm of connector)
- 2× 100Ω resistors (audio output current limiting, 1/4W)
- 1× 1.5A PTC resettable fuse (battery protection)

**TPS63070 buck-boost supporting components (per datasheet):**
- Inductor, input/output capacitors, feedback resistors (exact values per TI datasheet)

### Audio Output Path
- **Signal chain:** PCM5102A DAC → OPA1622 headphone amp → **Panel-mount 3.5mm TRS jack**
- **Critical:** The 3.5mm jack on the PCM5102A module is NOT used (line-level output, insufficient for headphones)
- **Panel-mount jack required:** PJ-320A or PJ-307 (3-pole TRS, threaded bushing, $1-3)
- **Wiring:** Solder wires from PCM5102A LOUT/ROUT/AGND pads → OPA1622 input circuit → panel jack
- **Placement:** Bottom-left edge of enclosure (see Physical Layout section)

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

**Why 2 dedicated LFOs (not 3)?**
- Filter cutoff modulation: Essential for movement (most common, 90% usage)
- Pitch modulation: Vibrato, drift, detuning effects (60% usage)
- PWM LFO removed: Manual PWM sweep possible, but fixed 50% square saves panel space
- Independent rate/depth per LFO allows complex interactions
- Slow rates (0.01Hz) enable glacial ambient evolution
- Reduction from 3 to 2 LFOs enables pocket-portable form factor (27→22 pots)

**Why these effects?**
- Delay: Essential for ambient music, long times (2-5sec) create reverb-like spaces
- Reverb: Spatial depth, standard for drone/ambient
- Drive: Warmth, harmonic richness, "glue" for sterile digital tones
- Delay feedback >100% can self-oscillate (additional voice)

**Why these components?**
- **ESP32-S3:** Powerful dual-core, I2S audio, 2× I2C buses, plenty of GPIO, affordable
- **PCM5102A DAC:** Excellent DAC, simple I2S, 112dB SNR, 24-bit support, proven compatibility
- **OPA1622 amp:** Audiophile-grade 119dB SNR (21dB better than TPA6130A2), fully utilizes 24-bit
- **SSD1351 1.5" OLED:** Infinite contrast (true blacks), perfect for oscilloscope aesthetic, 10-15mA
- **BQ24075 charger:** Play-while-charging capability (TP4056 cannot do this)
- **MAX17043 fuel gauge:** Accurate battery % (±1% vs ±15% voltage divider)
- **6× ADS1015 ADCs:** Flawless precision, no dead zones (ESP32 ADC fails at 0-0.1V and 3-3.16V)
- **18650 battery:** Standard, replaceable, protected cell, 20+ hour runtime
- **Alps RK09K pots:** Quality tactile feel, 100k+ cycle life, brass-insert knobs
- **Dual AP2112K LDOs:** Clean audio via isolated analog/digital rails, >65dB PSRR

**Why pocket-portable form factor (SP-404 size)?**
- Truly portable: fits in hand, pocket, or small bag
- Always accessible: bring anywhere for inspiration
- Portrait orientation: natural grip, screen visible while holding
- 22 pots achievable: intelligent control reduction preserves essential parameters
- Not "in-between": committed to pocket-size, not awkward lap-only instrument

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
- Quality components (satisfying tactile feel, Alps pots with brass-insert knobs)
- Maximum flexibility (blend filters, mix effects, modulate everything)
- **Flawless precision:** ADS1015 ADCs eliminate ESP32's dead zones and jitter
- No compromises on audio quality (24-bit, OPA1622, dual isolated power rails)

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
- ✅ 22Ω series resistors on I2S lines (standard source termination)
- ✅ Ground SCK pin on PCM5102A (prevents digital noise)
- ✅ Dual AP2112K LDO cascade (rejected RC filter - would cause 8.8V drop!)
- ✅ Massive decoupling (1000µF on ESP32 rail for transient handling)
- ✅ Protected 18650 cell (essential for safety)
- ✅ BQ24075 charger (upgraded from TP4056 for play-while-charging)
- ✅ MAX17043 fuel gauge (accurate ±1% battery monitoring)
- ✅ OPA1622 amp upgrade (119dB SNR vs TPA6130A2's 98dB)
- ✅ Hardware pop suppression (100kΩ + 1µF RC delay on amp enable)

**DSP implementation (non-negotiable):**
- ✅ PolyBLEP anti-aliasing for saw/square (prevents aliasing artifacts)
- ✅ Parameter smoothing (one-pole low-pass on all pots, prevents zipper noise)
- ✅ State Variable Filter for multi-mode capability (LP/HP/BP simultaneous outputs)
- ✅ Dual-core architecture (Core 1 audio priority, Core 0 UI/display)
- ✅ Single-precision floats exclusively (f32, optimal for ESP32)
- ✅ Exponential parameter scaling for musical response
- ✅ 24-bit @ 44.1kHz (upgraded from 16-bit, industry standard)
- ✅ 512-sample DMA buffer (11.6ms latency, down from 46ms)
- ✅ 4-tap FDN reverb (5-8% CPU vs Freeverb's wasteful 15-25%)

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
- 🚀 Filter mode **blending** (3 level pots vs. 4-position switch) - Total flexibility
- 🚀 **6× ADS1115 16-bit ADCs** on dual I2C buses - Expert suggested ADS1115, we confirmed and optimized I2C architecture
- 🚀 **1.5" OLED** (vs. 2.8" TFT) - Infinite contrast, true blacks, perfect oscilloscope aesthetic with always-on design
- 🚀 **256-sample buffer** (5.8ms) vs expert's 512-sample suggestion - Improved responsiveness while maintaining stability

### What We Changed After Expert Review (Final Design Decisions)

**Critical power system fix (prevents 80% battery capacity loss):**
- 🔄 **TPS63070 buck-boost + AP2112K LDO** (replaces dual LDO cascade)
- ✅ Maintains 3.6V across entire battery discharge (4.2V → 3.0V)
- ✅ Unlocks 95% battery capacity (2850mAh usable vs 600mAh with dual LDO)
- ✅ Achieves true 20+ hour runtime (dual LDO would shut down at 3.8V = only 4 hours)
- ✅ Adds only $3-4 in components for 500% capacity increase

**Form factor optimization (pocket-portable priority):**
- 🔄 Reduced from 27 to **22 potentiometers** (enables SP-404 size)
- 🔄 Removed Notch filter (LP/HP/BP sufficient, least-used mode)
- 🔄 Removed PWM pot (OSC3 fixed at 50% square wave)
- 🔄 Removed PWM LFO (manual sweep if needed, saves 2 pots)
- 🔄 Removed Delay Mix (always 100% wet when Time/FB > 0)
- 🔄 Removed Screen Toggle button (always-on display, 20+ hour battery accommodates)
- ✅ Result: **Pocket-portable 178×257×72mm** (fits in hand/pocket)
- 🔄 **4 columns × 6 rows layout** (importance-weighted, left = most-tweaked controls)
- 🔄 **16mm pots** (Bourns PTV09A/Alpha RD901F) at 25mm spacing for professional feel

**Audio quality uncompromising upgrades:**
- 🔄 **24-bit @ 44.1kHz** (was 16-bit) - Fully utilizes OPA1622's 119dB SNR
- 🔄 **All 22 pots via ADS1115 16-bit** (upgraded from ADS1015 12-bit)
  - ✅ 65,536 resolution steps vs 4,096 (16× smoother, eliminates stepping artifacts)
  - ✅ Professional audio standard, same cost as 12-bit
  - ✅ 25.5ms scan time acceptable for manual controls (61ms total latency < 100ms threshold)
- 🔄 **Dual I2C bus architecture** - 4× ADS1115 on I2C0, 2× on I2C1, elegant solution
- 🔄 **256-sample DMA buffer** (5.8ms vs 11.6ms) - Improved responsiveness, safe on dual-core
- 🔄 **4-tap FDN reverb** (not Freeverb) - 5-8% CPU vs 15-25%, better efficiency
- 🔄 **Panel-mount 3.5mm jack** required (PCM5102A module jack is line-level only)

**PCB & EMI mitigation (essential for functionality):**
- 🔄 **4-layer PCB mandatory** (perfboard impossible for 22 pots + dual I2C + high-freq signals)
  - ✅ Continuous ground plane for 240MHz ESP32 RF containment
  - ✅ AGND/DGND split with single-point connection
  - ✅ Cost: $50-100 for 5 prototypes ($10-20 each)
- 🔄 **1.6mm PCB thickness** (standard, optimal rigidity for panel-mount pots)
- 🔄 **SPI OLED EMI mitigation** (keep SSD1351, mitigate noise instead of switching to I2C):
  - ✅ Reduced GPIO drive strength (GPIO_DRIVE_CAP_0)
  - ✅ Lower SPI clock (10-15MHz vs 40MHz, ~40dB less EMI)
  - ✅ ≥20mm physical separation from audio components
  - ✅ Ground guard traces flanking I2S signals

**Power & UX improvements:**
- 🔄 **Linear volume pot** with software log scaling (simpler, flexible)
- 🔄 **Software muting + hardware RC delay** (pop suppression both methods)
- 🔄 **I2C pull-up resistor values specified:** 2.2kΩ (Bus 0), 3.3-4.7kΩ (Bus 1)
- 🔄 **~75-90 passive components detailed** (decoupling, ESD protection, safety)

---

## Next Steps (Implementation Priority)

### Phase 1: Core Audio Engine
1. Triple oscillator implementation (sine, saw with PolyBLEP, square-50%)
2. State Variable Filter with 3-mode blend (LP/HP/BP)
3. 2 LFO system (filter cutoff, pitch)
4. Parameter smoothing (one-pole low-pass, slew limiting)
5. I2S audio output (24-bit, 44.1kHz, 512-sample buffer, 22Ω series resistors, grounded SCK)

### Phase 2: Effects & Modulation
6. Delay (with PSRAM for 5-second buffer, always 100% wet)
7. 4-tap FDN reverb (efficient algorithm, mix control)
8. Drive/saturation (soft clipping/waveshaping)
9. Sub-oscillator and white noise (bass weight and texture)

### Phase 3: ADC & Display Integration
10. Dual I2C bus setup (I2C0 + I2C1)
11. 6× ADS1015 ADC configuration and pot reading (8ms scan, 125Hz update rate)
12. SSD1351 OLED display driver (SPI, oscilloscope waveform)
13. Parameter overlay on display with fade-out
14. MAX17043 fuel gauge (accurate battery %, I2C)
15. Screen toggle button implementation

### Phase 4: Power & Enclosure
16. Dual AP2112K LDO cascade (isolated analog/digital rails)
17. BQ24075 charging circuit (play-while-charging capability)
18. Hardware pop suppression (RC delay on OPA1622 enable)
19. Custom 2-layer PCB design (recommended over perfboard for 22 pots)
20. Enclosure design (SP-404 size: 178×257×72mm, 3D print or sourced)
21. Panel layout (22 pots with 22mm spacing, portrait orientation)
22. Final assembly and testing (thermal, audio quality, battery runtime verification)

---

**This design represents a complete, buildable MVP for a professional-quality pocket-portable drone synthesizer with uncompromising audio fidelity and flawless control precision.**
