# Hardware Shopping List - Monophonic Drone Synthesizer

Complete parts list for MVP build (22-pot pocket-portable design with flawless precision ADC).

---

## Core Electronics

### Microcontroller
- **1× ESP32-S3 Development Board** (owned)
  - Dual-core Xtensa LX7, 240 MHz
  - Minimum 16 GPIO pins required (29 pins spare for expansion)
  - Recommended: ESP32-S3-DevKitC-1 N16R8
  - Must support I2S output and dual I2C buses

### Audio DAC
- **1× PCM5102A DAC Module** (owned)
  - I2S interface, 24-bit support @ 44.1kHz
  - Pre-built breakout board recommended (simplifies wiring)
  - 112dB SNR, excellent audio quality
  - Stereo output (use mono or dual-mono configuration)
  - **Important:** Ground the SCK pin to prevent digital noise pickup
  - 3× 22Ω series resistors on I2S lines (BCLK, LRCK, DIN) for signal integrity

### Headphone Amplifier
- **1× OPA1622 Headphone Amplifier IC**
  - Audiophile-grade: 119dB SNR (21dB better than TPA6130A2)
  - Ultra-low noise, ultra-low distortion
  - Drives 16Ω–600Ω headphones, 119mW output power
  - Low power consumption: 2.6mA per channel
  - Requires supporting components (see datasheet):
    - 2× 10µF capacitors (input coupling)
    - 2× 220µF capacitors (output coupling)
    - 2× 0.1µF decoupling capacitors
    - 1× 100kΩ + 1× 1µF (RC delay for pop suppression)
    - Resistors for gain setting (values per datasheet)

### Display
- **1× SSD1351 1.5" OLED Display (128×128 RGB)**
  - SPI interface (7-wire: MOSI, CLK, CS, DC, RST, VCC, GND)
  - 3.3V logic level
  - 121 PPI pixel density
  - ~10-15mA power draw (content-dependent, mostly-black scope display)
  - **Infinite contrast ratio** (true blacks), perfect for oscilloscope aesthetic
  - 0.1ms pixel response time (no ghosting, smooth waveform animation)

---

## Power System

### Battery
- **1× 18650 Li-Ion Battery Cell**
  - 3.7V nominal voltage
  - 3000mAh capacity (minimum) - Provides **20+ hours runtime**
  - **Protected cell required** (built-in over-discharge protection)
  - Recommended brands: Samsung 25R, LG HG2, Panasonic NCR18650B

### Charging
- **1× BQ24075 Li-Ion Charging IC** (upgraded from TP4056)
  - USB-C compatible
  - 1A charging current
  - **True load-sharing capability** - can play while charging!
  - Built-in charge/discharge protection
  - LED indicators for charge status
  - Solves TP4056's limitation (cannot play while charging)

### Battery Monitoring
- **1× MAX17043 Fuel Gauge IC** (I2C)
  - Accurate battery percentage (±1% vs ±15% for voltage divider)
  - ModelGauge algorithm compensates for discharge curve nonlinearity
  - I2C interface, shares bus with ADS1015 ADCs
  - Low-battery interrupt capability
  - Superior user experience vs simple voltage monitoring

### Voltage Regulation

**Buck-Boost + LDO Architecture (Optimized for Maximum Battery Utilization):**

- **1× TPS63070 Buck-Boost Converter** (VSON-10 package)
  - **Function:** Maintains stable 3.6V output across entire 18650 discharge curve (4.2V → 3.0V)
  - **Efficiency:** 85-95% typical (far superior to linear regulators)
  - **Critical benefit:** Unlocks 95% of battery capacity (2850mAh usable vs 600mAh with dual LDO cascade)
  - **Why essential:** Dual LDO cascade has 500mV total dropout, shuts down at 3.8V battery = only 4hr runtime
  - **Supporting components (per TI datasheet):**
    - 1× 2.2µH inductor (Coilcraft XAL or similar, rated for 2A+)
    - 2× 22µF ceramic input capacitors (X5R/X7R, low ESR)
    - 2× 22µF ceramic output capacitors (X5R/X7R, low ESR)
    - 2× feedback resistors for 3.6V output (values per datasheet calculation)
    - 1× 100kΩ enable pull-up resistor

- **1× AP2112K-3.3 LDO Voltage Regulator** (SOT-23-5 package, single stage)
  - **Function:** Final regulation from 3.6V → 3.3V for all system components
  - **Dropout:** 250mV (3.6V input - 3.3V output = 300mV margin, safe operation)
  - **Provides:** 70dB PSRR at audio frequencies, clean power for analog/digital
  - **Supporting components:**
    - 1× 1µF ceramic input capacitor (X5R/X7R)
    - 1× 2.2µF ceramic output capacitor (X5R/X7R)
    - 1× 1000µF electrolytic bulk capacitor on output (ESP32 transient handling)
    - 2× 0.1µF ceramic decoupling capacitors
  - **Separate analog/digital power filtering:**
    - Local 10Ω + 100µF RC filters at PCM5102A DAC and OPA1622 amp power pins
    - Provides >65dB PSRR, <-100dB noise floor

**Why buck-boost + LDO vs dual LDO cascade:**
- Dual LDO cascade: 500mV dropout = shuts down at 3.8V = only 20% battery usable = **4 hour runtime**
- Buck-boost + LDO: Operates down to 3.0V = 95% battery usable = **20+ hour runtime**
- Added cost: Only $3-4 in components (TPS63070 + inductor + capacitors)
- **500% runtime increase** for minimal cost and complexity
- Maintains studio-grade audio quality via final LDO stage

### Protection
- **1× 1N5819 Schottky Diode**
  - Prevents backfeed from battery to BQ24075
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
- **22× 10kΩ Linear Potentiometers** (B-taper, **16mm body diameter**)
  - **All linear taper** (including Volume - logarithmic scaling done in software)
  - Shaft: 6mm diameter (standard for knobs)
  - **Body size: 16mm diameter** (industry standard for professional synth control surfaces)
  - Mounting: Panel-mount (threaded bushing with nut)
  - Single-turn (270° rotation)
  - **Recommended: Bourns PTV09A-4015F-B103** ($1.50-2 each)
    - Conductive plastic element (smooth, quiet operation)
    - 15,000+ cycle life minimum
    - Sealed construction
    - Available through Mouser/Digikey
  - **Budget alternative: Alpha RD901F 16mm** ($0.80-1.20 each)
    - Adequate quality, somewhat scratchy after wear
    - Still far better than no-name brands
  - **Note:** Alps RK09K has only 9.8mm body (too small for comfortable use), not recommended

### ADC Interface (Critical for Precision)
- **6× ADS1115 16-bit I2C ADC** (MSOP-10 or DIP-10 package)
  - 4 channels per chip × 6 chips = 24 channels (using 22 for pots)
  - **Why ADS1115 16-bit?**
    - ESP32-S3 internal ADC has ±7 LSB DNL with dead zones at 0-0.1V and 3.0-3.16V
    - ADS1115 provides **±0.5 LSB DNL across entire 0-3.3V range** (true 16-bit linearity)
    - **65,536 resolution steps** vs 4,096 for 12-bit ADS1015 (16× smoother parameter sweeps)
    - Eliminates audible stepping artifacts on fast filter cutoff sweeps
    - Professional audio standard - pro control surfaces use 16-bit minimum
    - Same cost as ADS1015 (~$1.20 each)
  - **Dual I2C bus distribution:**
    - I2C Bus 0: 4× ADS1115 (addresses 0x48, 0x49, 0x4A, 0x4B) → 16 pots
    - I2C Bus 1: 2× ADS1115 (addresses 0x48, 0x49) → 6 pots
  - **Scan time:** ~25.5ms for all 22 pots @ 860 SPS (39Hz update rate)
    - Total control-to-audio latency: ~61ms (25.5ms ADC + 30ms smoothing + 5.8ms buffer)
    - Well within acceptable range (<100ms threshold for manual controls)
    - Professional synths operate in 10-30ms range for manual controls
  - **I2C pull-up resistors required:**
    - Bus 0: 2× 2.2kΩ resistors (SDA, SCL) - 1% tolerance, 1/8W
    - Bus 1: 2× 3.3kΩ or 4.7kΩ resistors (SDA, SCL) - 1% tolerance, 1/8W
  - **Silky-smooth control:** No jitter, no dead zones, professional studio-grade precision
  - Cost: 6× $1.20 = $7.20 total

### Knob Caps
- **22× Knob Caps**
  - Davies 1900H style with **brass insert** (pointer knob, black with white indicator line)
  - Shaft fit: 6mm (match potentiometer shaft diameter)
  - **Diameter: 15-18mm** (optimized for 25mm pot spacing, provides 7-10mm finger clearance)
  - Material: ABS plastic with brass insert (prevents stripping, better durability)
  - **Critical:** Plastic-only versions strip after 20-30 uses - always specify brass insert
  - Common sources: Love My Switches, Thonk, AliExpress
  - **Note:** Screen toggle button removed from design (always-on OLED, 20+ hour battery accommodates)

---

## Audio Output

### Connector
- **1× 3.5mm TRS Headphone Jack** (panel-mount, **separate from PCM5102A module**)
  - **Type:** PJ-320A or PJ-307 (3-pole TRS, threaded bushing with hex nut)
  - **Mounting:** Edge-mounted on enclosure **bottom-left edge**
  - **Hole size:** 6mm diameter
  - **Cost:** $1-3
  - **Critical:** The 3.5mm jack on the PCM5102A module is NOT used
    - Module jack outputs line-level (~2V, too weak for headphones)
    - Signal chain: PCM5102A DAC → OPA1622 amp → **Panel-mount jack**
    - Wiring: Solder from PCM5102A LOUT/ROUT/AGND pads → OPA1622 circuit → panel jack
  - Stereo (TRS = Tip-Left, Ring-Right, Sleeve-Ground)

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

**Total count: ~75-90 passive components critical for proper operation**

### Resistors (1/4W, 1% Metal Film unless otherwise noted)

**I2S signal conditioning (3 total) - CRITICAL:**
- 3× 22Ω (1/4W, 5% tolerance) - BCLK, LRCK, DIN lines (placed at ESP32 output pins)

**I2C pull-up resistors (4 total) - CRITICAL:**
- 2× 2.2kΩ (1% tolerance, 1/8W) - Bus 0 SDA and SCL
- 2× 3.3kΩ or 4.7kΩ (1% tolerance, 1/8W) - Bus 1 SDA and SCL

**OPA1622 amplifier circuit (4 resistors):**
- 2× 499Ω (0.1% metal film) - Input resistors
- 2× 768Ω (0.1% metal film) - Feedback resistors

**Audio output protection (2 total):**
- 2× 100Ω (1/4W) - Current limiting on headphone jack outputs

**Local power filtering (2 total):**
- 2× 10Ω (1/4W) - RC filters at DAC/amp power pins

**TPS63070 buck-boost (2+ total):**
- 2× feedback resistors per TI datasheet for 3.6V output
- 1× 100kΩ enable pull-up resistor

**Assorted for general use:**
- Common values: 1kΩ, 4.7kΩ, 10kΩ, 100kΩ
- Buy 10-20 of each common value

### Capacitors

**CRITICAL: Use only X5R or X7R ceramic capacitors (NOT Y5V/Z5U which lose 80% capacitance under DC bias)**

**Decoupling capacitors (32-36 total) - ESSENTIAL FOR STABILITY:**

**0.1µF ceramic (X5R/X7R, 50V) - 25-30 total:**
- ESP32-S3: 6-8 capacitors (distributed across digital power pins, <10mm from IC)
- 6× ADS1115 ADCs: 6 capacitors (1 per chip, within 10mm of VDD pin)
- PCM5102A DAC: 2-3 capacitors
- SSD1351 OLED: 1 capacitor
- MAX17043 fuel gauge: 1 capacitor
- AP2112K LDO: 2 capacitors
- TPS63070 buck-boost: 2 capacitors (per datasheet)
- OPA1622 amp: 4 capacitors (power supply bypass)
- **Buy 30-40 total** (lose-prone, absolutely critical)

**10µF ceramic (X5R/X7R, 25V) - 6-8 total:**
- ESP32-S3: 2 capacitors (1× main power, 1× VDD3P3_CPU)
- 6× ADS1115 shared bulk: 1-2 capacitors
- PCM5102A DAC: 1 capacitor (analog supply bulk)

**22µF ceramic (X5R/X7R, 25V) - 4 total:**
- TPS63070 buck-boost: 2× input, 2× output (low ESR required)

**Electrolytic (aluminum, radial) - 6-8 total:**
- 1× 1000µF, 16V (ESP32 bulk capacitance on AP2112K output) ✓ CRITICAL
- 2× 100µF, 16V (local DAC/amp power filtering with 10Ω resistors)
- 2× 220µF, 16V (OPA1622 output coupling capacitors)
- 1× 1µF, 16V (AP2112K input capacitor)
- 1× 2.2µF, 16V (AP2112K output capacitor)

**Film/C0G capacitors (2 total):**
- 2× 470pF (OPA1622 bandwidth limiting capacitors, low distortion)

### Inductors

**TPS63070 buck-boost (1 total):**
- 1× 2.2µH power inductor (rated for 2A+, low DCR)
  - Coilcraft XAL series or similar
  - Must handle switching frequency (typically 1-2 MHz)
  - Size: 5mm × 5mm or larger for proper current handling

### ESD Protection & Safety

**USB-C ESD protection (1 total) - CRITICAL:**
- 1× USBLC6-2SC6 integrated ESD suppressor (or equivalent)
  - Place within 5mm of USB-C connector
  - Protects USB data lines and power from electrostatic discharge

**Battery protection (1 total) - CRITICAL:**
- 1× 1.5A PTC resettable fuse
  - Placed on battery output
  - Provides overcurrent protection (BQ24075 has built-in protection, but redundancy is good)

### PCB Requirements

**This design requires a custom 4-layer PCB - perfboard construction is impossible:**

- **Layer stackup:**
  - Layer 1 (Top): Components and primary signal routing
  - Layer 2 (Inner): Solid ground plane (no gaps except analog/digital boundary)
  - Layer 3 (Inner): Power distribution plus secondary signals
  - Layer 4 (Bottom): Additional components and routing
- **Thickness:** 1.6mm (standard, optimal rigidity for panel-mount 16mm pots)
- **Why 4-layer is mandatory:**
  - Continuous ground plane essential for 240MHz ESP32-S3 RF containment
  - Dual I2C buses (22 pots + 6 ADCs) need controlled impedance
  - I2S audio at 2.8MHz requires proper trace routing with guard traces
  - AGND/DGND split requires separate ground zones with single-point connection
  - High-frequency SPI OLED (10-15MHz) needs isolation from audio path
- **Cost:** $50-100 for 5 prototype boards from JLCPCB ($10-20 each)
  - Production volumes of 10+ reduce cost to $5-8/board

---

## Enclosure

**Target dimensions:** **178mm (W) × 257mm (H) × 72mm (D)** (Roland SP-404 size)
- **Pocket-portable, portrait orientation** (taller than wide, fits in hand/pocket)
- Form factor priority: truly portable, not in-between lap/handheld

**Options to consider:**
- 3D printed custom enclosure (ABS/PETG, allows exact design, ~$15-25)
- Sourced enclosure from AliExpress (cost-effective, modify for cutouts)
- Hammond aluminum project box (modify for cutouts, professional finish, $40-80)
- Laser-cut acrylic layers (maker aesthetic, see-through option)

**Required cutouts:**
- 22× holes for potentiometer shafts (~7mm diameter, threaded bushings secure pots)
  - **Layout:** 4 columns × 6 rows at 25mm horizontal, 23mm vertical spacing
- 1× rectangular cutout for 1.5" OLED display (~40mm × 40mm for screen area, top-center)

**Edge-mounted connectors:**
- **Top-left edge:** 1× 3.5mm headphone jack hole (~6mm diameter)
- **Top-right edge:** 1× power switch hole (~6-12mm)
- **Bottom-left edge:** (3.5mm jack - same as top-left, user preference)
- **Bottom-right edge:** 1× USB-C port hole (~9mm × 3.5mm slot)
- **Note:** Screen toggle button removed from design (always-on OLED)

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
| PCM5102A Module               | $3–8       | Owned, 24-bit support |
| OPA1622 Headphone Amp + Comp  | $6–12      | Audiophile-grade 119dB SNR |
| SSD1351 1.5" OLED Display     | $15–20     | Infinite contrast, true blacks |
| 18650 Battery (protected)     | $6–12      | Quality cell (Samsung/LG/Panasonic), 20+ hours |
| BQ24075 USB-C Charger IC      | $3–5       | Play-while-charging capability |
| TPS63070 Buck-Boost + Comp    | $4–8       | Critical: Unlocks 95% battery (20+ hr runtime) |
| 1× AP2112K-3.3 LDO + Comp     | $1–3       | Single stage after buck-boost |
| 6× ADS1115 16-bit ADC         | $7–10      | Pro-grade precision, 65,536 steps, silky-smooth |
| MAX17043 Fuel Gauge           | $2–4       | Accurate battery % (I2C) |
| 22× Potentiometers (16mm)     | $33–44     | $1.50-2 each (Bourns PTV09A recommended) |
| 22× Knob Caps (brass insert)  | $17–44     | $0.75-2 each, 15-18mm diameter |
| Power Switch                  | $1–3       | SPST toggle, edge-mount |
| Panel-mount 3.5mm Jack        | $1–3       | TRS, separate from PCM5102A module |
| Passives (resistors/caps)     | $15–25     | ~75-90 components: decoupling, I2C pull-ups, ESD |
| I2S Resistors (3× 22Ω)        | Included   | In passives count |
| I2C Pull-ups (4× resistors)   | Included   | In passives count |
| ESD Protection + PTC Fuse     | $2–4       | USBLC6-2SC6 + 1.5A fuse |
| 2.2µH Inductor (buck-boost)   | $1–2       | Coilcraft XAL or similar |
| 4-Layer Custom PCB (÷5)       | $10–20     | $50-100 for 5 boards from JLCPCB (mandatory) |
| Wire, connectors, misc        | $5–10      | Silicone hookup wire, heat shrink |
| Battery Holder                | $1–3       | 18650 single-cell |
| **Subtotal (electronics)**    | **$135–245** | Mid-range quality components |
| **Enclosure (separate)**      | **$15–80** | 3D print ($15-25) or Hammond ($40-80) |
| **TOTAL**                     | **$150–325** | Pocket-portable professional build |

**Cost optimization tips:**
- **Budget build (~$150-180):** AliExpress for display/knobs, 3D printed enclosure, Alpha 16mm pots
- **Mid-range build (~$210-260):** Mix of AliExpress + Mouser, sourced enclosure, Bourns pots (recommended)
- **Premium build (~$280-325):** Mouser/Digikey components, Bourns pots, Hammond box, aluminum knobs

**Note:** This design prioritizes uncompromising audio quality and functionality:
- 24-bit @ 44.1kHz audio (256-sample buffer)
- OPA1622 audiophile amp (119dB SNR)
- ADS1115 16-bit ADCs (65,536 steps, pro-grade precision)
- TPS63070 buck-boost (20+ hour runtime, not 4 hours)
- 4-layer PCB with proper ground plane (mandatory for functionality)
- SPI OLED EMI mitigation for clean audio

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

| Component                | Pins Required | Interface | Notes                          |
|--------------------------|---------------|-----------|--------------------------------|
| 22× Pots (via ADS1115)   | 4             | I2C       | 2× I2C buses (SDA+SCL each)    |
| MAX17043 Fuel Gauge      | 0             | I2C       | Shares I2C Bus 0 with ADS1115  |
| SSD1351 OLED Display     | 7             | SPI       | MOSI, CLK, CS, DC, RST, VCC, GND |
| PCM5102A DAC             | 3             | I2S       | BCLK, LRCK, DIN                |
| OPA1622 Amp Enable       | 1             | GPIO      | Pop suppression RC delay       |
| **Total**                | **15**        |           | **30 pins spare for expansion**|

**Detailed pin assignment:**
- **I2C Bus 0** (GPIO1 SDA, GPIO2 SCL): 4× ADS1115 (addresses 0x48, 0x49, 0x4A, 0x4B) + MAX17043
- **I2C Bus 1** (GPIO3 SDA, GPIO4 SCL): 2× ADS1115 (addresses 0x48, 0x49)
- **I2S** (GPIO16 BCLK, GPIO17 LRCK, GPIO18 DIN): PCM5102A DAC (with 22Ω series resistors at ESP32 outputs)
- **SPI** (GPIO11 MOSI, GPIO12 CLK, GPIO10 CS, GPIO9 DC, GPIO8 RST): SSD1351 OLED
- **GPIO5**: OPA1622 enable (pop suppression with 100kΩ + 1µF RC delay)

**Notes:**
- Screen toggle button removed from design (GPIO6 now available for future expansion)
- SPI OLED uses reduced drive strength (GPIO_DRIVE_CAP_0) and 10-15MHz clock for EMI mitigation
- I2S lines have 22Ω series resistors at ESP32 outputs (reduces ringing/reflections)
- I2C buses use pull-up resistors: 2.2kΩ (Bus 0), 3.3-4.7kΩ (Bus 1)

ESP32-S3 has ~45 usable GPIO, providing substantial headroom for future features (MIDI, CV, etc.).

---

## Assembly Notes

### Power Supply Best Practices
1. **Star grounding:** Connect all ground points (digital, analog, battery) at a single point near power input
2. **Buck-boost + LDO architecture:**
   - TPS63070 maintains 3.6V across entire battery discharge (4.2V → 3.0V)
   - Single AP2112K-3.3V regulates final 3.3V for all components
   - Critical: Ensures 95% battery capacity utilization (20+ hour runtime)
3. **Decoupling placement:** 0.1µF capacitors immediately at IC power pins (< 1cm trace length, <10mm ideal)
4. **Bulk capacitance:** 1000µF electrolytic on ESP32 rail (AP2112K output) to handle transient current spikes
5. **Local filtering:** 10Ω + 100µF RC filters at PCM5102A DAC and OPA1622 amp power pins for additional noise rejection
6. **PCB ground plane:** Separate AGND (analog) and DGND (digital) zones, connected via single 0Ω resistor at power input

### I2S Signal Integrity & EMI Mitigation
1. **Series resistors:** Place 22Ω resistors close to ESP32 output pins (BCLK, LRCK, DIN), not at PCM5102A
2. **Ground SCK pin:** On PCM5102A module, ground the SCK pin (prevents digital noise pickup)
3. **Short traces:** Keep I2S signal traces as short as possible, route over DGND (not AGND, they're digital signals)
4. **Guard traces:** Flank I2S traces with GND traces on both sides to prevent coupling
5. **SPI OLED EMI mitigation (critical for audio quality):**
   - Reduce ESP32 GPIO drive strength: `gpio_set_drive_capability(GPIO_DRIVE_CAP_0)` (weakest)
   - Lower SPI clock to 10-15MHz (still 60+ FPS, ~40dB less EMI vs 40MHz)
   - Physical separation: ≥20mm between OLED and audio components (PCM5102A, OPA1622)
   - Route audio traces away from SPI signals, use ground guard traces
   - Keep SPI traces short and direct to minimize antenna effect

### Potentiometer Wiring
1. **I2C bus routing:** Keep I2C traces (SDA, SCL) short, use twisted pair if >10cm
2. **Star ground:** Connect all pot ground pins to ADS1115 ground, then star-ground to main GND
3. **ADC addressing:** Verify ADDR pin connections for correct I2C addresses (0x48-0x4B)
4. **Pull-up resistors:** Install 2.2kΩ on Bus 0, 3.3-4.7kΩ on Bus 1 (SDA and SCL each)
5. **No shielding needed:** ADS1115's differential inputs and I2C digital protocol are noise-resistant

### Battery Safety
1. **Never short 18650 cells** - Can cause fire/explosion
2. **Use protected cells** - Built-in over-discharge/over-charge protection
3. **Don't over-discharge** - BQ24075 handles protection with integrated circuitry
4. **Charge supervision** - Don't leave charging unattended for first few cycles
5. **CAN play while charging** - BQ24075 provides true load-sharing (advantage over TP4056)

### Testing Protocol (Before Final Assembly)
1. **Test modules individually:**
   - ESP32-S3: Upload blink sketch, verify operation
   - PCM5102A: Test I2S output (24-bit @ 44.1kHz) with simple sine wave
   - SSD1351 OLED: Test SPI communication, draw test pattern (check true blacks)
   - BQ24075: Verify charging circuit with multimeter, test load-sharing
   - MAX17043: Verify I2C communication, read battery %
2. **Test power supply:**
   - Verify 3.3V output on both digital and analog rails (multimeter)
   - Check ripple with oscilloscope (if available, should be <10mV)
   - Verify current draw: ~70mA ESP32 + 20mA DAC/amp + 15mA OLED = ~105mA typical
3. **Test audio path:**
   - Generate test tone (24-bit), verify output with headphones
   - Check for noise, clicks, pops (if present, verify I2S 22Ω resistors + decoupling)
   - Test pop suppression: power on/off should be silent (hardware RC delay working)
4. **Test ADC (ADS1015) reading:**
   - Connect one pot to ADS1015, verify I2C communication (addresses 0x48-0x4B)
   - Sweep pot, verify readings 0-4095 across ENTIRE range (no dead zones at extremes)
   - Verify stable readings (±1 LSB jitter acceptable, not ±50 like ESP32 ADC)

---

## Notes & Warnings

### Component Selection
- **Buy extras:** Order 10-20% more passives and small components (resistors/caps) in case of loss/damage
- **Verify voltages:** Ensure all modules are 3.3V compatible (ESP32-S3 is 3.3V logic, damage from 5V signals)
- **Check datasheets:** Verify pinouts and supporting components for TPA6130A2 and voltage regulators before ordering
- **Quality matters for pots:** Cheap potentiometers feel scratchy and wear out quickly; Alpha/Bourns worth the premium

### Critical Components (Don't Cheap Out)
- ✓ **18650 battery:** Use protected cells from reputable brands (Samsung/LG/Panasonic)
- ✓ **ADS1015 ADCs:** Don't substitute with ESP32 internal ADC - dead zones will ruin UX
- ✓ **Voltage regulators:** AP2112K dual cascade mandatory, RC filter causes voltage collapse
- ✓ **0.1µF ceramic caps:** Buy X5R or X7R dielectric (Y5V/Z5U lose capacitance under voltage)
- ✓ **22Ω I2S resistors:** Don't skip these - they eliminate audio crackling
- ✓ **Potentiometers:** Alps RK09K recommended (Alpha acceptable, avoid no-name brands)
- ✓ **Knob caps with brass insert:** Plastic-only versions strip after 20-30 uses

### Common Mistakes to Avoid
- ❌ Using Y5V/Z5U ceramic capacitors (lose 80% capacitance under DC bias)
- ❌ Skipping I2S series resistors (causes crackling/pops)
- ❌ Not grounding SCK pin on PCM5102A (causes digital noise)
- ❌ Insufficient bulk capacitance on ESP32 rail (causes brown-outs)
- ❌ Using ESP32 internal ADC instead of ADS1015 (dead zones at 0-0.1V and 3-3.16V)
- ❌ RC filter power supply (220Ω causes 8.8V drop - use dual AP2112K cascade)
- ❌ Forgetting pop suppression RC delay on OPA1622 enable (loud clicks damage headphones)

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

**This list supports the 22-pot pocket-portable design with professional-grade precision defined in DESIGN.md.**

**Total GPIO requirement: 15 pins (ESP32-S3 has ~45 usable, providing 30-pin headroom for expansion)**

**Key upgrades from initial design and expert feedback:**
- ✅ **TPS63070 buck-boost** replaces dual LDO cascade - unlocks 95% battery capacity (20+ hours not 4)
- ✅ **6× ADS1115 16-bit ADCs** eliminate ESP32's dead zones - 65,536 steps for silky-smooth sweeps
- ✅ **4-layer PCB mandatory** with ground plane - essential for RF containment and audio quality
- ✅ **16mm potentiometers** (Bourns PTV09A) at 25mm spacing - professional tactile feel
- ✅ **4×6 importance-weighted layout** - primary controls on left, secondary on right
- ✅ **Panel-mount 3.5mm jack** required - PCM5102A module jack is line-level only
- ✅ **SPI OLED EMI mitigation** - reduced drive, lower clock, guard traces, 20mm separation
- ✅ **BQ24075 charger** enables play-while-charging (TP4056 cannot do this)
- ✅ **MAX17043 fuel gauge** provides ±1% accurate battery % (vs ±15% voltage divider)
- ✅ **OPA1622 amp** delivers 119dB SNR, 21dB better than TPA6130A2
- ✅ **256-sample buffer** @ 24-bit/44.1kHz - 5.8ms latency, fully utilizes audiophile signal path
- ✅ **~75-90 passive components** detailed - decoupling, I2C pull-ups, ESD protection, safety
- ✅ **Pocket-portable 178×257×72mm** form factor (SP-404 size)
- ✅ **Screen button removed** - always-on OLED, 20+ hour battery accommodates continuous visualization
