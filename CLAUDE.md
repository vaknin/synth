# ESP32-S3 Synthesizer

Audio synthesizer with oscillators, filters, LFOs, and real-time OLED waveform visualization.

# Documentation

**Architecture:**
- `docs/mvp/ARCHITECTURE.md` - Processing rates, optimizations, patterns

**Dual-Core:**
- Core 0: Audio rendering (hotpath)
- Core 1: ADC + display (UI)
- Use `StaticCell<Stack<SIZE>>` for Core 1 stack
- Cross-core communication:
  - Embassy Channel (MPSC): Pot messages (Core 1 → Core 0)
  - heapless::spsc::Queue: Waveform data (Core 0 → Core 1)
  - Pattern: Message-passing over shared mutable state

**GPIO (Phase 1):**
- I2S: GPIO35/36/37 (safe on N16R8)
- Pots: GPIO1 (cutoff), GPIO2 (resonance)
- Display: GPIO9-13 (SPI2, 20 MHz, SSD1351 128×128 OLED)
  - Time-based scrolling: constant velocity matching decimated sample rate
  - Scroll rate auto-derived from DISPLAY_REFRESH_RATE_HZ (configurable: 30/60/120Hz)
  - Zoom (TIME_WINDOW_MS) independent of refresh rate
  - Initialize scroll at half-buffer (512 samples) for safe separation

# Project Guidelines

**Values:** Audio quality, speed, efficiency, modern idiomatic Rust

**Design Priorities (in order):**
1. Portability (pocket-sized)
2. User Experience (fun, creative)
3. Audio Quality
4. CPU Budget (optimize smartly)
5. Cost (budget-conscious)

**Development:**
- Hardware: ESP32-S3 (dual-core XTensa LX7)
- Build: `./scripts/build.sh`
- After each task/phase: Run `cargo clippy` and `cargo fmt`
- Crates: Use latest versions
- Code style: Elegant, efficient, public fields over getters
- Questions: Number them, attach recommendations (based on audio/hardware best practices)
- Comments/docs: Minimal, concise, technical. No verbose explanations or marketing language
- Config constants: All constants in `src/config.rs` must be derived from other constants where possible (no magic numbers). Ensure derivations remain valid when base constants change (e.g., DISPLAY_REFRESH_RATE_HZ is configurable)
