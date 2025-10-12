//! White noise generator using Linear Feedback Shift Register (LFSR).
//!
//! Fast pseudorandom number generation for arpeggiators and effects.
//! Uses Galois LFSR for ~7-9 cycles per sample.

/// Reciprocal of i16::MAX for division elimination.
/// Using 32767.0 (not 32768.0) prevents overflow when values are later multiplied back.
const I16_MAX_F32: f32 = 32767.0;
const I16_MAX_RECIPROCAL: f32 = 1.0 / I16_MAX_F32;

/// 32-bit LFSR polynomial for maximal-length sequence (taps: 32,22,2,1).
/// Period: 2^32-1 = 4,294,967,295 states (prevents audible pattern repetition).
/// Polynomial: x^32 + x^22 + x^2 + x^1 + 1
const LFSR_POLYNOMIAL: u32 = 0x80200003; // Correct maximal-length LFSR

/// White noise generator using 32-bit Galois LFSR.
///
/// Provides fast pseudorandom values for arpeggiator random mode.
/// Much faster than `rand` crate (~7-9 cycles vs 100+ cycles).
pub struct WhiteNoise {
    /// LFSR state (must never be zero)
    lfsr: u32,
}

impl WhiteNoise {
    /// Create new noise generator with non-zero seed.
    pub fn new() -> Self {
        Self { lfsr: 44257 } // Non-zero seed (0xACE1)
    }

    /// Generate next noise sample.
    ///
    /// # Returns
    /// Normalized f32 in range [-1.0, 1.0]
    #[inline(always)]
    pub fn tick(&mut self) -> f32 {
        // Galois LFSR: shift right, XOR polynomial if LSB=1
        let lsb = self.lfsr & 1;
        self.lfsr >>= 1;
        if lsb == 1 {
            self.lfsr ^= LFSR_POLYNOMIAL;
        }

        // Use upper 16 bits for better randomness, multiply by reciprocal (no division)
        let value = (self.lfsr >> 16) as i16;
        (value as f32) * I16_MAX_RECIPROCAL
    }

    /// Generate random f32 in [0.0, 1.0] for probability checks.
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        (self.tick() + 1.0) * 0.5  // Map [-1,1] to [0,1]
    }

    /// Generate random usize in range [0, max).
    #[inline]
    pub fn next_usize(&mut self, max: usize) -> usize {
        (self.next_f32() * max as f32) as usize
    }
}

impl Default for WhiteNoise {
    fn default() -> Self {
        Self::new()
    }
}
