//! Voice module: instrument instance with oscillator, volume, and active state.

use crate::{config::VOLUME_SMOOTHING_COEFF, oscillator::Oscillator};

/// A single voice in the synth.
/// Wraps an oscillator with volume control and active state.
pub struct Voice {
    /// Wavetable oscillator for audio generation
    osc: Oscillator,

    /// Target volume set by user (0.0 = silent, 1.0 = full scale)
    volume_target: f32,

    /// Current smoothed volume (interpolated toward target)
    /// Updated each tick() to eliminate zipper noise
    volume_current: f32,

    /// Whether voice is active (on) or inactive (off)
    /// When inactive, tick() returns 0.0 regardless of volume
    pub active: bool,
}

impl Voice {
    /// Create a new voice at specified frequency.
    ///
    /// # Arguments
    /// * `frequency` - Initial frequency in Hz
    /// * `sample_rate` - Audio sample rate in Hz
    ///
    /// # Returns
    /// Voice with specified frequency, DEFAULT_VOLUME, inactive state
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        let default_vol = crate::config::DEFAULT_VOICE_VOLUME;
        Self {
            osc: Oscillator::new(frequency, sample_rate),
            volume_target: default_vol,
            volume_current: default_vol,
            active: false,
        }
    }

    /// Set voice frequency in Hz.
    pub fn set_frequency(&mut self, freq: f32) {
        self.osc.set_frequency(freq);
    }

    /// Set target volume (0.0 to 1.0).
    /// Actual volume will smoothly interpolate to this target to prevent clicks.
    pub fn set_volume(&mut self, vol: f32) {
        self.volume_target = vol.clamp(0.0, 1.0);
    }

    /// Set voice active state.
    /// true = voice plays, false = voice silent (but retains frequency/volume)
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Generate next audio sample.
    ///
    /// # Returns
    /// Audio sample (-1.0 to 1.0) scaled by smoothed volume, or 0.0 if inactive
    pub fn tick(&mut self) -> f32 {
        
        if self.active {
            // Smooth volume using exponential moving average
            // This eliminates zipper noise from instant parameter changes
            self.volume_current = self.volume_current * VOLUME_SMOOTHING_COEFF
                + self.volume_target * (1.0 - VOLUME_SMOOTHING_COEFF);
            self.osc.tick() * self.volume_current
        } else {
            0.0
        }
    }
}
