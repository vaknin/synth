//! Low-frequency oscillator for modulation.
//!
//! Simple sine LFO for parameter modulation (filter cutoff, pitch, etc).
//! Runs at buffer rate - parameters updated directly without Slew.

use crate::config::{BUFFER_RATE, WAVETABLE_MASK, WAVETABLE_SIZE_F32};
use crate::oscillator::SINE;
use micromath::F32Ext;

/// Reciprocal of buffer rate for fast phase increment calculation.
/// Eliminates division in hot path (multiply is faster than divide).
const BUFFER_RATE_RECIPROCAL: f32 = 1.0 / BUFFER_RATE;

/// Low-frequency oscillator for parameter modulation.
///
/// Generates smooth sine wave modulation at sub-audio rates.
/// Ticked at buffer rate, not audio rate.
/// No Slew smoothing needed - LFO output is inherently smooth, and modulation
/// targets (filter coefficients, pitch) have their own smoothing.
pub struct LFO {
    /// Current phase [0.0, 1.0)
    phase: f32,
    /// Phase increment per tick (frequency / BUFFER_RATE)
    phase_increment: f32,
    /// LFO frequency in Hz
    frequency: f32,
    /// LFO depth 0.0-1.0 (modulation amount)
    pub depth: f32,
}

impl LFO {
    /// Create new LFO with specified frequency and depth.
    ///
    /// # Arguments
    /// * `frequency` - LFO rate in Hz
    /// * `depth` - Modulation depth 0.0-1.0
    pub fn new(frequency: f32, depth: f32) -> Self {
        let phase_increment = frequency * BUFFER_RATE_RECIPROCAL;

        Self {
            phase: 0.0,
            phase_increment,
            frequency,
            depth,
        }
    }

    /// Set LFO frequency in Hz.
    ///
    /// Updates phase_increment to maintain correct rate.
    ///
    /// # Arguments
    /// * `frequency` - LFO rate in Hz
    #[inline]
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.phase_increment = frequency * BUFFER_RATE_RECIPROCAL;
    }

    /// Generate next LFO sample (BUFFER RATE).
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

        // Advance phase with modulo wrapping
        self.phase += self.phase_increment;
        self.phase = self.phase.fract();

        // Return modulation value scaled by depth
        sine * self.depth
    }
}
