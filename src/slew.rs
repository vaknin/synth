//! Generic parameter smoother for eliminating zipper noise and clicks.
//!
//! Uses exponential moving average (EMA) / one-pole lowpass filter to smooth
//! parameter changes. Essential for frequency, volume, filter cutoff, and any
//! other parameter that changes rapidly during audio rendering.

use crate::config::SMOOTHING_COEFF as SLEW_COEFF;

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
    /// Target value to reach
    target: f32,
    /// Smoothing coefficient
    coeff: f32
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
            coeff: SLEW_COEFF
        }
    }

    /// Set new target value to smoothly transition to.
    ///
    /// The slew limiter will exponentially approach this value over subsequent value() calls.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Get next smoothed value.
    ///
    /// Advances the internal state and returns the current smoothed value.
    /// Call once per audio sample when you need the parameter value.
    #[inline]
    pub fn value(&mut self) -> f32 {
        self.tick();
        self.current
    }

    /// Internal tick function - advances the slew limiter state.
    ///
    /// Called automatically by value(). Not meant to be used directly.
    #[inline]
    fn tick(&mut self) {
        // EMA formula: current = current * α + target * (1 - α)
        // This is a one-pole lowpass filter in the time domain
        self.current = self.current * self.coeff + self.target * (1.0 - self.coeff);
    }
}
