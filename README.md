# ESP32-S3 Drone Synthesizer

<p align="center">
  <img src="assets/image.png" alt="Synth Hardware" width="333">
</p>

**Pocket-portable monophonic drone synthesizer** for exploratory sound creation. Always-on continuous synthesis with real-time control, designed for completion.

## Core Features

- **Always-on drone synthesis** - Sculpt and morph a living, breathing tone
- **Triple oscillator** (Sine, Saw, Square) with independent detune
- **Multi-mode filter** (LP/HP/BP blend) with self-oscillating resonance
- **Professional audio** - 24-bit @ 44.1kHz, OPA1622 headphone amp (119dB SNR)
- **16-bit ADC precision** - 6× ADS1115, 65,536 steps, silky-smooth control
- **Live oscilloscope** - 1.5" OLED waveform visualization
- **Battery powered** - 20+ hours runtime, USB-C charging

## Quick Start

```bash
# Build firmware
./scripts/build.sh

# Run on device
./scripts/run.sh
```

## Documentation

- **[BUILD_PLAN.md](docs/mvp/BUILD_PLAN.md)** - Active MVP roadmap (breadboard/perfboard phases)
- **[Post-MVP Design](docs/post-mvp/DESIGN.md)** - Full 22-pot pocket-portable reference spec
- **[Hardware List](docs/post-mvp/HARDWARE.md)** - Complete component BOM and assembly notes

## Development

- **Platform**: ESP32-S3 (dual-core Xtensa LX7 @ 240 MHz)
- **Language**: Rust (esp-hal 1.0.0-rc.0)
- **Audio**: PCM5102A DAC via I2S
- **Display**: SSD1351 1.5" OLED (128×128 RGB, SPI)
- **Controls**: ADS1115 16-bit ADCs (dual I2C buses)

## Philosophy

Engineer's instrument with full parameter control, no hidden behavior, precise and predictable. Progressive complexity - start with one oscillator, add layers when ready. Completion over perfection.

---

Built with Rust 🦀 on ESP32-S3
