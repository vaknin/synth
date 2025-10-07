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
- **CRITICAL**: Never call Slew.value() at audio rate (inside per-sample loops). Update slewed parameters once per buffer, cache the values, use cached values in the sample loop. Audio rate = 44.1kHz (per sample), Buffer rate = ~172Hz (per buffer), Control rate = 50Hz (ADC polls). Slew is for control→buffer rate smoothing only.