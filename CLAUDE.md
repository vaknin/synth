# ESP32-S3 Synthesizer

Audio synthesizer with oscillators, filters, LFOs, and real-time OLED waveform visualization.

# Current Status

**✅ WORKING (2025-10-30):**
- Async SPI + DMA with `embedded-hal-bus::ExclusiveDevice` (idiomatic single-device pattern)
- Hardware/wiring verified correct
- Test pattern renders successfully (13ms flush @ 20 MHz)
- **Step 1 COMPLETE**: Display task running as Embassy task on Core 0 ✅
- **Step 2 COMPLETE**: Display task running on Core 1 (cross-core async SPI working!) ✅
  - Key: `SpiDmaBus<Blocking>` is `Send`, `.into_async()` called ON Core 1
  - Previous BusWriteError avoided by correct async binding

**Next Steps (incremental, one at a time):**
1. ~~Move async display to embassy task on Core 0 (still single-core)~~ ✅ DONE
2. ~~Enable dual-core with display task on Core 1~~ ✅ DONE
3. Add I2S audio back on Core 0
4. Add ADC + pot task on Core 1
5. Restore full synthesizer functionality

**Rule: If any step fails, revert to previous working state. Never advance two steps.**

# Documentation

**References:**
- [ESP-HAL API (ESP32-S3)](https://docs.espressif.com/projects/rust/esp-hal/1.0.0/esp32s3/esp_hal/index.html) - Hardware abstraction layer
- `docs/mvp/ARCHITECTURE.md` - Processing rates, optimizations, patterns

**Dual-Core:**
- Core 0: Audio rendering (hotpath)
- Core 1: ADC + display (UI)
- Use `StaticCell<Stack<SIZE>>` for Core 1 stack
- Cross-core communication:
  - Embassy Channel (MPSC): Pot messages (Core 1 → Core 0)
  - Embassy Signal: Waveform data (Core 0 → Core 1, latest-value semantics)
  - Pattern: Message-passing over shared mutable state

**Embassy Task Spawning (Critical Pattern):**
- **Keep task parameters small** - spawner copies all params onto executor spawn stack during `.spawn()`
- ✅ Pass raw peripheral handles (thin types: `ADC1`, `GPIO1`, etc. - ~8 bytes each)
- ✅ Initialize heavy peripherals INSIDE tasks (on task's own stack)
- ❌ DON'T pass initialized peripherals with calibration data (e.g., `Adc<ADC1>` with curves ~4-8KB)
- Example:
  ```rust
  // WRONG: Heavy param copy during spawn (13KB)
  let (adc_bus, pins) = setup_adc(peripherals.ADC1, ...);
  spawner.spawn(pot_task(adc_bus, pins))  // ❌ Overflows executor spawn stack

  // RIGHT: Light param copy, heavy init inside task
  spawner.spawn(pot_task(peripherals.ADC1, peripherals.GPIO1))  // ✅ ~16 bytes
  // Inside pot_task:
  let (adc_bus, pins) = setup_adc(adc1, gpio1);  // Init on task's own stack
  ```
- Symptom of violation: `BusWriteError` or memory corruption during spawn, even with large stack sizes
- Root cause: Executor spawn stack frame is small, separate from task execution stack

**GPIO (Phase 1):**
- I2S: GPIO35/36/37 (safe on N16R8)
- Pots: GPIO1 (cutoff), GPIO2 (resonance)
- Display: GPIO9-13 (SPI2, 20 MHz, SSD1351 128×128 OLED)
  - **SPI Pattern**: `embedded-hal-bus::ExclusiveDevice` (not Embassy `shared_bus`) - idiomatic for single-device SPI
    - Direct SPI ownership, zero mutex overhead
    - CS auto-managed by ExclusiveDevice wrapper
    - Embassy `shared_bus::SpiDevice` only needed when multiple devices share one SPI bus
  - Display controls own read pointer (local_read_index), advances by WAVEFORM_SCROLL_RATE per frame
  - **CRITICAL**: local_read_index MUST initialize to 0.0 (same as audio write_index start)
    - Initializing to WAVEFORM_BUFFER_SIZE/2 causes display to read uninitialized zeros → straight line bug
  - Smooth scrolling: linear interpolation between adjacent samples using fractional position
  - Signal provides complete buffer snapshot (~127 Hz), display samples at refresh rate (30/60/120Hz configurable)
  - Zoom (TIME_WINDOW_MS) independent of refresh rate
  - **DMA Configuration:**
    - `dma_buffers!(4, 32768)` = minimal 4-byte RX, full 32KB TX buffer
    - ESP-HAL SPI DMA limit: 32736 bytes/transfer (32 bytes less than 32KB)
    - Framebuffer (128×128×2 = 32768 bytes) split into 32736 + 32 byte writes
    - Split write has zero overhead: 13ms @ 20 MHz (async), 17ms @ 15 MHz (blocking)
    - 40 MHz overclock: 7ms @ 37 MHz effective ⚡

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
- Diagnostic logging: Use sparse logs (startup + every 1-2 sec) to avoid UART choking; frequent logs can overwhelm serial output
