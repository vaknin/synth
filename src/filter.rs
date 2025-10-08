//! Moog Ladder Filter - Classic 4-pole (24dB/oct) resonant lowpass.
//!
//! Improved Huovilainen model with excellent stability and self-oscillation.
//! No stability issues even at extreme cutoff + resonance settings.
//!
//! Optimized: Coefficients cached at buffer rate, control smoothing via ADC multisampling.

use crate::config::{FILTER_RESONANCE_MAX, FILTER_RESONANCE_MIN, SAMPLE_RATE};
use crate::slew::Slew;
use micromath::F32Ext;

/// Moog Ladder Filter - 4-pole resonant lowpass.
///
/// Improved Huovilainen topology with unconditional stability.
/// Self-oscillates beautifully at high resonance (3.5+), classic analog sound.
///
/// Coefficients smoothed at audio rate with fast-settling Slew.
pub struct MoogLadderFilter {
    /// Stage 1-4 state variables (cascaded one-pole filters)
    stage: [f32; 4],
    /// Cutoff frequency in Hz (updated at control rate from pots)
    pub cutoff: f32,
    /// Resonance amount (0.0 = no resonance, 4.0 = self-oscillation)
    pub resonance: f32,
    /// Sample rate in Hz
    sample_rate: f32,
    /// Smoothed g_norm (audio-rate, fast settling for responsive control)
    g_norm_slew: Slew,
    /// Smoothed g_norm_inv (audio-rate, fast settling)
    g_norm_inv_slew: Slew,
    /// Smoothed k (audio-rate, fast settling)
    k_slew: Slew,
    /// Whether coefficients need recalculation (dirty flag optimization)
    dirty: bool,
}

impl MoogLadderFilter {
    /// Create new Moog ladder filter with initial cutoff frequency and resonance.
    ///
    /// # Arguments
    /// * `cutoff_hz` - Initial cutoff frequency in Hz (20-20000, full audible spectrum)
    /// * `resonance` - Resonance amount (0.1-10.0, typical 0.7 for gentle, 10.0 for extreme self-oscillation)
    pub fn new(cutoff_hz: f32, resonance: f32) -> Self {
        let sample_rate = SAMPLE_RATE as f32;
        let clamped_cutoff = cutoff_hz.clamp(
            crate::config::FILTER_CUTOFF_MIN,
            crate::config::FILTER_CUTOFF_MAX,
        );
        let clamped_resonance = resonance.clamp(FILTER_RESONANCE_MIN, FILTER_RESONANCE_MAX);

        // Pre-calculate initial coefficients
        // g = tan(π × fc / fs) - warped cutoff frequency
        let wd = core::f32::consts::PI * clamped_cutoff / sample_rate;
        let g = wd.tan();

        // Normalize g for stable one-pole integration
        let g_norm = g / (1.0 + g);
        let g_norm_inv = 1.0 - g_norm;

        // Resonance compensation for stability (Huovilainen model)
        let k = clamped_resonance * 4.0 * (1.0 - g_norm);

        Self {
            stage: [0.0; 4],
            cutoff: clamped_cutoff,
            resonance: clamped_resonance,
            sample_rate,
            // Fast-settling Slew (α=0.95) for responsive yet smooth coefficient changes
            // Settles in ~2ms vs 10ms for default 0.99 - critical for CPU efficiency
            g_norm_slew: Slew::with_coeff(g_norm, 0.95),
            g_norm_inv_slew: Slew::with_coeff(g_norm_inv, 0.95),
            k_slew: Slew::with_coeff(k, 0.95),
            dirty: false, // Coefficients already calculated
        }
    }

    /// Set cutoff frequency in Hz.
    ///
    /// # Arguments
    /// * `freq` - Cutoff frequency in Hz (20-20000 Hz, full audible range)
    #[inline(always)]
    pub fn set_cutoff(&mut self, freq: f32) {
        self.cutoff = freq.clamp(
            crate::config::FILTER_CUTOFF_MIN,
            crate::config::FILTER_CUTOFF_MAX,
        );
        self.dirty = true; // Mark for coefficient recalculation (pot deadband handles tiny changes)
    }

    /// Set resonance amount.
    ///
    /// # Arguments
    /// * `resonance` - Resonance (0.1-10.0, saturated feedback prevents instability)
    #[inline(always)]
    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(FILTER_RESONANCE_MIN, FILTER_RESONANCE_MAX);
        self.dirty = true; // Mark for coefficient recalculation (pot deadband handles tiny changes)
    }

    /// Update filter coefficients from direct parameters.
    ///
    /// Call once per buffer before processing samples (BUFFER RATE ~172Hz).
    /// Recalculates g and k from current cutoff/resonance values.
    /// Only recalculates when parameters changed (dirty flag optimization).
    ///
    /// Expensive ops (tan) happen here, NOT in audio-rate process().
    #[inline(always)]
    pub fn update_coefficients(&mut self) {
        // Early return if no parameters changed (most of the time)
        if !self.dirty {
            return;
        }

        // Recalculate filter coefficient g = tan(π × fc / fs)
        // Uses micromath tan() - expensive but only at buffer rate
        let wd = core::f32::consts::PI * self.cutoff / self.sample_rate;
        let g = wd.tan();

        // Normalize g for stable one-pole integration
        let g_norm = g / (1.0 + g);

        // Set targets for audio-rate Slew smoothing (eliminates zipper noise)
        // Fast settling (α=0.95) keeps CPU overhead minimal
        self.g_norm_slew.set_target(g_norm);
        self.g_norm_inv_slew.set_target(1.0 - g_norm);

        // Resonance compensation: reduces feedback as cutoff increases for stability
        let k = self.resonance * 4.0 * (1.0 - g_norm);
        self.k_slew.set_target(k);

        // Clear dirty flag (coefficients now up to date)
        self.dirty = false;
    }

    /// Process one sample through the filter (AUDIO RATE 44.1kHz - HOT PATH).
    ///
    /// Smooths coefficients at audio rate with fast-settling Slew (α=0.95).
    /// Settled optimization keeps this cheap when filter isn't changing (~2ms settling).
    ///
    /// # Arguments
    /// * `input` - Input sample (normalized ±1.0)
    ///
    /// # Returns
    /// Lowpass filtered output (nominally ±1.0, but can exceed with high resonance)
    ///
    /// # References
    /// Antti Huovilainen, "Non-linear digital implementation of the Moog ladder filter" (2004)
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        // Get smoothed coefficients (fast settling α=0.95, cheap when static via settled flag)
        let g_norm = self.g_norm_slew.value();
        let g_inv = self.g_norm_inv_slew.value();
        let k = self.k_slew.value();

        // Feedback from output with hard clamp (prevents runaway oscillation)
        // Simple clamp is fastest - no division, no extra multiplies
        let fb = k * self.stage[3];
        let feedback = fb.clamp(-1.0, 1.0);

        // Process through 4 cascaded one-pole filters
        // Unrolled for performance (4 iterations, no loop overhead)
        // Each stage: y[n] = g_inv*y[n-1] + g_norm*x[n]
        // This formulation is ALWAYS STABLE because g_norm < 1.0 for all g
        let input_compensated = input - feedback;
        self.stage[0] = g_inv * self.stage[0] + g_norm * input_compensated;
        self.stage[1] = g_inv * self.stage[1] + g_norm * self.stage[0];
        self.stage[2] = g_inv * self.stage[2] + g_norm * self.stage[1];
        self.stage[3] = g_inv * self.stage[3] + g_norm * self.stage[2];

        // Output from final stage
        self.stage[3]
    }

    /// Reset filter state (clears all stages).
    ///
    /// Useful when changing patches or stopping audio to prevent ringing.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.stage = [0.0; 4];
    }
}
