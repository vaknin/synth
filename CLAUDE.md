# Documentation

**ESP32-S3 HAL References:**
- Local: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/esp-hal-1.0.0-rc.0/`

**Architecture Docs:**
- **`docs/mvp/ARCHITECTURE.md`**: HOW we build (processing rates, optimizations, patterns)
- **`docs/mvp/ROADMAP.md`**: WHAT we're building next (generative synthesis phases 4A→4E)

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
- Crates: Use latest versions
- Code style: Elegant, efficient, public fields over getters
- Collaboration: Write code incrementally (function/struct-length), not full files - user wants ownership
- Questions: Number them, attach recommendations (based on audio/hardware best practices)
- Comments: Only when strictly needed, concise. Never include specific numeric values in comments if those values come from config.rs - they may change and create stale documentation
- Where possible and necessary, run python code to verify the algorithm/calculation is correct
