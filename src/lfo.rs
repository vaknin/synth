//! Low-frequency oscillator for modulation.
//!
//! Simple sine LFO for parameter modulation (filter cutoff, pitch, etc).
//! Runs at buffer rate (~172 Hz) - parameters updated directly without Slew.

use crate::config::{SAMPLE_RATE, WAVETABLE_MASK, WAVETABLE_SIZE_F32};
use crate::oscillator::SINE;

/// Low-frequency oscillator for parameter modulation.
///
/// Generates smooth sine wave modulation at sub-audio rates (0.01-20 Hz).
/// No Slew smoothing needed - LFO output is inherently smooth, and modulation
/// targets (filter coefficients, pitch) have their own smoothing.
pub struct LFO {
    /// Current phase [0.0, 1.0)
    phase: f32,
    /// Phase increment per tick (frequency / sample_rate)
    phase_increment: f32,
    /// LFO frequency in Hz (0.01-20 Hz range)
    frequency: f32,
    /// LFO depth 0.0-1.0 (modulation amount)
    pub depth: f32,
    /// Sample rate (constant)
    sample_rate: f32,
}

impl LFO {
    /// Create new LFO with specified frequency and depth.
    ///
    /// # Arguments
    /// * `frequency` - LFO rate in Hz (0.01-20 Hz typical)
    /// * `depth` - Modulation depth 0.0-1.0
    pub fn new(frequency: f32, depth: f32) -> Self {
        let sample_rate = SAMPLE_RATE as f32;
        let phase_increment = frequency / sample_rate;

        Self {
            phase: 0.0,
            phase_increment,
            frequency,
            depth,
            sample_rate,
        }
    }

    /// Set LFO frequency in Hz.
    ///
    /// Updates phase_increment to maintain correct rate.
    ///
    /// # Arguments
    /// * `frequency` - LFO rate in Hz (0.01-20 Hz typical)
    #[inline]
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.phase_increment = frequency / self.sample_rate;
    }

    /// Generate next LFO sample (BUFFER RATE ~172 Hz).
    ///
    /// Returns bipolar modulation value [-depth, +depth].
    /// Apply this to modulation targets in update_buffer_parameters().
    ///
    /// # Returns
    /// Sine wave scaled by depth: range [-depth, +depth]
    #[inline(always)]
    pub fn tick(&mut self) -> f32 {
        // Sine wavetable lookup with linear interpolation (shares SINE from oscillator)
        let table_pos = self.phase * WAVETABLE_SIZE_F32;
        let index = table_pos as usize;
        let frac = table_pos - index as f32;

        let s1 = SINE[index];
        let s2 = SINE[(index + 1) & WAVETABLE_MASK];
        let sine = s1 + (s2 - s1) * frac;

        // Advance phase
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Return modulation value scaled by depth
        sine * self.depth
    }
}
