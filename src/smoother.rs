//! Generic parameter smoother for eliminating zipper noise and clicks.
//!
//! Uses exponential moving average (EMA) / one-pole lowpass filter to smooth
//! parameter changes. Essential for frequency, volume, filter cutoff, and any
//! other parameter that changes rapidly during audio rendering.

use crate::config::SMOOTHING_COEFF;

/// Generic parameter smoother using exponential moving average (EMA).
///
/// Smooths parameter changes to prevent audible artifacts (clicks, pops, zipper noise).
/// Works for any parameter: frequency, volume, filter cutoff, pan, etc.
///
/// # Example
/// ```
/// let mut freq_smoother = Smoother::new(440.0);
/// freq_smoother.set_target(880.0);
///
/// // In audio callback:
/// freq_smoother.tick();
/// let smooth_freq = freq_smoother.current;
/// ```
pub struct Smoother {
    /// Current smoothed value (read directly, update via tick())
    pub current: f32,
    /// Target value to reach
    target: f32,
    coeff: f32
}

impl Smoother {
    /// Create a new smoother with initial value.
    ///
    /// Uses global PARAM_SMOOTHING_COEFF for smoothing behavior.
    ///
    /// # Arguments
    /// * `initial_value` - Starting value (also used as initial target)
    pub fn new(initial_value: f32) -> Self {
        Self {
            current: initial_value,
            target: initial_value,
            coeff: SMOOTHING_COEFF
        }
    }

    /// Set new target value to smoothly transition to.
    ///
    /// The smoother will exponentially approach this value over subsequent tick() calls.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Generate next smoothed value.
    ///
    /// Call once per audio sample to update the smoothed parameter value.
    #[inline]
    pub fn tick(&mut self) {
        // EMA formula: current = current * α + target * (1 - α)
        // This is a one-pole lowpass filter in the time domain
        self.current = self.current * self.coeff + self.target * (1.0 - self.coeff);
    }
}
