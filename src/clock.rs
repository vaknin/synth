//! Master tempo clock for rhythm and melody generation.
//!
//! Generates step ticks at configured subdivision (16th notes default).
//! BPM changes are smoothed via slew limiter for musical tempo transitions.

use crate::config::{
    BUFFER_RATE, CLOCK_BPM_DEFAULT, CLOCK_BPM_MAX, CLOCK_BPM_MIN,
    CLOCK_STEPS_PER_BEAT, CLOCK_TEMPO_SLEW_MS,
};
use crate::slew::Slew;
use micromath::F32Ext;

/// Master tempo clock for rhythm and melody generation.
///
/// Generates step ticks at configured subdivision (16th notes default).
/// BPM changes are smoothed via slew limiter for musical tempo transitions.
pub struct Clock {
    /// Current BPM with smooth transitions
    pub bpm: Slew,
    /// Phase accumulator in seconds (wraps at step interval)
    phase: f32,
}

impl Clock {
    /// Create new clock with default tempo.
    pub fn new() -> Self {
        // Calculate slew coefficient from CLOCK_TEMPO_SLEW_MS
        // For EMA: α = exp(-Δt / τ) where τ = slew time, Δt = buffer period
        let tau = CLOCK_TEMPO_SLEW_MS / 1000.0; // Convert ms to seconds
        let delta_t = 1.0 / BUFFER_RATE;
        let slew_coeff = (-delta_t / tau).exp();

        Self {
            bpm: Slew::with_coeff(CLOCK_BPM_DEFAULT, slew_coeff),
            phase: 0.0,
        }
    }

    /// Advance clock and check if step occurred.
    ///
    /// Call once per buffer callback (~43 Hz).
    ///
    /// # Returns
    /// true if a step tick occurred this buffer, false otherwise
    #[inline]
    pub fn tick(&mut self) -> bool {
        // Advance BPM slew limiter
        let current_bpm = self.bpm.value();

        // Calculate step interval in seconds
        // steps_per_second = (BPM / 60) * CLOCK_STEPS_PER_BEAT
        let steps_per_second = (current_bpm / 60.0) * CLOCK_STEPS_PER_BEAT as f32;
        let step_interval = 1.0 / steps_per_second;

        // Advance phase by buffer period
        let delta_t = 1.0 / BUFFER_RATE;
        self.phase += delta_t;

        // Check if step occurred
        if self.phase >= step_interval {
            self.phase -= step_interval; // Wrap phase (keep remainder for accuracy)
            true
        } else {
            false
        }
    }

    /// Set target BPM (smoothly transitions over CLOCK_TEMPO_SLEW_MS).
    pub fn set_bpm(&mut self, bpm: f32) {
        let clamped = bpm.clamp(CLOCK_BPM_MIN, CLOCK_BPM_MAX);
        self.bpm.set_target(clamped);
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
