//! Potentiometer reading with EMA filtering and parameter mapping.

use crate::config::*;
use crate::controls::CtrlSender;
use crate::hardware::PotPin;
use crate::message::Message;
use esp_hal::analog::adc::{Adc, AdcChannel};
use esp_hal::peripherals::ADC1;
use esp_hal::Blocking;

/// Potentiometer with filtering, deadband, and parameter mapping.
///
/// Each pot owns its own signal processing state (EMA filter, deadband),
/// sample buffer, and behavior (mapping function). Hardware (ADC, pins)
/// is owned by the control task that polls all pots sequentially.
pub struct Potentiometer {
    /// EMA filtered value
    filtered: f32,
    /// Filter coefficient (0.0 to 1.0, higher = more smoothing)
    alpha: f32,
    /// Last sent normalized value (for deadband detection)
    last_sent: f32,
    /// Deadband threshold (only send if change >= this)
    threshold: f32,
    /// Mapping function from normalized value to Message
    map_fn: fn(f32) -> Message,
    /// Sample buffer for multisampling (reused each poll)
    samples: [u16; ADC_MULTISAMPLING_COUNT],
}

impl Potentiometer {
    /// Create new potentiometer with mapping function and deadband.
    ///
    /// # Arguments
    /// * `map_fn` - Function to map normalized value (0.0-1.0) to Message
    /// * `threshold` - Deadband threshold (0.0 = always send, typical: POT_CHANGE_THRESHOLD)
    pub fn new(map_fn: fn(f32) -> Message, threshold: f32) -> Self {
        Self {
            filtered: ((POT_MIN + POT_MAX) / 2) as f32,
            alpha: ADC_EMA_ALPHA,
            last_sent: 0.0,
            threshold,
            map_fn,
            samples: [0u16; ADC_MULTISAMPLING_COUNT],
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
    /// * `adc` - ADC peripheral (borrowed from control task)
    /// * `pin` - Potentiometer GPIO pin (borrowed from control task)
    pub async fn poll_and_send<P>(
        &mut self,
        sender: &CtrlSender,
        adc: &mut Adc<'static, ADC1<'static>, Blocking>,
        pin: &mut PotPin<P>,
    )
    where
        P: AdcChannel,
    {
        // 1. Multisample: read N samples into internal buffer
        for sample in self.samples.iter_mut() {
            *sample = adc.read_blocking(pin);
        }

        // 2. Average samples (multisampling reduces noise)
        let sum: u32 = self.samples.iter().map(|&s| s as u32).sum();
        let avg = (sum / self.samples.len() as u32) as f32;

        // 3. Apply EMA filter: filtered = alpha * filtered + (1-alpha) * new
        self.filtered = self.filtered * self.alpha + avg * (1.0 - self.alpha);

        // 4. Normalize to 0.0-1.0 range using calibrated min/max
        let normalized = ((self.filtered as u16).saturating_sub(POT_MIN) as f32
            / (POT_MAX - POT_MIN) as f32)
            .clamp(0.0, 1.0);

        // 5. Deadband check: only send if changed significantly
        if (normalized - self.last_sent).abs() >= self.threshold {
            self.last_sent = normalized;
            sender.send((self.map_fn)(normalized)).await;
        }
    }
}

/// Map normalized potentiometer value to frequency (Hz).
///
/// Uses linear mapping from FREQUENCY_MIN to FREQUENCY_MAX.
/// (Exponential mapping would require libm in no_std)
pub fn map_freq(normalized: f32) -> Message {
    let freq = FREQUENCY_MIN + normalized * (FREQUENCY_MAX - FREQUENCY_MIN);
    Message::SetFrequency(freq)
}

/// Map normalized potentiometer value to volume (0.0-1.0).
///
/// Direct linear mapping: pot position = volume level.
pub fn map_vol(normalized: f32) -> Message {
    Message::SetVolume(normalized.clamp(0.0, 1.0))
}
