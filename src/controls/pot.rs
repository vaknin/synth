//! Potentiometer reading with EMA filtering and parameter mapping.

use crate::config::*;
use crate::controls::CtrlSender;
use crate::message::Message;
use log::warn;
use micromath::F32Ext;

/// Potentiometer with filtering, deadband, and parameter mapping.
///
/// Each pot owns its own signal processing state (deadband, multisampling),
/// sample buffer, and behavior (mapping function). Hardware (ADC, pins)
/// is owned by the control task that polls all pots sequentially.
///
/// DIAGNOSTIC: Re-enabling ADC-side EMA to test if its removal caused crackling.
pub struct Potentiometer {
    /// Last sent normalized value (for deadband detection)
    last_sent: f32,
    /// Mapping function from normalized value to Message
    map_fn: fn(f32) -> Message,
    /// Sample buffer for multisampling (reused each poll)
    samples: [u16; ADC_MULTISAMPLING_COUNT],
    /// EMA filtered value (ADC-side smoothing)
    filtered: f32,
    /// EMA alpha coefficient
    alpha: f32,
}

impl Potentiometer {
    /// Create new potentiometer with mapping function and deadband.
    ///
    /// # Arguments
    /// * `map_fn` - Function to map normalized value (0.0-1.0) to Message
    pub fn new(map_fn: fn(f32) -> Message) -> Self {
        Self {
            last_sent: 0.0,
            map_fn,
            samples: [0u16; ADC_MULTISAMPLING_COUNT],
            filtered: (POT_MAX / 2) as f32,  // Initialize to midpoint
            alpha: ADC_EMA_ALPHA,
        }
    }

    /// Read, filter, and conditionally send message if value changed significantly.
    ///
    /// Performs complete signal chain:
    /// 1. Multisampling (reduces noise by √N)
    /// 2. Averaging
    /// 3. EMA filtering (smooth out remaining noise)
    /// 4. Normalization (POT_MIN..POT_MAX → 0.0..1.0)
    /// 5. Deadband check (only send if change >= threshold)
    /// 6. Message mapping and send
    ///
    /// # Arguments
    /// * `sender` - Embassy channel sender
    /// * `pin` - Type-erased ADC pin (contains ADC reference internally)
    pub async fn poll_and_send(
        &mut self,
        sender: CtrlSender,
        pin: &mut crate::hardware::AnyAdcPin,
        adc: &mut crate::hardware::Adc1,
    ) {
        // 1. Multisample: read N samples into internal buffer
        for sample in self.samples.iter_mut() {
            *sample = pin.read_blocking(adc);
        }

        // 2. Average samples (multisampling reduces noise)
        let sum: u32 = self.samples.iter().map(|&s| s as u32).sum();
        let avg = (sum / self.samples.len() as u32) as f32;

        // 3. DIAGNOSTIC: Re-enable ADC-side EMA filtering to test if removal caused crackling
        self.filtered = self.filtered * self.alpha + avg * (1.0 - self.alpha);

        // 4. Normalize to 0.0-1.0 range using calibrated min/max, defensive clamping
        let normalized = (self.filtered / (POT_MAX as f32)).clamp(0.0, 1.0);

        // 5. Deadband check: only send if changed significantly
        if (normalized - self.last_sent).abs() >= POT_CHANGE_THRESHOLD {
            self.last_sent = normalized;
            let msg = (self.map_fn)(normalized);
            if let Err(e) = sender.try_send(msg) {
                warn!("Pot message dropped (queue full): {:?}", e);
            }
        }
    }
}

// === Phase 1: Oscillator Control Mappings ===

/// Map normalized value (0.0-1.0) to master pitch frequency (30Hz-1024Hz), logarithmically.
pub fn map_master_pitch(normalized: f32) -> Message {
    let curved = normalized.powi(POT_EXPONENT_SCALE);
    let freq = FREQUENCY_MIN + (FREQUENCY_MAX - FREQUENCY_MIN) * curved;
    Message::SetMasterPitch(freq)
}

/// Map normalized value (0.0-1.0) to oscillator level.
pub fn map_osc_level(osc_id: crate::message::OscillatorId, normalized: f32) -> Message {
    Message::SetOscLevel(osc_id, normalized)
}

/// Map normalized value (0.0-1.0) to detune in cents (configurable range).
pub fn map_osc_detune(osc_id: crate::message::OscillatorId, normalized: f32) -> Message {
    // Map 0.0-1.0 to -DETUNE_RANGE_CENTS to +DETUNE_RANGE_CENTS (centered at 0.5)
    // Adjust DETUNE_RANGE_CENTS in config.rs for different detune intensities
    let cents = (normalized - 0.5) * (DETUNE_RANGE_CENTS * 2.0);
    Message::SetOscDetune(osc_id, cents)
}

/// Map normalized value (0.0-1.0) to filter cutoff frequency, exponentially.
pub fn map_cutoff(normalized: f32) -> Message {
    use micromath::F32Ext;
    use crate::config::{FILTER_CUTOFF_MIN, FILTER_CUTOFF_MAX};

    // Exponential mapping for musical frequency perception
    // Uses config constants for min/max range (safe margin below Nyquist for filter stability)
    let curved = normalized.powi(POT_EXPONENT_SCALE);
    let freq = FILTER_CUTOFF_MIN * (FILTER_CUTOFF_MAX / FILTER_CUTOFF_MIN).powf(curved);
    Message::SetCutoff(freq)
}
