# Documentation References

- https://docs.espressif.com/projects/rust/esp-hal/1.0.0-rc.0/esp32/esp_hal/i2s/master/struct.I2sTx.html
- https://github.com/esp-rs/esp-hal/blob/esp-hal-v1.0.0-rc.0/esp-hal/src/i2s/master.rs

# Hardware Wiring

- ESP32-S3 DevKitC → PCM5102: `GPIO7` → `BCK`, `GPIO8` → `LRCK`, `GPIO9` → `DATA`, module `VCC` on 5 V.

# Audio Notes

- PCM5102A expects standard I²S timing when FMT is strapped low; keep Philips framing and leave `tx_msb_shift` alone unless hardware says otherwise.
- Stream 24-bit payloads in 32-bit slots (`Data32Channel32`) at 44.1 kHz. Shift the signed sample left by eight bits so the DAC sees the data in the MSBs while the HAL/DMA stay in native endianness.
- Prime the DMA ring before calling `write_dma_circular_async`; freshly generated audio removes the risk that stale descriptor contents blend into the first few frames.
- Sixteen 128-frame chunks (~16 KiB total) keep ~46 ms of headroom at 44.1 kHz so the DMA never outruns the mixer. Trim toward 12 chunks only after profiling CPU load, and avoid eight chunks until the per-voice cost drops further.
- `push_with(|window| tone.write(window))` is safe as long as generators respect arbitrary byte-aligned slices. Avoid reinterpreting the window as words; fill it as `u8` audio frames.
- Keep `DMA_CHUNK_BYTES` a multiple of `FRAME_BYTES`; otherwise esp-hal panics with `misaligned DMA buffer suffix`.
- With 32-bit slots BCK sits at 64×Fs, which the PCM5102A datasheet recommends for three-wire mode. Re-measure BCK/LRCK whenever you touch the clock tree.
- Sine wave generation uses a fast polynomial approximation (Taylor series up to x^7) for speed on embedded hardware, reducing render time while maintaining acceptable audio quality.

# Software Architecture

- Workspace split:
  - `synth-core` (`core/`) holds audio config, mix buffer, voices, and engine logic. It is `#![no_std]` by default but unit tests run with `std` on the host.
  - `synth-firmware` (`firmware/`) depends on `synth-core` plus `esp-hal` peripherals, DMA wiring, and the async task entrypoint.
- `synth_firmware::wiring::init` claims peripherals, clocks, and GPIO wiring, handing back ready-to-use I²S TX and the primed DMA buffer.
- `synth_firmware::audio::dma` owns ring priming plus the async circular refill loop and keeps underrun safeguards in one spot.
- `synth_core::audio::mix_buffer` accumulates normalized stereo frames, enforces mix headroom, and packs 24-bit payloads into the 32-bit slots expected by the PCM5102A.
- `synth_core::synth::{voice, engine}` expose oscillator/envelope/modulation hooks and mix multiple voices per DMA chunk.
  Voices cache stereo pan gains whenever pan changes so the hot loop only multiplies, and the engine zero-fills DMA slices when idle to skip pointless mixing work.
- `synth_core::control` remains a stub so future MIDI/UI handlers can stay outside the audio hot path.
- Comprehensive test suite in `core/tests/` covers mix buffers, engine lifecycle, and voice components.
- Host-side unit tests: run `scripts/test.sh` (internally `cargo +stable test -p synth-core`).
- Quality assurance: run `scripts/preflight.sh` before major changes (tests, linting, build validation).
- See `TESTING.md` for complete testing strategy and current test structure.

# Project Values

- We highly value audio quality, best practices, speed, and efficiency.

# Process Guidance

- Update this file with new hardware findings or tuning steps so future changes keep the tone clean.
- Listening tests were not rerun in this refactor—queue them on hardware before the next release burn-in.
- Firmware build/flash helpers: `scripts/build.sh` (release build) and `scripts/run.sh` (flash + monitor via `cargo run`). Both source `scripts/env.sh`; run `source scripts/env.sh` manually if invoking Cargo commands yourself.
