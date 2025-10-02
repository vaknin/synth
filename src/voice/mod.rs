//! Voice module: complete voice unit with audio synthesis and hardware controls.
//!
//! Each Voice owns:
//! - Audio synthesis (oscillator, volume, active state)
//! - Button input (for select/toggle control)
//! - LED output (for status indication)

use crate::oscillator::Oscillator;
use esp_hal::gpio::{Input, Output};

/// Complete voice unit: audio synthesis + hardware controls.
///
/// Combines audio generation (oscillator, volume, active state) with
/// physical hardware controls (button, LED) into a self-contained unit.
pub struct Voice {
    // === Audio Synthesis ===
    /// Wavetable oscillator for audio generation
    osc: Oscillator,

    /// User-set volume (0.0 = silent, 1.0 = full scale)
    volume: f32,

    /// Whether voice is active (on) or inactive (off)
    /// When inactive, tick() returns 0.0 regardless of volume
    pub active: bool,

    // === Hardware Controls ===
    /// Voice ID (0-based)
    id: u8,

    /// Button input for voice control
    button: Input<'static>,

    /// LED output for status indication
    led: Output<'static>,
}

impl Voice {
    /// Create a new voice with audio synthesis and hardware controls.
    ///
    /// # Arguments
    /// * `id` - Voice ID (0-based)
    /// * `frequency` - Initial frequency in Hz
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `button` - GPIO input configured for button (with pull-up)
    /// * `led` - GPIO output configured for LED
    ///
    /// # Returns
    /// Voice with specified parameters, DEFAULT_VOLUME, inactive state
    pub fn new(
        id: u8,
        frequency: f32,
        sample_rate: f32,
        button: Input<'static>,
        led: Output<'static>,
    ) -> Self {
        Self {
            osc: Oscillator::new(frequency, sample_rate),
            volume: crate::config::DEFAULT_VOLUME,
            active: false,
            id,
            button,
            led,
        }
    }

    /// Get voice ID.
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Get reference to button input.
    pub fn button(&self) -> &Input<'static> {
        &self.button
    }

    /// Get mutable reference to button input.
    pub fn button_mut(&mut self) -> &mut Input<'static> {
        &mut self.button
    }

    /// Get reference to LED output.
    pub fn led(&self) -> &Output<'static> {
        &self.led
    }

    /// Get mutable reference to LED output.
    pub fn led_mut(&mut self) -> &mut Output<'static> {
        &mut self.led
    }

    /// Set voice frequency in Hz.
    pub fn set_frequency(&mut self, freq: f32) {
        self.osc.set_frequency(freq);
    }

    /// Set voice volume (0.0 to 1.0).
    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
    }

    /// Set voice active state.
    /// true = voice plays, false = voice silent (but retains frequency/volume)
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Generate next audio sample.
    ///
    /// # Returns
    /// Audio sample (-1.0 to 1.0) scaled by volume, or 0.0 if inactive
    pub fn tick(&mut self) -> f32 {
        if self.active {
            self.osc.tick() * self.volume
        } else {
            0.0
        }
    }
}
