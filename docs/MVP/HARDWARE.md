# Hardware Shopping List - Monophonic Drone Synthesizer

Complete parts list for MVP build (27-pot triple-oscillator design).

---

## Core Electronics

### Microcontroller
- **1× ESP32-S3 Development Board** (owned)
  - Dual-core Xtensa LX7, 240 MHz
  - Minimum 37 GPIO pins required
  - Recommended: ESP32-S3-DevKitC-1 N16R8
  - Must support I2S output

### Audio DAC
- **1× PCM5102A DAC Module** (owned)
  - I2S interface
  - Pre-built breakout board recommended (simplifies wiring)
  - 5V compatible
  - Stereo output (use mono or dual-mono configuration)
  - **Important:** Ground the SCK pin to prevent digital noise pickup

### Headphone Amplifier
- **1× TPA6130A2 Headphone Amplifier IC**
  - Low noise, low distortion
  - Drives 16Ω–600Ω headphones
  - Requires supporting components (see datasheet):\
    - 2× 10µF capacitors (input coupling)
    - 2× 220µF capacitors (output coupling)
    - 2× 0.1µF decoupling capacitors
    - Resistors for gain setting (values per datasheet)
  - Alternative: Pre-built TPA6130A2 breakout module

### Display
- **1× ILI9341 3.5" TFT LCD Display (320×240)**
  - SPI interface (4-wire)
  - 3.3V logic level
  - Landscape orientation (320 wide × 240 tall)
  - ~25-30mA power draw
  - Significantly more readable than 2.8" for oscilloscope visualization

---

## Power System

### Battery
- **1× 18650 Li-Ion Battery Cell**
  - 3.7V nominal voltage
  - 3000mAh capacity (minimum)
  - **Protected cell required** (built-in over-discharge protection)
  - Recommended brands: Samsung 25R, LG HG2, Panasonic NCR18650B

### Charging
- **1× TP4056 Li-Ion Charging Module**
  - USB-C variant (not micro-USB)
  - 1A charging current typical
  - Built-in charge/discharge protection
  - LED indicators for charge status
  - **Note:** Cannot operate load while charging (load-sharing limitation)

### Voltage Regulation

**Digital Power Rail (ESP32-S3 + Display):**
- **1× HT7833 3.3V Linear Voltage Regulator** (recommended) or LF33CV
  - 500mA output current
  - TO-220 package
  - 4µA quiescent current (HT7833) vs. 5mA (LF33)
  - Supporting components:
    - 1× 1000µF electrolytic capacitor (bulk capacitance for WiFi transients)
    - 2× 10µF electrolytic capacitors (input/output filtering)
    - 2× 0.1µF ceramic capacitors (decoupling)

**Analog Power Rail (DAC + Headphone Amp):**

**Option A: RC Filter (Budget, Simple)**
- **1× 220Ω resistor** (from digital rail to analog rail)
- **1× 220µF electrolytic capacitor** (analog rail filtering)
- Provides adequate noise isolation for most use cases

**Option B: Dedicated Ultra-Low-Noise Regulator (Premium)**
- **1× TPS7A4700 3.3V Ultra-Low-Noise Regulator** (~$3)
  - 4.17µVrms noise, 86dB PSRR @ 10kHz
  - TO-220 package
  - Supporting components:
    - 2× 10µF capacitors (input/output)
    - 2× 0.1µF ceramic capacitors (decoupling)

### Protection
- **1× 1N5819 Schottky Diode**
  - Prevents backfeed from battery to TP4056
  - Low forward voltage drop (~0.3V)
  - 1A current rating

### Power Control
- **1× SPST Toggle Switch** (Single-Pole Single-Throw)
  - Panel-mount
  - 3A minimum current rating
  - ON-OFF (power switch for entire device)

---

## User Controls

### Potentiometers
- **26× 10kΩ Linear Potentiometers** (B-taper)
  - Taper: Linear (for all synthesis parameters)
  - Shaft: 6mm diameter (standard for knobs)
  - Size: 16mm body diameter (quality, comfortable grip)
  - Mounting: Panel-mount (threaded bushing with nut)
  - Single-turn (270° rotation)
  - Common brands: Alpha, Bourns, TT Electronics

- **1× 10kΩ Logarithmic Potentiometer** (A-taper, for Volume control)
  - Same specs as above, but logarithmic taper
  - Human hearing perceives volume logarithmically

### Buttons
- **1× Tactile Push Button** (screen on/off toggle)
  - Panel-mount or PCB-mount with extension
  - Momentary switch (SPST-NO)
  - 6mm × 6mm or 12mm × 12mm (depending on panel design)

### Knob Caps
- **27× Knob Caps**
  - Davies 1900H style (pointer knob, black with white indicator line)
  - Shaft fit: 6mm (match potentiometer shaft diameter)
  - Diameter: 15–20mm recommended
  - Material: ABS plastic or aluminum (aluminum feels more premium)
  - Common sources: Love My Switches, Thonk, AliExpress

---

## Audio Output

### Connector
- **1× 3.5mm TRS Headphone Jack**
  - Panel-mount (threaded for front panel installation)
  - Stereo (TRS = Tip-Ring-Sleeve)
  - Standard: PJ-320A or similar

---

## Signal Path Components (Critical for Clean Audio)

### I2S Signal Conditioning
- **3× 22Ω Resistors** (1/4W, 5% tolerance)
  - Series resistors for I2S lines: BCLK, LRCK, DIN
  - **Critical:** Prevents crackling and pops by reducing ringing/reflections
  - Place close to PCM5102A DAC inputs

---

## Connectors & Mechanical

### USB Port
- **1× USB-C Port/Connector**
  - Already included in TP4056 module (no separate part needed)
  - Panel-mount USB-C extension cable (optional, for cleaner installation)

### Battery Holder
- **1× 18650 Battery Holder**
  - Single-cell holder with solder tabs or wire leads
  - PCB-mount or case-mount variant

### Wiring & Connectors
- **Hookup Wire** (22–24 AWG stranded)
  - Multiple colors for organization (red/black for power, various for signals)
  - Silicone-insulated recommended (flexible, heat-resistant)
  - ~5-10 meters total
- **Optional: JST-XH or Dupont Connectors** (for modular connections)

---

## Passive Components

### Resistors (1/4W, 1% Metal Film)

**Required quantities:**
- 3× 22Ω (I2S series resistors) ✓ Critical
- 1× 220Ω (analog RC filter, if using Option A) ✓ Critical
- Assorted resistor kit for TPA6130A2 gain setting and general use
  - Common values: 1kΩ, 4.7kΩ, 10kΩ, 100kΩ
  - Buy 10-20 of each common value

### Capacitors

**Electrolytic (aluminum, radial):**
- 1× 1000µF, 16V (ESP32 bulk capacitance) ✓ Critical
- 4× 220µF, 16V (TPA6130 output coupling, analog RC filter)
- 6× 10µF, 16V (LF33/HT7833 filtering, TPA6130 input coupling)

**Ceramic (X5R or X7R dielectric, MLCC):**
- 20× 0.1µF (100nF), 50V ✓ Critical
  - Decoupling for all IC power pins (place immediately at pins)
  - ESP32: 4-6 capacitors
  - PCM5102A: 2 capacitors
  - TPA6130A2: 2 capacitors
  - Display: 1 capacitor
  - Voltage regulators: 2-4 capacitors
  - **Buy extras** (lose-prone, critical for clean power)
- 4× 10µF ceramic, 16V (additional ESP32 filtering, regulator output)

### Other
- **Perfboard/Protoboard** (if not using custom PCB)
  - Double-sided recommended (easier grounding)
  - Size: ~15cm × 12cm minimum (accommodates ESP32, DAC, amp, regulators)
  - Alternatively: Multiple smaller boards connected via headers

---

## Enclosure (TBD - Design Phase)

**Target dimensions:** ~178mm (W) × 276mm (H) × 50-70mm (D)
- Portrait orientation (taller than wide, similar to Roland SP-404)

**Options to consider:**
- 3D printed custom enclosure (ABS/PETG, allows exact design)
- Hammond aluminum project box (modify for cutouts, professional finish)
- Laser-cut acrylic layers (maker aesthetic, see-through option)
- Hand-fabricated wood/metal case (artisan approach)

**Required cutouts:**
- 27× holes for potentiometer shafts (~7mm diameter, threaded bushings secure pots)
- 1× hole for tactile button (~6-12mm depending on button choice)
- 1× hole for power switch (~6-12mm)
- 1× hole for USB-C port (~9mm × 3.5mm slot, or circular if using extension cable)
- 1× hole for 3.5mm headphone jack (~6mm diameter)
- 1× rectangular cutout for 3.5" LCD display (~74mm × 56mm for screen area)

---

## Tools & Supplies (Not in BOM, but needed)

- Soldering iron + solder (lead-free or 60/40 rosin-core)
- Wire strippers
- Multimeter (verify connections, voltages, continuity)
- Small screwdrivers (Phillips, flathead)
- Drill + drill bits (for enclosure holes)
- Step drill bit or reamer (for panel-mount components, creates clean holes)
- Hot glue or epoxy (strain relief for wiring)
- Helping hands / PCB holder
- Flush cutters (trim component leads)
- Desoldering pump or wick (fix mistakes)
- Label maker or permanent marker (label pots on panel)

---

## Estimated Cost Breakdown (USD, Approximate)

| Category                      | Cost (USD) | Notes |
|-------------------------------|------------|-------|
| ESP32-S3 Dev Board            | $8–15      | Owned |
| PCM5102A Module               | $3–8       | Owned |
| TPA6130A2 + Components        | $5–10      | Or pre-built module $8-12 |
| ILI9341 3.5" Display          | $10–18     | Upgrade from 2.8" (+$2-3) |
| 18650 Battery (protected)     | $6–12      | Quality cell (Samsung/LG/Panasonic) |
| TP4056 USB-C Module           | $2–5       | With protection |
| HT7833 or LF33 + Components   | $3–6       | Add $3 for TPS7A4700 if using Option B |
| 27× Potentiometers            | $27–70     | $1-2.50 each (Alpha/Bourns) |
| 1× Button                     | $0.50–2    | Tactile switch |
| 27× Knob Caps                 | $20–65     | $0.75-2.50 each (plastic/aluminum) |
| Power Switch                  | $1–3       | SPST toggle |
| 3.5mm Headphone Jack          | $1–3       | Panel-mount TRS |
| Passives (resistors/caps)     | $8–15      | Buy variety pack + bulk 0.1µF |
| I2S Resistors (3× 22Ω)        | $0.50      | Critical for clean audio |
| Perfboard/Protoboard          | $4–10      | Or custom PCB ($20-40 for 5 boards) |
| Wire, connectors, misc        | $5–10      | Silicone hookup wire, heat shrink |
| Battery Holder                | $1–3       | 18650 single-cell |
| **Subtotal (electronics)**    | **$105–235** | Depends on component quality |
| **Enclosure (separate)**      | **$15–80** | 3D print ($15-25) to Hammond box ($40-80) |
| **TOTAL**                     | **$120–315** | Budget to premium build |

**Cost optimization tips:**
- **Budget build (~$120-150):** AliExpress for pots/knobs/caps, 3D printed enclosure, generic components
- **Mid-range build (~$180-220):** Mix of AliExpress + Mouser, Hammond box, quality pots (Alpha/Bourns)
- **Premium build (~$250-315):** Mouser/Digikey components, TPS7A4700 regulator, aluminum knobs, custom enclosure

---

## Recommended Suppliers

### USA (Fast Shipping, Premium Pricing)
- **Mouser Electronics** - Reliable, vast selection, excellent search
- **Digikey** - Similar to Mouser, good for passives in bulk
- **Adafruit** - Beginner-friendly modules, good documentation
- **SparkFun** - Similar to Adafruit
- **Amazon** - Fast shipping (Prime), mixed quality, verify seller

### International / Budget (Slow Shipping, Best Prices)
- **AliExpress** - Cheapest option, 2-4 week shipping, verify seller ratings
- **Banggood** - Similar to AliExpress
- **LCSC** - Good for passives, integrated with JLCPCB for PCB orders
- **Tayda Electronics** - Budget DIY components, good for pots/jacks

### Knobs & Hardware Specialty
- **Love My Switches** - High-quality knobs, switches (USA)
- **Thonk** - Synth DIY parts, based in UK, excellent selection
- **Small Bear Electronics** - DIY audio components (USA)

---

## GPIO Pin Allocation (ESP32-S3)

| Component            | Pins Required | Interface | Notes |
|----------------------|---------------|-----------|-------|
| 27× Potentiometers   | 27            | ADC       | ADC1/ADC2 channels |
| 1× Button            | 1             | GPIO      | Digital input with pull-up |
| ILI9341 Display      | 5             | SPI       | MOSI, CLK, CS, DC, RST |
| PCM5102A DAC         | 3             | I2S       | BCLK, LRCK, DIN |
| Battery Monitor      | 1             | ADC       | Voltage divider to ADC |
| **Total**            | **37**        |           | ESP32-S3 has ~45 usable GPIO |

**Pin assignment considerations:**
- Use ADC1 channels for pots if possible (ADC2 conflicts with WiFi if enabled)
- I2S pins: Use I2S0 peripheral, standard pins: GPIO16 (BCLK), GPIO17 (LRCK), GPIO18 (DIN)
- SPI pins for display: Use VSPI or HSPI, avoid pins used by flash (GPIO6-11)
- Reserve GPIO pins for future expansion (MIDI, CV, etc.)

---

## Assembly Notes

### Power Supply Best Practices
1. **Star grounding:** Connect all ground points (digital, analog, battery) at a single point near power input
2. **Separate analog/digital power:** Use RC filter (220Ω + 220µF) minimum, dedicated regulator preferred
3. **Decoupling placement:** 0.1µF capacitors immediately at IC power pins (< 1cm trace length)
4. **Bulk capacitance:** 1000µF electrolytic on ESP32 rail to handle WiFi current spikes (even if WiFi disabled)

### I2S Signal Integrity
1. **Series resistors:** Place 22Ω resistors close to PCM5102A inputs (BCLK, LRCK, DIN)
2. **Ground SCK pin:** On PCM5102A module, ground the SCK pin (prevents digital noise pickup)
3. **Short traces:** Keep I2S signal traces as short as possible, avoid running near noisy signals

### Potentiometer Wiring
1. **Twisted pairs:** Twist ADC signal wire with ground wire to reduce noise pickup
2. **Star ground pots:** Connect all pot ground pins to a single ground point (not daisy-chained)
3. **Shielded cable (optional):** For long pot runs (>10cm), use shielded cable with shield grounded at one end

### Battery Safety
1. **Never short 18650 cells** - Can cause fire/explosion
2. **Use protected cells** - Built-in over-discharge/over-charge protection
3. **Don't over-discharge** - TP4056 module handles this with protection circuit
4. **Charge supervision** - Don't leave charging unattended for first few cycles
5. **Cannot play while charging** - Disconnect load during charge (TP4056 limitation)

### Testing Protocol (Before Final Assembly)
1. **Test modules individually:**
   - ESP32-S3: Upload blink sketch, verify operation
   - PCM5102A: Test I2S output with simple sine wave
   - Display: Test SPI communication, draw test pattern
   - TP4056: Verify charging circuit with multimeter
2. **Test power supply:**
   - Verify 3.3V output under load (multimeter)
   - Check ripple with oscilloscope (if available)
   - Verify current draw matches expectations
3. **Test audio path:**
   - Generate test tone, verify output with headphones
   - Check for noise, clicks, pops (if present, check decoupling/I2S resistors)
4. **Test pot reading:**
   - Connect one pot, verify ADC reads 0-4095 smoothly
   - Verify no jitter or noise in readings

---

## Notes & Warnings

### Component Selection
- **Buy extras:** Order 10-20% more passives and small components (resistors/caps) in case of loss/damage
- **Verify voltages:** Ensure all modules are 3.3V compatible (ESP32-S3 is 3.3V logic, damage from 5V signals)
- **Check datasheets:** Verify pinouts and supporting components for TPA6130A2 and voltage regulators before ordering
- **Quality matters for pots:** Cheap potentiometers feel scratchy and wear out quickly; Alpha/Bourns worth the premium

### Critical Components (Don't Cheap Out)
- ✓ **18650 battery:** Use protected cells from reputable brands (Samsung/LG/Panasonic)
- ✓ **Voltage regulators:** HT7833 > LF33 for lower quiescent current
- ✓ **0.1µF ceramic caps:** Buy X5R or X7R dielectric (Y5V/Z5U lose capacitance under voltage)
- ✓ **22Ω I2S resistors:** Don't skip these - they eliminate audio crackling
- ✓ **Potentiometers:** Alpha or Bourns for reliability and tactile feel

### Common Mistakes to Avoid
- ❌ Using Y5V/Z5U ceramic capacitors (lose 80% capacitance under DC bias)
- ❌ Skipping I2S series resistors (causes crackling/pops)
- ❌ Not grounding SCK pin on PCM5102A (causes digital noise)
- ❌ Insufficient bulk capacitance on ESP32 rail (causes brown-outs during WiFi activity)
- ❌ Using linear taper for volume pot (logarithmic is correct for human hearing)
- ❌ Daisy-chaining pot grounds (creates ground loops, use star grounding)

---

## Future Expansion Considerations

If adding features post-MVP, consider ordering now:
- **I2C GPIO expander** (MCP23017) - Add 16 more GPIO if needed
- **SD card module** - Save/load patches (SPI interface)
- **MIDI input components** - 6N138 opto-isolator + DIN-5 jack + resistors
- **Larger battery** - 3500mAh 18650 (Sanyo NCR18650GA) or dual-cell configuration
- **Stereo output** - Second 3.5mm jack + stereo effects processing
- **Accelerometer** - ADXL345 ($2, I2C) for gesture control (rejected for MVP but easy to add)

---

**This list supports the 27-pot triple-oscillator design defined in DESIGN.md.**

**Total GPIO requirement: 37 pins (ESP32-S3 has ~45 usable, providing 8-pin headroom for expansion)**
