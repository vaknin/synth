# DMA Circular Buffer Alignment Issue with I2S

## Problem

When using `dma_circular_buffers!()` with I2S stereo audio, certain buffer sizes cause misaligned descriptor boundaries, resulting in callbacks with partial stereo frames that cannot be processed.

### Observed Behavior

With a buffer size of **4096 bytes** and `DataFormat::Data16Channel16`:

```
INFO - Buffer: 1366 bytes   ← OK
INFO - Buffer: 2 bytes       ← Too small for stereo frame (need 4 bytes)!
INFO - Buffer: 2 bytes       ← Infinite loop
```

The 2-byte buffers cannot hold a complete stereo frame (4 bytes = left channel 2 bytes + right channel 2 bytes), causing:
- **Returning 0**: DMA deadlock (buffer pointer doesn't advance)
- **Filling with silence**: Breaks I2S WS synchronization

## Root Cause

For circular buffers **≤ 8184 bytes**, esp-hal splits the buffer into 3 descriptors using:

```rust
// From esp-hal/src/dma/mod.rs:1036-1043
let max_chunk_size = if circular && len <= self.chunk_size * 2 {
    len / 3 + len % 3  // ← Does not guarantee frame alignment
} else {
    self.chunk_size    // = 4092
};
```

### Example: 4096 bytes with 16-bit stereo

```
max_chunk_size = 4096 / 3 + 4096 % 3 = 1365 + 1 = 1366
Descriptors: [1366, 1366, 1364] bytes
Problem: 1366 % 4 = 2 ❌ (not aligned to 4-byte stereo frames)
```

### Frame Sizes by Data Format

| DataFormat | Bytes per Frame | Required Buffer Divisibility |
|------------|-----------------|------------------------------|
| Data32Channel32 | 8 | **buffer_size % 24 = 0** |
| Data32Channel24 | 8 | **buffer_size % 24 = 0** |
| Data32Channel16 | 8 | **buffer_size % 24 = 0** |
| Data32Channel8  | 8 | **buffer_size % 24 = 0** |
| Data16Channel16 | 4 | **buffer_size % 12 = 0** |
| Data16Channel8  | 4 | **buffer_size % 12 = 0** |
| Data8Channel8   | 2 | **buffer_size % 6 = 0** |

**Formula**: `buffer_size % LCM(3, frame_size) = 0`

Where LCM is the Least Common Multiple of 3 (descriptors) and the frame size in bytes.

## Solution

### Buffer Size Requirements

Choose buffer sizes based on your data format:

#### 16-bit Stereo (Data16Channel16) - Divisible by 12

```rust
// ✓ Correct
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 2052);
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 4092);

// ❌ Wrong
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 1024);  // 1024 % 12 != 0
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 4096);  // 4096 % 12 != 0
```

#### 32-bit Stereo (Data32Channel32) - Divisible by 24

```rust
// ✓ Correct
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 2040);
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 4080);
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 8184);  // Maximum

// ❌ Wrong
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 4092);  // 4092 % 24 != 0
```

**Important**: For 32-bit formats, buffers **must be ≤ 8184 bytes**. Larger buffers use 4092-byte chunks which are not divisible by 8, breaking frame alignment.

#### 8-bit Stereo (Data8Channel8) - Divisible by 6

```rust
// ✓ Correct
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 2040);
let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 4092);
```

### Verification

Add debug logging to detect misalignment:

```rust
transaction.push_with(|buffer| {
    let frame_size = 4; // 4 bytes for Data16Channel16

    if buffer.len() < frame_size {
        // Should NEVER happen with correct buffer size
        panic!("Buffer too small: {} bytes", buffer.len());
    }

    let remainder = buffer.len() % frame_size;
    // Process complete frames...
    buffer.len() - remainder
}).await;
```

Expected output with correctly sized buffers:

```
INFO - Buffer: 684 bytes    ← 684 % 4 = 0 ✓
INFO - Buffer: 1368 bytes   ← Two adjacent descriptors available
INFO - Buffer: 684 bytes
```

## Proposed Fixes for esp-hal

### 1. Documentation (Minimal Change)

Add to `dma_circular_buffers!` macro documentation:

```rust
/// ⚠️ **For I2S stereo audio**: Buffer size must be divisible by `LCM(3, frame_size)`
/// to ensure descriptor boundaries align with frame boundaries:
/// - 16-bit stereo: divisible by 12
/// - 32-bit stereo: divisible by 24 (and ≤ 8184 bytes)
/// - 8-bit stereo: divisible by 6
///
/// Non-aligned sizes cause partial-frame callbacks that cannot be processed.
```

### 2. Runtime Validation (Recommended)

Add validation in I2S driver:

```rust
pub fn write_dma_circular_async<TXBUF: ReadBuffer>(
    mut self,
    words: TXBUF,
) -> Result<I2sWriteDmaTransferAsync<'d, TXBUF>, Error> {
    let (ptr, len) = unsafe { words.read_buffer() };

    let frame_size = match self.data_format {
        DataFormat::Data32Channel32 | DataFormat::Data32Channel24 |
        DataFormat::Data32Channel16 | DataFormat::Data32Channel8 => 8,
        DataFormat::Data16Channel16 | DataFormat::Data16Channel8 => 4,
        DataFormat::Data8Channel8 => 2,
    };

    let required_divisibility = frame_size * 3 / gcd(3, frame_size);

    if len % required_divisibility != 0 {
        return Err(Error::InvalidArgument);
    }

    // ... rest of function
}
```

### 3. Compile-Time Helper (Ideal)

```rust
/// Create I2S-safe circular DMA buffers with compile-time alignment checks
macro_rules! dma_circular_buffers_i2s {
    ($rx_size:expr, $tx_size:expr, frame_size = $frame_size:expr) => {{
        const REQUIRED_DIVISIBILITY: usize = {
            // LCM(3, frame_size)
            let gcd = const_gcd(3, $frame_size);
            (3 * $frame_size) / gcd
        };

        const {
            ::core::assert!(
                $tx_size % REQUIRED_DIVISIBILITY == 0,
                "I2S buffer size must be divisible by LCM(3, frame_size) for proper alignment"
            );
        }

        $crate::dma_circular_buffers!($rx_size, $tx_size)
    }};
}
```

## References

- **Issue**: Buffer sizes not divisible by LCM(3, frame_size) cause partial-frame callbacks
- **Affected**: esp-hal v1.0.0-rc.0 (likely all versions)
- **Code**: `esp-hal/src/dma/mod.rs:1036-1043` (buffer splitting), `esp-hal/src/dma/mod.rs:1262` (chunks)
- **Hardware**: ESP32 DMA CHUNK_SIZE = 4092 bytes
- **Discovered**: 2025-10, during ESP32 synthesizer development
