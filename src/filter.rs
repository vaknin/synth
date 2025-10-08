//! State Variable Filter (SVF) - 2-pole resonant filter.
//!
//! Provides simultaneous lowpass, highpass, and bandpass outputs.
//! Uses Chamberlin topology for numerical stability and efficiency.
//!
//! Optimized: Coefficients cached at buffer rate, control smoothing via ADC multisampling.

use crate::config::{
    FILTER_CUTOFF_MAX, FILTER_INTEGRATOR_LIMIT, FILTER_RESONANCE_MAX, FILTER_RESONANCE_MIN,
    SAMPLE_RATE,
};

/// State Variable Filter with simultaneous LP/HP/BP outputs.
///
/// Digital implementation of analog SVF topology (Chamberlin).
/// Self-oscillates at high resonance (~1.0+), stable up to Nyquist.
///
/// Call update_coefficients() once per buffer, then process() at audio rate.
pub struct StateVariableFilter {
    /// Lowpass integrator state
    lp: f32,
    /// Bandpass integrator state
    bp: f32,
    /// Cutoff frequency in Hz (updated at control rate from pots)
    pub cutoff: f32,
    /// Resonance amount (0.0 = no resonance, 1.0+ = self-oscillation)
    pub resonance: f32,
    /// Sample rate in Hz
    sample_rate: f32,
    /// Cached frequency coefficient: f = 2π × fc/fs (updated per buffer)
    cached_f: f32,
    /// Cached damping coefficient: q = 1/resonance (updated per buffer)
    cached_q: f32,
    /// Whether to clamp integrators (true when resonance > 2.0)
    needs_clamping: bool,
    /// Whether coefficients need recalculation (dirty flag optimization)
    dirty: bool,
}

impl StateVariableFilter {
    /// Create new filter with initial cutoff frequency and resonance.
    ///
    /// # Arguments
    /// * `cutoff_hz` - Initial cutoff frequency in Hz (20-20000)
    /// * `resonance` - Resonance amount (0.0-1.2, typical 0.7 for gentle resonance)
    pub fn new(cutoff_hz: f32, resonance: f32) -> Self {
        let sample_rate = SAMPLE_RATE as f32;
        let clamped_cutoff = cutoff_hz.clamp(crate::config::FILTER_CUTOFF_MIN, FILTER_CUTOFF_MAX);
        let clamped_resonance = resonance.clamp(FILTER_RESONANCE_MIN, FILTER_RESONANCE_MAX);

        // Pre-calculate initial coefficients
        let f = (2.0 * core::f32::consts::PI * clamped_cutoff / sample_rate).min(1.0);
        let q = 1.0 / clamped_resonance;

        Self {
            lp: 0.0,
            bp: 0.0,
            cutoff: clamped_cutoff,
            resonance: clamped_resonance,
            sample_rate,
            cached_f: f,
            cached_q: q,
            needs_clamping: clamped_resonance > 2.0,
            dirty: false, // Coefficients already calculated
        }
    }

    /// Set cutoff frequency in Hz.
    ///
    /// # Arguments
    /// * `freq` - Cutoff frequency in Hz (FILTER_CUTOFF_MIN to FILTER_CUTOFF_MAX)
    #[inline]
    pub fn set_cutoff(&mut self, freq: f32) {
        self.cutoff = freq.clamp(crate::config::FILTER_CUTOFF_MIN, FILTER_CUTOFF_MAX);
        self.dirty = true; // Mark for coefficient recalculation (pot deadband handles tiny changes)
    }

    /// Set resonance amount.
    ///
    /// # Arguments
    /// * `resonance` - Resonance (FILTER_RESONANCE_MIN to FILTER_RESONANCE_MAX)
    #[inline]
    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(FILTER_RESONANCE_MIN, FILTER_RESONANCE_MAX);
        self.dirty = true; // Mark for coefficient recalculation (pot deadband handles tiny changes)
    }

    /// Update filter coefficients from direct parameters.
    ///
    /// Call once per buffer before processing samples.
    /// Recalculates f and q coefficients from current cutoff/resonance values.
    /// Only recalculates when parameters changed (dirty flag optimization).
    #[inline]
    pub fn update_coefficients(&mut self) {
        // Early return if no parameters changed (most of the time)
        if !self.dirty {
            return;
        }

        // Use direct values (updated at control rate from pots)
        let cutoff = self.cutoff;
        let resonance = self.resonance;

        // Recalculate frequency coefficient: f = 2π × fc/fs
        self.cached_f = (2.0 * core::f32::consts::PI * cutoff / self.sample_rate).min(1.0);

        // Recalculate damping coefficient: q = 1/resonance
        self.cached_q = 1.0 / resonance;

        // Conditional clamping: only needed at high resonance
        self.needs_clamping = resonance > 2.0;

        // Clear dirty flag (coefficients now up to date)
        self.dirty = false;
    }

    /// Process one sample through the filter (all three outputs).
    ///
    /// Uses cached coefficients - call update_coefficients() once per buffer first.
    ///
    /// # Arguments
    /// * `input` - Input sample (normalized ±1.0)
    ///
    /// # Returns
    /// Tuple of (lowpass, highpass, bandpass) outputs (nominally ±1.0, but can exceed with high resonance)
    ///
    /// # References
    /// Hal Chamberlin, "Musical Applications of Microprocessors" (1985)
    #[inline]
    pub fn process(&mut self, input: f32) -> (f32, f32, f32) {
        // Use cached coefficients (no smoothing, no branches!)
        let f = self.cached_f;
        let q = self.cached_q;

        // SVF equations (Chamberlin topology):
        // hp = input - lp - q * bp
        let hp = input - self.lp - q * self.bp;

        // bp = bp + f * hp  (bandpass integrator)
        self.bp += f * hp;

        // lp = lp + f * bp  (lowpass integrator)
        self.lp += f * self.bp;

        // Conditional integrator clamping (only at high resonance > 2.0)
        if self.needs_clamping {
            self.bp = self
                .bp
                .clamp(-FILTER_INTEGRATOR_LIMIT, FILTER_INTEGRATOR_LIMIT);
            self.lp = self
                .lp
                .clamp(-FILTER_INTEGRATOR_LIMIT, FILTER_INTEGRATOR_LIMIT);
        }

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
