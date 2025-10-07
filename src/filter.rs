//! State Variable Filter (SVF) - 2-pole resonant filter.
//!
//! Provides simultaneous lowpass, highpass, and bandpass outputs.
//! Uses Chamberlin topology for numerical stability and efficiency.

use crate::config::{
    FILTER_CUTOFF_MAX, FILTER_INTEGRATOR_LIMIT, SAMPLE_RATE,
};
use crate::slew::Slew;

/// State Variable Filter with simultaneous LP/HP/BP outputs.
///
/// Digital implementation of analog SVF topology (Chamberlin).
/// Self-oscillates at high resonance (~1.0+), stable up to Nyquist.
pub struct StateVariableFilter {
    /// Lowpass integrator state
    lp: f32,
    /// Bandpass integrator state
    bp: f32,
    /// Cutoff frequency in Hz (smoothed)
    cutoff: Slew,
    /// Resonance amount (0.0 = no resonance, 1.0+ = self-oscillation)
    /// Stored as smoothed Slew for parameter automation
    resonance: Slew,
    /// Sample rate in Hz
    sample_rate: f32,
    /// Cached frequency coefficient (recalculated only when cutoff changes)
    cached_f: f32,
    /// Cached damping coefficient (recalculated only when resonance changes)
    cached_q: f32,
    /// Last cutoff value (for change detection)
    last_cutoff: f32,
    /// Last resonance value (for change detection)
    last_resonance: f32,
}

impl StateVariableFilter {
    /// Create new filter with initial cutoff frequency and resonance.
    ///
    /// # Arguments
    /// * `cutoff_hz` - Initial cutoff frequency in Hz (20-20000)
    /// * `resonance` - Resonance amount (0.0-1.2, typical 0.7 for gentle resonance)
    pub fn new(cutoff_hz: f32, resonance: f32) -> Self {
        let sample_rate = SAMPLE_RATE as f32;
        let clamped_resonance = resonance.clamp(0.1, 10.0);

        // Pre-calculate initial coefficients
        let f = (2.0 * core::f32::consts::PI * cutoff_hz / sample_rate).min(1.0);
        let q = 1.0 / clamped_resonance;

        Self {
            lp: 0.0,
            bp: 0.0,
            cutoff: Slew::new(cutoff_hz),
            resonance: Slew::new(clamped_resonance),
            sample_rate,
            cached_f: f,
            cached_q: q,
            last_cutoff: cutoff_hz,
            last_resonance: clamped_resonance,
        }
    }

    /// Set cutoff frequency in Hz.
    ///
    /// # Arguments
    /// * `freq` - Cutoff frequency in Hz (FILTER_CUTOFF_MIN to FILTER_CUTOFF_MAX)
    #[inline]
    pub fn set_cutoff(&mut self, freq: f32) {
        // Clamp to safe range using config constants
        let safe_freq = freq.clamp(crate::config::FILTER_CUTOFF_MIN, FILTER_CUTOFF_MAX);
        self.cutoff.set_target(safe_freq);
    }

    /// Set resonance amount.
    ///
    /// # Arguments
    /// * `resonance` - Resonance (0.1-10.0, 1.0+ enables self-oscillation)
    #[inline]
    pub fn set_resonance(&mut self, resonance: f32) {
        let safe_resonance = resonance.clamp(0.1, 10.0);
        self.resonance.set_target(safe_resonance);
    }

    /// Process one sample through the filter (all three outputs).
    ///
    /// # Arguments
    /// * `input` - Input sample
    ///
    /// # Returns
    /// Tuple of (lowpass, highpass, bandpass) outputs
    ///
    /// # References
    /// Hal Chamberlin, "Musical Applications of Microprocessors" (1985)
    #[inline]
    pub fn process(&mut self, input: f32) -> (f32, f32, f32) {
        // Get smoothed parameters (cheap when settled!)
        let cutoff_hz = self.cutoff.value();
        let res = self.resonance.value();

        // Only recalculate coefficients if parameters actually changed
        // Optimization: During steady-state (90%+ of the time), this skips expensive calculations
        if cutoff_hz != self.last_cutoff {
            self.last_cutoff = cutoff_hz;
            // Calculate frequency coefficient: f = 2π × fc / fs (approximation)
            // Accurate for fc < fs/6, which covers our range (FILTER_CUTOFF_MAX ≈ 7kHz)
            let f = 2.0 * core::f32::consts::PI * cutoff_hz / self.sample_rate;
            // Chamberlin SVF stability constraint: f ≤ 1.0
            self.cached_f = f.min(1.0);
        }

        if res != self.last_resonance {
            self.last_resonance = res;
            // Damping coefficient (inverse resonance)
            self.cached_q = 1.0 / res;
        }

        // Use cached coefficients
        let f = self.cached_f;
        let q = self.cached_q;

        // SVF equations (Chamberlin topology):
        // hp = input - lp - q * bp
        let hp = input - self.lp - q * self.bp;

        // bp = bp + f * hp  (bandpass integrator)
        self.bp += f * hp;

        // lp = lp + f * bp  (lowpass integrator)
        self.lp += f * self.bp;

        // Integrator state limiting to prevent unbounded growth/distortion
        // Uses config constant for headroom (allows resonance peaks while preventing runaway)
        self.bp = self.bp.clamp(-FILTER_INTEGRATOR_LIMIT, FILTER_INTEGRATOR_LIMIT);
        self.lp = self.lp.clamp(-FILTER_INTEGRATOR_LIMIT, FILTER_INTEGRATOR_LIMIT);

        // Return all three outputs
        (self.lp, hp, self.bp)
    }

    /// Reset filter state (clears integrators).
    ///
    /// Useful when changing patches or stopping audio to prevent ringing.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.lp = 0.0;
        self.bp = 0.0;
    }
}
