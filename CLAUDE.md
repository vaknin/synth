# Documentation References

- https://docs.espressif.com/projects/rust/esp-hal/1.0.0-rc.0/esp32/esp_hal/i2s/master/struct.I2sTx.html
- https://github.com/esp-rs/esp-hal/blob/esp-hal-v1.0.0-rc.0/esp-hal/src/i2s/master.rs
- Local source code lives at: "~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/esp-hal-1.0.0-rc.0/"
- General I2S note for the ESP-IDF (C) version of ESP32-S3:
  https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/api-reference/peripherals/i2s.html

# Project Values

- We highly value audio quality, best practices, speed, and efficiency.
- Always use modern and idiomatic Rust

# Guidelines
- Always build with ./scripts/build.sh
- When adding new crates, make sure they're the latest version possible
- We're running the project on the ESP32-S3 - a dual-core XTensa LX7 MCU