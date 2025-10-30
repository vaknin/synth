# PSRAM Enablement Plan

**Hardware:** ESP32-S3 N16R8 (16MB Flash + 8MB PSRAM)
**Status:** Not yet implemented (documented 2025-10-30)
**Priority:** Medium - Reduces stack pressure, enables future features

---

## Hardware Verification (from boot log)

```
ESP-ROM:esp32s3-20210327
rst:0x1 (POWERON),boot:0x8 (SPI_FAST_FLASH_BOOT)
mode:DIO, clock div:2
SPI Flash Size : 16MB
efuse block revision: v1.3
Boot SPI Speed : 40MHz
Partition Table:
 - factory: 0x00010000-0x00FB0000 (15.68MB available)
```

**Key findings:**
- **PSRAM:** 8MB external PSRAM available (N16R8 variant confirmed)
- **Flash:** 16MB SPI flash @ 40MHz DIO mode
- **Partition:** 15.68MB available for application (extremely generous)
- **Currently:** PSRAM is physically present but **not initialized or used**

---

## Current Memory Allocation Analysis

### 1. Display Framebuffer (32KB - Primary Target)

**Location:** `src/display/mod.rs:26-27`
```rust
const FRAMEBUFFER_SIZE: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 2) as usize;
static FRAMEBUFFER: StaticCell<[u8; FRAMEBUFFER_SIZE]> = StaticCell::new();
```

- **Size:** 128×128×2 bytes = 32,768 bytes (32KB)
- **Current allocation:** Static in `.bss` section (internal SRAM)
- **Initialization:** `FRAMEBUFFER.init([0u8; FRAMEBUFFER_SIZE])` in `DisplayDriver::new()`
- **Ownership:** Passed as `&'static mut [u8]` to `GraphicsModeAsync` (does not own buffer)
- **Impact on Core 1 stack:** Not directly on stack, but increases overall SRAM pressure

**Note:** Comments in `config.rs:18` incorrectly state "DisplayDriver initialization: 32KB framebuffer + ~2KB temporaries" as stack usage. The framebuffer is actually static (`.bss`), but the DisplayDriver struct itself (~2KB with cached_waveform) IS allocated on Core 1 stack during task init.

### 2. Waveform Buffers (2-8KB total)

**Primary buffer:** `src/message.rs:26-29`
```rust
pub struct WaveformData {
    pub samples: [i16; WAVEFORM_BUFFER_SIZE],  // 1024 samples × 2 bytes = 2KB
}
```

**Allocation locations:**
- **DisplayDriver.cached_waveform:** `src/display/mod.rs:35` - 2KB field in DisplayDriver struct
  - Lives on Core 1 stack during DisplayDriver initialization, then in task context
- **Engine waveform_capture.buffer:** Referenced in `src/engine.rs` - likely similar structure
- **Signal storage:** `WaveformSignal = Signal<ChannelMutex, WaveformData>` stores one 2KB copy internally

**Current total:** ~6KB across all waveform buffers (small, not primary target)

### 3. Core 1 Stack Configuration

**Current:** `src/config.rs:21`
```rust
pub const CORE_1_STACK_SIZE: usize = 98304; // 96KB
```

**Breakdown (from comments):**
- Embassy executor overhead: ~2-4KB
- pot_task ADC init: ~13KB (calibration data)
- display_task SPI DMA setup: ~2KB
- DisplayDriver struct (including 2KB cached_waveform): ~2KB
- XTensa call stack depth + alignment: ~5-10KB
- **Total estimated:** ~22-31KB actual usage, 96KB provides significant headroom

**Opportunity:** With better understanding of actual usage, could potentially reduce to 64KB.

### 4. No Heap Allocator Currently

- **Finding:** Zero uses of `esp_alloc`, `alloc` crate, or dynamic allocation
- **Current strategy:** Pure `no_std` with static (`StaticCell`) and stack allocations only
- **Implication:** Enabling PSRAM requires adding heap allocator infrastructure

---

## PSRAM Enablement Plan (Incremental)

### Phase 1: Enable PSRAM Infrastructure ⚙️

**Goal:** Set up PSRAM heap without moving any allocations yet (safe, reversible)

#### 1.1 Update `Cargo.toml`

**Add dependency:**
```toml
[dependencies]
esp-alloc = "0.6"  # PSRAM heap allocator
```

**Update esp-hal features:**
```toml
esp-hal = { version = "=1.0.0", features = [
  "esp32s3",
  "log-04",
  "unstable",
  "psram-8m",      # ← Enable 8MB PSRAM support
  "quad-psram",    # ← Quad SPI mode (faster than single)
] }
```

#### 1.2 Initialize PSRAM in `src/bin/main.rs`

**Add at top of file:**
```rust
extern crate alloc;  // Enable allocator support
use esp_alloc as _;  // Initialize global allocator
```

**In `main()` function, BEFORE spawning any tasks:**
```rust
#[esp_rtos::task(stack_size = config::CORE_0_STACK_SIZE_ESTIMATE)]
async fn main() {
    // Initialize PSRAM heap allocator (512KB initial size)
    // Note: 512KB is tiny fraction of 8MB available - can grow as needed
    esp_alloc::psram_allocator!(512 * 1024);

    log::info!("PSRAM heap initialized: 512KB");

    // ... rest of initialization
}
```

**Verification:**
- Build and flash
- Check boot logs for PSRAM initialization messages
- Should see no functional changes (heap exists but nothing uses it yet)

---

### Phase 2: Move Framebuffer to PSRAM 🎯

**Goal:** Allocate 32KB framebuffer in PSRAM heap instead of static SRAM

#### 2.1 Modify `src/display/mod.rs`

**Remove static allocation:**
```rust
// DELETE THIS:
// static FRAMEBUFFER: StaticCell<[u8; FRAMEBUFFER_SIZE]> = StaticCell::new();
```

**Update `DisplayDriver::new()` signature:**
```rust
use alloc::boxed::Box;

pub fn new(/* ... existing params ... */) -> Self {
    // Allocate framebuffer in PSRAM heap
    let framebuffer: &'static mut [u8; FRAMEBUFFER_SIZE] =
        Box::leak(Box::new([0u8; FRAMEBUFFER_SIZE]));

    // OLD CODE (remove):
    // let framebuffer = FRAMEBUFFER.init([0u8; FRAMEBUFFER_SIZE]);

    // ... rest of function unchanged (API stays identical)
}
```

**Alternative (explicit PSRAM allocator):**
If `Box::new()` doesn't default to PSRAM, use explicit allocator:
```rust
use alloc::alloc::{alloc_zeroed, Layout};

let layout = Layout::new::<[u8; FRAMEBUFFER_SIZE]>();
let ptr = unsafe { alloc_zeroed(layout) } as *mut [u8; FRAMEBUFFER_SIZE];
let framebuffer: &'static mut [u8; FRAMEBUFFER_SIZE] =
    unsafe { &mut *ptr };
```

**Benefits:**
- Frees 32KB in `.bss` section (reduces SRAM pressure)
- Identical API - callers unchanged
- PSRAM access slightly slower than SRAM, but framebuffer writes are async DMA (no impact)

---

### Phase 3: Optimize Core 1 Stack Size ⚡

**Goal:** Reduce stack allocation based on actual usage

#### 3.1 Update `src/config.rs`

**Before:**
```rust
pub const CORE_1_STACK_SIZE: usize = 98304; // 96KB
```

**After:**
```rust
/// Stack size for Core 1 (APP_CPU) - Embassy executor + ADC polling + display rendering.
/// Optimized to 64KB after verifying actual stack usage.
/// Stack usage breakdown:
/// - Embassy executor overhead (~2-4KB)
/// - pot_task: ADC init on Core 1 stack (~13KB calibration data)
/// - display_task SPI DMA setup + peripherals (~2KB)
/// - DisplayDriver struct (2KB cached_waveform + small overhead)
/// - XTensa call stack depth + alignment padding (~5-10KB)
/// Total estimated ~22-31KB, 64KB provides 2× safety margin.
/// Display framebuffer (32KB) now lives in PSRAM, not stack.
pub const CORE_1_STACK_SIZE: usize = 65536; // 64KB (reduced from 96KB)
```

**Verification strategy:**
1. First verify Phase 2 works with 96KB stack (no changes)
2. Then reduce to 64KB and test thoroughly
3. If any stack overflow issues, increase back to 80KB or 96KB

---

### Phase 4: Test & Verify ✅

**Build and test:**
```bash
./scripts/build.sh
# Monitor serial output for:
# - PSRAM initialization success
# - Display rendering works correctly
# - No crashes or memory errors
```

**Success criteria:**
- [ ] PSRAM heap initializes without errors
- [ ] Display task starts and renders correctly
- [ ] Waveform scrolling smooth (no visual artifacts)
- [ ] No stack overflow panics on Core 1
- [ ] Overall system stable for 5+ minutes

**Rollback plan if issues arise:**
1. Revert `config.rs` stack size to 96KB
2. Revert `src/display/mod.rs` to static framebuffer
3. Remove PSRAM features from `Cargo.toml`

---

## Future Optimizations (Post-PSRAM Enablement)

### Option A: Larger Waveform Buffer for Smoother Scrolling

**Current:** 1024 samples (~2KB)
**Proposed:** 4096-8192 samples (8-16KB)

**Changes required:**
```rust
// src/config.rs
pub const WAVEFORM_BUFFER_SIZE: usize = (DISPLAY_WIDTH as usize) * 64; // 8192 samples
```

**Allocation strategy:**
- Move WaveformData buffer to PSRAM (static or heap)
- Signal passes pointer/index instead of copying full buffer
- Display task reads directly from shared PSRAM buffer

**Benefits:**
- 4-8× more samples = smoother interpolation
- Reduces aliasing artifacts in waveform display
- Better handling of high-frequency content

**Complexity:**
- Requires synchronization strategy (currently copy-based Signal is dead simple)
- Minimal gain vs implementation effort - **recommend skipping unless display quality issues**

### Option B: Delay Effect Buffer in PSRAM

**Future feature:** Echo/delay effect requires large circular buffer
**Requirement:** ~44,100 samples/sec × 1 second delay = 88KB minimum (stereo i16)

**With PSRAM:**
- Allocate 256KB-1MB delay buffer in PSRAM
- Negligible impact (< 1/8th of 8MB available)
- Enables professional-quality delay effect

### Option C: Multiple Wavetable Banks

**Current:** Single 1024-sample wavetable in SRAM
**With PSRAM:** Store 32+ wavetables (32KB each) for morphing/switching

---

## Technical Notes & Gotchas

### PSRAM Performance Characteristics

- **Bandwidth:** ~40 MB/s (quad SPI @ 40MHz)
- **Latency:** ~200ns (vs ~10ns for internal SRAM)
- **Cache:** ESP32-S3 has PSRAM cache, mitigates latency for sequential access
- **DMA:** SPI DMA can read/write PSRAM directly (no CPU involvement)

**Implication for framebuffer:**
- Framebuffer writes are async DMA → PSRAM latency irrelevant
- Display flush time unchanged (DMA-limited, not memory-limited)

### Embassy Task Spawning (Critical Pattern Reminder)

From `CLAUDE.md`:
- **Keep task parameters small** during spawn (executor copies params to spawn stack)
- ✅ Pass raw peripheral handles (thin types: `ADC1`, `GPIO1`, ~8 bytes each)
- ✅ Initialize heavy peripherals INSIDE tasks (on task's own stack)
- ❌ DON'T pass initialized peripherals with calibration data (e.g., `Adc<ADC1>` ~4-8KB)

**Applies to PSRAM:**
- Framebuffer allocation happens inside `DisplayDriver::new()` (called within display task)
- No spawn stack impact - PSRAM heap allocation is runtime, inside task context
- Pattern remains safe ✅

### Stack Size Reduction Caution

**Conservative approach:**
1. Don't reduce stack below 64KB (2× safety margin over 31KB estimate)
2. Test extensively before going smaller
3. If unsure, keep 96KB (SRAM is abundant on ESP32-S3)

**XTensa call stack considerations:**
- XTensa can have deep call stacks (nested async calls)
- Embassy executor adds overhead
- Better to waste 32KB SRAM than risk mysterious stack overflow

---

## Decision Matrix

| Use Case | Enable PSRAM? | Move Framebuffer? | Reduce Stack? |
|----------|---------------|-------------------|---------------|
| **Current project (display + audio)** | ✅ Yes | ✅ Yes | ⚠️ Maybe (64KB conservative) |
| **Future delay effect** | ✅ Required | ✅ Yes | ✅ Yes (need SRAM for audio) |
| **Future wavetable morphing** | ✅ Recommended | ✅ Yes | ✅ Yes |
| **Minimal project (no future features)** | ❌ Skip | ❌ Skip | ❌ Keep 96KB |

**Recommendation for this project:**
Implement Phases 1-2 (PSRAM + framebuffer move) now. Defer Phase 3 (stack reduction) until delay effect implementation when SRAM pressure increases.

---

## Appendix: Research Data

### File Locations (for implementation)

| Component | File Path | Line Numbers |
|-----------|-----------|--------------|
| Framebuffer static declaration | `src/display/mod.rs` | 26-27 |
| Framebuffer initialization | `src/display/mod.rs` | 57 |
| DisplayDriver struct definition | `src/display/mod.rs` | 32-38 |
| Core 1 stack size constant | `src/config.rs` | 21 |
| WaveformData struct | `src/message.rs` | 26-29 |
| WaveformSignal type alias | `src/lib.rs` | 36 |

### Memory Layout (Pre-PSRAM)

```
ESP32-S3 Memory Map (320KB internal SRAM):
┌─────────────────────────────────┐
│ Core 0 Stack (~16KB runtime)    │ Audio rendering (esp-rtos managed)
├─────────────────────────────────┤
│ Core 1 Stack (96KB allocated)   │ Embassy executor + display task
│  ├─ Embassy executor ~4KB       │
│  ├─ ADC calibration ~13KB       │
│  ├─ DisplayDriver struct ~2KB   │
│  └─ Call stack + margin ~77KB   │
├─────────────────────────────────┤
│ .bss segment (static data)      │
│  ├─ FRAMEBUFFER 32KB ◄──────────┼─── TARGET: Move to PSRAM
│  └─ Other statics ~10KB         │
├─────────────────────────────────┤
│ .data segment (initialized)     │ ~20KB
├─────────────────────────────────┤
│ Heap (if enabled)               │ (Currently unused - no allocator)
└─────────────────────────────────┘

External PSRAM (8MB):
┌─────────────────────────────────┐
│ UNUSED - NOT INITIALIZED        │ 8,388,608 bytes available!
└─────────────────────────────────┘
```

### Memory Layout (Post-PSRAM)

```
ESP32-S3 Memory Map (320KB internal SRAM):
┌─────────────────────────────────┐
│ Core 0 Stack (~16KB runtime)    │ Audio rendering
├─────────────────────────────────┤
│ Core 1 Stack (64KB allocated)   │ ◄─── Reduced by 32KB
│  ├─ Embassy executor ~4KB       │
│  ├─ ADC calibration ~13KB       │
│  ├─ DisplayDriver struct ~2KB   │
│  └─ Call stack + margin ~45KB   │
├─────────────────────────────────┤
│ .bss segment (static data)      │
│  └─ Statics ~10KB               │ ◄─── 32KB freed!
├─────────────────────────────────┤
│ .data segment                   │ ~20KB
├─────────────────────────────────┤
│ (SRAM freed for future use)     │ ~32KB+ available
└─────────────────────────────────┘

External PSRAM (8MB):
┌─────────────────────────────────┐
│ Heap allocator (512KB region)   │
│  ├─ FRAMEBUFFER 32KB ◄──────────┼─── Moved from SRAM
│  └─ Available ~480KB            │
├─────────────────────────────────┤
│ Unused PSRAM (~7.5MB)           │ For future delay/wavetables
└─────────────────────────────────┘
```

**Net result:**
- ~64KB SRAM freed (32KB stack + 32KB .bss)
- 32KB PSRAM used (trivial - only 0.4% of 8MB)
- Abundant headroom for future features

---

**Status:** Documentation complete, ready for implementation when needed.
