# Documentation References

- https://docs.espressif.com/projects/rust/esp-hal/1.0.0-rc.0/esp32/esp_hal/i2s/master/struct.I2sTx.html
- https://github.com/esp-rs/esp-hal/blob/esp-hal-v1.0.0-rc.0/esp-hal/src/i2s/master.rs
- Local source code lives at: "~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/esp-hal-1.0.0-rc.0/"
- General I2S note for the ESP-IDF (C) version of ESP32-S3:
  https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/api-reference/peripherals/i2s.html

# Project Values

- We highly value audio quality, best practices, speed, and efficiency.
- Always use modern and idiomatic Rust
- Hot path optimization is critical: use `#[inline]` on small, frequently-called functions (audio callback @ 44.1kHz)

# Guidelines
- Always build with ./scripts/build.sh
- When adding new crates, make sure they're the latest version possible
- We're running the project on the ESP32-S3 - a dual-core XTensa LX7 MCU
- please number your questions so it's easier to discuss and reply. attach your recommandation to each question — based on software, hardware and digital audio best-pracrices.
- when suggesting code to implement, think first: "can we make it more elegant and efficient? are we using best-practices to maximize performence and audio-quality?"
- opt for using public fields instead of getter methods where it makes sense, we always prefer idiomatic and elegant Rust code

## Processing Rate Architecture

**Three Processing Rates:**
- **Control Rate (50Hz)**: ADC pot polling - where parameter changes originate
- **Buffer Rate (~172Hz)**: DMA buffer callbacks - message consumption & coefficient caching
- **Audio Rate (44.1kHz)**: Per-sample processing - the super hot path

**Engine Rate Methods** (see `src/engine.rs`):
```rust
// BUFFER RATE: Called once per buffer (~172Hz)
fn consume_messages(&mut self)         // Drain pot message queue, set targets
fn update_buffer_parameters(&mut self) // Recalc filter coefficients (if dirty)

// AUDIO RATE: Called per sample (44.1kHz)
fn render_sample(&mut self) -> f32     // Mix → normalize → filter
```

**Smoothing Decision Rules:**

1. **Parameters that Multiply at Audio Rate** → **Use Slew (Audio Rate)**
   - **Why**: Direct multiplication without smoothing causes zipper noise
   - **Examples**: Oscillator pitch, oscillator level, mix normalization factor
   - **Pattern**: Call `.value()` in audio-rate code (e.g., inside `tick()` or `render_sample()`)
   - **Cost**: Minimal due to `settled` optimization (just branch check when stable)

2. **Filter Parameters** → **Direct (No Slew)**
   - **Why**: ADC multisampling (9×) provides sufficient smoothing. Dirty flag prevents redundant recalc.
   - **Examples**: Filter cutoff, filter resonance
   - **Pattern**: Direct assignment + dirty flag, coefficients cached at buffer rate
   - **Cost**: Zero smoothing overhead

3. **Modulation Sources** → **Generate Smooth (No Slew)**
   - **Why**: LFOs/Envelopes produce smooth signals natively
   - **Examples**: LFO output, ADSR envelope
   - **Pattern**: Generate at audio rate, apply to Slew targets at buffer rate
   - **Cost**: No additional smoothing

**Signal Flow:**
```
Pots (50Hz) → [9× multisampling + deadband] → Messages → Targets
                                                            ↓
Buffer Rate (172Hz): consume_messages() → Slew.set_target() or direct assignment
                                                            ↓
Audio Rate (44.1kHz): render_sample() → Slew.value() for smooth params
```