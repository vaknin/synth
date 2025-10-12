//! Turing Machine melody generator using probabilistic bit mutations.
//!
//! A shift register (LFSR-style) with controlled mutation for evolving melodies.
//! Output is bipolar [-1.0, 1.0] for scale quantization.

use crate::config::{
    TURING_LOOP_LENGTH_DEFAULT, TURING_LOOP_LENGTH_MAX, TURING_LOOP_LENGTH_MIN,
    TURING_MUTATION_DEFAULT, TURING_MUTATION_MAX,
};
use crate::noise::WhiteNoise;

/// Turing Machine melody generator.
///
/// 32-bit shift register with probabilistic mutations.
/// Outputs normalized value for scale quantization.
pub struct TuringMachine {
    /// 32-bit shift register state (used for filter modulation)
    pub register: u32,
    /// Active loop length in bits (2-32)
    pub loop_length: usize,
    /// Mutation probability per step (0.0-0.5)
    pub mutation_rate: f32,
    /// RNG for mutation decisions
    noise: WhiteNoise,
}

impl TuringMachine {
    /// Create new Turing Machine with random initial state.
    pub fn new() -> Self {
        let mut noise = WhiteNoise::new();

        // Generate random initial register state (full 32 bits)
        let register = (noise.next_f32() * u32::MAX as f32) as u32;

        Self {
            register,
            loop_length: TURING_LOOP_LENGTH_DEFAULT,
            mutation_rate: TURING_MUTATION_DEFAULT,
            noise,
        }
    }

    /// Advance shift register and return normalized output.
    ///
    /// Call this once per clock tick (buffer rate ~43 Hz).
    ///
    /// # Returns
    /// Normalized register value in range [-1.0, 1.0] for quantizer input
    #[inline]
    pub fn tick(&mut self) -> f32 {
        // Extract rightmost bit (will wrap to left)
        let mut bit = self.register & 1;

        // Mutate with probability mutation_rate
        if self.noise.next_f32() < self.mutation_rate {
            bit ^= 1;  // Flip the bit
        }

        // Shift register right by 1
        self.register >>= 1;

        // Insert (possibly mutated) bit at leftmost position of loop
        self.register |= bit << (self.loop_length - 1);

        // Mask to keep only loop_length bits active
        let mask = (1 << self.loop_length) - 1;
        self.register &= mask;

        // Normalize to [-1.0, 1.0]
        let max_value = mask as f32;
        (self.register as f32 / max_value) * 2.0 - 1.0
    }

    /// Set loop length in bits (2-32) and re-mask register.
    pub fn set_loop_length(&mut self, length: usize) {
        let new_length = length.clamp(TURING_LOOP_LENGTH_MIN, TURING_LOOP_LENGTH_MAX);
        if new_length != self.loop_length {
            self.loop_length = new_length;
            // Re-mask register to new length
            let mask = (1 << self.loop_length) - 1;
            self.register &= mask;
        }
    }

    /// Set mutation rate (0.0-0.5 for 0-50% probability).
    pub fn set_mutation_rate(&mut self, rate: f32) {
        self.mutation_rate = rate.clamp(0.0, TURING_MUTATION_MAX);
    }
}

impl Default for TuringMachine {
    fn default() -> Self {
        Self::new()
    }
}
