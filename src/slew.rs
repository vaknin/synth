//! Generic parameter smoother for eliminating zipper noise and clicks.
//!
//! Uses exponential moving average (EMA) / one-pole lowpass filter to smooth
//! parameter changes. Essential for frequency, volume, filter cutoff, and any
//! other parameter that changes rapidly during audio rendering.

use crate::config::{SMOOTHING_COEFF as SLEW_COEFF, SLEW_SETTLE_THRESHOLD};

/// Generic parameter smoother using exponential moving average (EMA).
///
/// Smooths parameter changes to prevent audible artifacts (clicks, pops, zipper noise).
/// Works for any parameter: frequency, volume, filter cutoff, pan, etc.
///
/// # Example
/// ```
/// let mut freq_slew = Slew::new(440.0);
/// freq_slew.set_target(880.0);
///
/// // In audio callback:
/// let smooth_freq = freq_slew.value();  // Advances and returns smoothed value
/// ```
pub struct Slew {
    /// Current smoothed value (private - access via value())
    current: f32,
    /// Target value to reach (private - access via target())
    target: f32,
    /// Smoothing coefficient
    coeff: f32,
    /// Settled flag (true when current ≈ target, skips computation)
    settled: bool,
}

impl Slew {
    /// Create a new slew limiter with initial value.
    ///
    /// Uses global SLEW_COEFF for smoothing behavior.
    ///
    /// # Arguments
    /// * `initial_value` - Starting value (also used as initial target)
    pub fn new(initial_value: f32) -> Self {
        Self {
            current: initial_value,
            target: initial_value,
            coeff: SLEW_COEFF,
            settled: true, // Start settled
        }
    }

    /// Create a new slew limiter with custom coefficient.
    ///
    /// Allows per-instance smoothing speed control (e.g., fast soft-start vs slow parameter smoothing).
    ///
    /// # Arguments
    /// * `initial_value` - Starting value (also used as initial target)
    /// * `coeff` - Smoothing coefficient (0.0-1.0). Higher = slower. Use 0.99 for ~10ms rise, 0.999 for ~100ms.
    pub fn with_coeff(initial_value: f32, coeff: f32) -> Self {
        Self {
            current: initial_value,
            target: initial_value,
            coeff,
            settled: true, // Start settled
        }
    }

    /// Set new target value to smoothly transition to.
    ///
    /// The slew limiter will exponentially approach this value over subsequent value() calls.
    #[inline]
    pub fn set_target(&mut self, target: f32) {
        if target != self.target {
            self.target = target;
            self.settled = false; // Mark unsettled when target changes
        }
    }

    /// Get the current target value.
    ///
    /// Useful for reading the target without advancing the slew state (e.g., for normalization caching).
    #[inline]
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Get current smoothed value without advancing state.
    ///
    /// Zero-cost read for audio-rate loops. Use this in per-sample loops
    /// after calling value() once at buffer rate.
    ///
    /// **Pattern:**
    /// ```
    /// // Once per buffer:
    /// self.param.value();
    ///
    /// // In sample loop:
    /// let p = self.param.current();
    /// ```
    #[inline]
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Get next smoothed value (advances state).
    ///
    /// Call once per buffer, then use current() in the sample loop.
    /// Do NOT call this at audio rate (44.1kHz) - only at buffer rate (~172Hz).
    ///
    /// **Optimization:** Skips EMA computation when settled (90% of the time).
    /// Settling threshold: when error becomes negligible (< 1/65536 for 16-bit audio).
    #[inline]
    pub fn value(&mut self) -> f32 {
        if !self.settled {
            self.tick();
        }
        self.current
    }

    /// Internal tick function - advances the slew limiter state.
    ///
    /// Called automatically by value() when not settled.
    /// Checks if settled after each iteration (error < threshold for 16-bit precision).
    #[inline]
    fn tick(&mut self) {
        // EMA formula: current = current * α + target * (1 - α)
        // This is a one-pole lowpass filter in the time domain
        self.current = self.current * self.coeff + self.target * (1.0 - self.coeff);

        // Check if settled: error < 1/65536 (negligible for 16-bit audio)
        // This happens after ~10-20 samples with coeff=0.99
        let error = (self.target - self.current).abs();
        if error < SLEW_SETTLE_THRESHOLD {
            self.current = self.target; // Snap to target
            self.settled = true;
        }
    }
}
