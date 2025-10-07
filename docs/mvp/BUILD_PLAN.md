# Drone Synthesizer MVP Build Plan

High-level roadmap for building a pocket-portable monophonic drone synthesizer with professional audio quality.

---

## Phase 0: Component Procurement (Week 1)

**Status**: Not Started
**Budget**: $20-35 (already own ESP32-S3 and PCM5102A)

### Order from AliExpress/Budget Suppliers
- [ ] 3× ADS1115 16-bit I2C ADC modules ($4.50-9 total)
- [ ] SSD1351 1.5" OLED display 128×128 RGB ($10-15)
- [ ] 10× B10K linear potentiometers 10kΩ ($2-6 total)
- [ ] 2× 830-point breadboards - quality brands ($8-12 total)
- [ ] 22 AWG solid core wire kit - multiple colors ($8-12)
- [ ] Capacitor assortment (0.1µF, 10µF, 100-470µF) ($5-8)
- [ ] 4.7kΩ resistors 1/4W (10× pack) ($0.50)
- [ ] USB-C cable for programming/power ($2-4)

### Verify Owned Components
- [ ] ESP32-S3-DevKitC-1 with N8R8 (8MB PSRAM) - functional
- [ ] PCM5102A DAC module with 3.5mm jack - functional

**Shipping time**: 2-4 weeks from AliExpress, 3-7 days from domestic suppliers

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
    LpLevel = 8,
    HpLevel = 9,
    BpLevel = 10,
    FilterLfoRate = 11,
    FilterLfoDepth = 12,
    PitchLfoRate = 13,
    PitchLfoDepth = 14,
    DelayTime = 15,
    DelayFeedback = 16,
    ReverbMix = 17,
    Drive = 18,
    SubLevel = 19,
    NoiseLevel = 20,
    Volume = 21,
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

**Implementation:**
```rust
match self.waveform {
    WaveformType::Sine => self.sine_table_lookup(),  // 1024-sample table, linear interpolation
    WaveformType::Saw => (self.phase * 2.0) - 1.0,   // Direct math (faster than table)
    WaveformType::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },  // Branch predictor wins
}
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

**Unified message type:**
```rust
pub enum Message {
    SetControl(ControlId, f32),  // Single message type for all 22 parameters
}
```

**Benefits:**
- ✅ Scalable: Adding controls 11-22 requires zero new message types
- ✅ Type-safe: Enum prevents invalid control IDs
- ✅ Clean pot code: All pots send identical message format

**Message handling pattern (prevents 100+ line match):**
```rust
fn consume_messages(&mut self) {
    while let Ok(msg) = self.consumer.try_receive() {
        match msg {
            Message::SetControl(id, value) => self.handle_control(id, value),
        }
    }
}

fn handle_control(&mut self, id: ControlId, value: f32) {
    match id {
        MasterPitch => self.set_master_pitch(value),      // Helper method
        Osc1Level => self.osc1_level.set_target(value),   // Direct assignment
        Cutoff => self.filter.set_cutoff(value),          // Delegate to module
        // ... grouped by category, ~30 lines total
    }
}

// Complex logic extracted to helper methods
fn set_master_pitch(&mut self, value: f32) {
    let freq = map_pitch(value);  // 0.0-1.0 → 30Hz-1024Hz exponential
    self.osc1.set_frequency(freq);
    self.osc2.set_frequency(freq * self.osc2_detune_ratio);
    self.osc3.set_frequency(freq * self.osc3_detune_ratio);
    self.sub_osc.set_frequency(freq * 0.5);
}
```

### Engine Structure

**Monophonic architecture (replaces polyphonic Voice abstraction):**
```rust
pub struct Engine {
    // 4 oscillator instances (direct fields for cache locality)
    osc1: Oscillator,      // Sine
    osc2: Oscillator,      // Sawtooth
    osc3: Oscillator,      // Square 50%
    sub_osc: Oscillator,   // Sine, -1 octave

    // Level controls (smoothed to prevent zipper noise)
    osc1_level: Slew,
    osc2_level: Slew,
    osc3_level: Slew,
    sub_level: Slew,

    // Detuning state
    osc2_detune_ratio: f32,  // Cached ratio for fast frequency updates
    osc3_detune_ratio: f32,

    // Message receiver
    consumer: ChannelReceiver,
}
```

**Design rationale:**
- ✅ **No Voice abstraction:** Unnecessary overhead for monophonic synthesis
- ✅ **Direct oscillator fields:** Cache locality (all oscillators adjacent in memory → L1 cache hits)
- ✅ **Master pitch tracking:** Single control updates all 4 oscillators simultaneously
- ✅ **Hot-path optimization:** `render()` has direct access to all state without indirection
- ✅ **Slew limiters on all params:** Prevents clicks and zipper noise (5-30ms smoothing)

**Rendering pipeline (Phase 1-3):**
```rust
pub fn render(&mut self, buffer: &mut [i16]) {
    for sample in buffer.chunks_exact_mut(2) {  // Stereo
        let mixed = (self.osc1.tick() * self.osc1_level.value())
                  + (self.osc2.tick() * self.osc2_level.value())
                  + (self.osc3.tick() * self.osc3_level.value())
                  + (self.sub_osc.tick() * self.sub_level.value());

        // Phase 1-3: Direct output
        let output = (mixed * MASTER_GAIN * I16_MAX_F32) as i16;

        // Phase 4+: Add filter/drive/effects here
        // let filtered = self.filter.process(mixed);
        // let driven = self.drive.process(filtered);
        // let output = (driven * self.master_volume.value() * I16_MAX_F32) as i16;

        sample[0] = output;  // Left
        sample[1] = output;  // Right (mono)
    }
}
```

---

## Phase 1: Breadboard MVP - Core Audio (Weeks 2-5)

**Status**: Not Started
**Goal**: Working 10-control drone synthesizer with line-level audio output

### Week 2: Audio Output Validation

- [ ] Set up dual breadboard workspace with power rail distribution
- [ ] Wire ESP32-S3 + PCM5102A on breadboard
- [ ] Configure I2S pins (GPIO1/2/3 → BCLK/LRCK/DIN)
- [ ] Implement 440Hz sine wave test tone in Rust
- [ ] Verify clean audio output through powered speakers
- [ ] Add proper decoupling capacitors (0.1µF at DAC VCC)

**Checkpoint**: Clean sine wave with no crackling or dropouts

### Week 3: First Oscillator + Control Reading

- [ ] Wire first ADS1115 module to I2C bus (address 0x48)
- [ ] Install 4.7kΩ pull-up resistors on I2C SDA/SCL lines
- [ ] Solder wires to first potentiometer (Master Pitch)
- [ ] Implement I2C ADC reading code in Rust
- [ ] Implement OSC1 (sine wave) with variable pitch control
- [ ] Add parameter smoothing (one-pole low-pass filter)

**Checkpoint**: Pot smoothly controls oscillator pitch from ~20Hz to ~2kHz

### Week 4: Triple Oscillator System

- [ ] Add remaining 2× ADS1115 modules (addresses 0x49, 0x4A)
- [ ] Solder wires to 7 more pots (OSC1/2/3 Levels, Sub Level, Cutoff, Resonance, Drive)
- [ ] Implement OSC2 (sawtooth with PolyBLEP anti-aliasing)
- [ ] Implement OSC3 (square wave 50% duty cycle with PolyBLEP)
- [ ] Implement Sub-oscillator (OSC1 pitched -1 octave)
- [ ] Implement oscillator mixer with independent level controls

**Checkpoint**: Can blend three oscillators + sub, hear distinct waveform characters

### Week 5: Filter + Effects Chain

- [ ] Implement State Variable Filter (LP/HP/BP simultaneous outputs)
- [ ] Add Cutoff and Resonance pot control with exponential scaling
- [ ] Implement Drive/saturation (soft clipping waveshaping)
- [ ] Wire final pots (Volume, potentially one spare for testing)
- [ ] Implement master volume control with software logarithmic scaling
- [ ] Tune parameter ranges and scaling curves

**Checkpoint**: Full signal chain OSC → Filter → Drive → Output sounds musical

---

## Phase 2: Display Integration (Week 6)

**Status**: Not Started
**Goal**: Visual feedback with oscilloscope waveform and parameter overlay

- [ ] Wire SSD1351 OLED to SPI bus (GPIO17/16/23/18/5 → CS/DC/MOSI/SCK/RST)
- [ ] Add 0.1µF decoupling capacitor at display VCC
- [ ] Port/integrate Adafruit_SSD1351 library for Rust (or use embedded-graphics)
- [ ] Implement oscilloscope waveform visualization (128 pixels wide, green trace)
- [ ] Implement parameter overlay display (shows control name + value)
- [ ] Add 2-second fade timer for overlay (returns to pure waveform view)
- [ ] Optimize SPI refresh rate for 30+ FPS without audio glitches

**Checkpoint**: Real-time waveform visualization with parameter feedback on pot adjustment

---

## Phase 3: Sound Design Refinement (Week 7)

**Status**: Not Started
**Goal**: Musical, expressive drone synthesis with optimized parameter ranges

- [ ] Test and tune filter cutoff range (20Hz - 20kHz, exponential mapping)
- [ ] Test and tune resonance range (0.0 - 1.2, enable self-oscillation)
- [ ] Verify PolyBLEP anti-aliasing eliminates harsh artifacts
- [ ] Add white noise generator with level control (if 10th pot available)
- [ ] Calibrate Drive saturation curve for warmth without harshness
- [ ] Test parameter smoothing time constants (5-30ms, prevent zipper noise)
- [ ] Document optimal pot positions for various drone textures

**Checkpoint**: Can create rich, evolving drone tones; all controls feel responsive and musical

---

## Phase 4: Code Optimization (Week 8)

**Status**: Not Started
**Goal**: Stable 44.1kHz audio with <5% CPU usage headroom

- [ ] Profile audio callback CPU usage (should be <95% of available time)
- [ ] Add `#[inline]` to hot-path functions (oscillators, filter, DSP)
- [ ] Optimize State Variable Filter implementation
- [ ] Verify 256-sample I2S buffer maintains zero underruns
- [ ] Test ADC scan rate (should complete 22 pots in ~25ms @ 860 SPS)
- [ ] Ensure display updates don't cause audio glitches
- [ ] Verify dual-core architecture (Core 1 audio, Core 0 UI/display)

**Checkpoint**: Audio never glitches, pots feel responsive, waveform animates smoothly

---

## Phase 5: Extended Features (Weeks 9-10) - OPTIONAL

**Status**: Not Started
**Goal**: Add modulation and effects if Phase 1-4 proves successful

### Modulation (if desired)
- [ ] Implement Filter Cutoff LFO (sine wave, 0.01-20Hz rate + depth)
- [ ] Implement Pitch LFO (sine wave, 0.01-20Hz rate + depth)
- [ ] Requires 4 additional pots - may need 4th ADS1115 module

### Effects (if desired)
- [ ] Implement Delay (50ms-5sec time, 0-110% feedback) using PSRAM buffer
- [ ] Implement Reverb (4-tap FDN algorithm, mix control)
- [ ] Requires 2-3 additional pots

**Decision point**: Only pursue if Phase 1-4 delivers satisfying core drone synthesis

---

## Phase 6: Perfboard Transition (Weeks 11-16) - Future

**Status**: Not Started
**Goal**: Permanent build with all 22 controls, battery power, proper enclosure

**Prerequisites**: Phase 1-4 completely validated and working on breadboard

### Power System Components (NEW - not in Phase 1)
- [ ] Order 18650 Li-Ion battery 3000mAh protected cell ($8-12)
- [ ] Order TP4056 charging module with USB-C ($1-2)
- [ ] Order TPS63070 buck-boost converter IC + supporting components ($4-8)
  - 2.2µH inductor, 22µF input/output caps, feedback resistors
- [ ] Order AP2112K-3.3 LDO regulator + capacitors ($1-3)
- [ ] Order 1N5819 Schottky diode for backfeed protection ($0.50)
- [ ] Order SPST power switch panel-mount ($1-3)

### Additional Control Components
- [ ] Order 12 more potentiometers (total 22 for full design)
- [ ] Choose quality level: Budget Alpha ($10-15) vs Premium Bourns ($36-60)
- [ ] Order 3 more ADS1115 modules for dual I2C bus architecture (6 total)
- [ ] Order 22× knob caps with brass inserts ($17-44)

### Audio Output Upgrade (OPTIONAL - only if headphone amp desired)
- [ ] Order OPA1622 headphone amplifier IC ($7-9)
- [ ] Order supporting components (resistors, capacitors per datasheet)
- [ ] Order panel-mount 3.5mm TRS jack ($2-4)

### Perfboard Construction
- [ ] Design modular board layout (control, ADC, main controller, audio, power)
- [ ] Order FR-4 perfboard assortment 130×180mm + smaller sizes ($15-25)
- [ ] Build and test power board standalone (battery → 3.3V regulation)
- [ ] Build and test ESP32 board with dual I2C buses
- [ ] Build and test 6× ADS1115 ADC bank
- [ ] Build and test audio output board (DAC + optional headphone amp)
- [ ] Assemble control panel (22 pots mounted to front panel)
- [ ] Integrate all boards with ribbon cables
- [ ] Full system testing before enclosure closure

### Enclosure and Final Assembly
- [ ] Design/source enclosure 178×257×72mm (SP-404 size)
- [ ] Options: 3D print ($15-25), sourced AliExpress, Hammond box ($40-80)
- [ ] Drill 22× pot holes + display cutout + edge-mount connectors
- [ ] Mount all components to enclosure
- [ ] Final wiring dress and strain relief
- [ ] Battery runtime validation (should achieve 20+ hours)

**Estimated Phase 6 time**: 24-34 hours across 6 weeks
**Estimated Phase 6 cost**: $185-293 additional (beyond Phase 1 components)

---

## Success Criteria

### Phase 1 Complete When:
- ✅ Triple oscillator + sub-oscillator blend creates rich timbres
- ✅ State Variable Filter shapes tone musically
- ✅ All 8-10 pots respond smoothly with no jitter or dead zones
- ✅ Audio quality is clean with no crackling, noise, or glitches
- ✅ Can create evolving drone textures for 10+ minutes continuously

### Phase 2 Complete When:
- ✅ Oscilloscope shows real-time waveform at 30+ FPS
- ✅ Parameter overlay appears when adjusting pots, fades after 2 seconds
- ✅ Display updates don't cause audio interruptions

### Phase 6 Complete When:
- ✅ All 22 controls functional (full DESIGN.md specification)
- ✅ Battery provides 20+ hours runtime
- ✅ Pocket-portable form factor achieved (fits in hand/bag)
- ✅ Professional build quality (no loose wires, solid panel)

---

## Risk Mitigation

**Risk**: AliExpress components arrive DOA or defective
**Mitigation**: Order from sellers with >95% rating and 1000+ orders; budget extra time for replacements

**Risk**: Breadboard connections prove unreliable (intermittent audio)
**Mitigation**: Use quality breadboards; add solid ground plane under breadboard; minimize wire lengths

**Risk**: I2S audio has crackling/noise on breadboard
**Mitigation**: Proper decoupling caps; separate analog/digital sections; add 22Ω series resistors if needed

**Risk**: ESP32-S3 ADC noise (if using internal ADC instead of ADS1115)
**Mitigation**: Already using ADS1115 16-bit external ADCs - correct choice

**Risk**: Phase 1 sound isn't satisfying, invalidates full build
**Mitigation**: This is WHY we breadboard first - minimal $30 investment to validate concept

**Risk**: 256-sample buffer causes audio underruns
**Mitigation**: Increase to 512 samples (11.6ms latency still acceptable); optimize DSP code

---

## Current Status: Phase 0 - Ready to Order Components

**Next action**: Order Phase 1 components from AliExpress/budget suppliers (estimated $20-35)

**Timeline estimate**:
- Phase 0: 1 week (ordering + shipping wait)
- Phase 1-4: 6-8 weeks (incremental firmware development)
- Phase 5: Optional, 2-3 weeks if pursued
- Phase 6: Future decision based on Phase 1-4 success

**Total MVP cost** (Phase 1-4): $20-35 + owned components
**Total full build cost** (Phase 6): $233-378 including all phases
