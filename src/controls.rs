//! Control input handling: ADC reading, filtering, and parameter mapping.

use crate::config::*;
use crate::message::Message;
use embassy_time::{Duration, Timer};
use log::info;

#[derive(PartialEq)]
pub enum PotType {
    Frequency,
    Volume
}

/// Potentiometer reader with EMA filtering and normalization.
pub struct Potentiometer {
    /// EMA filtered value
    filtered: f32,
    /// Filter coefficient (0.0 to 1.0, higher = more smoothing)
    alpha: f32,
    last_value: f32,
    /// Minimum ADC value (calibrated)
    min: u16,
    /// Maximum ADC value (calibrated)
    max: u16,
    pot_type: PotType
}

impl Potentiometer {
    /// Create new potentiometer reader with calibrated range.
    pub fn new(pot_type: PotType) -> Self {
        Self {
            filtered: ((POT_MIN + POT_MAX) / 2) as f32, // Initialize at calibrated midpoint
            alpha: ADC_EMA_ALPHA,
            last_value: 0.0,
            min: POT_MIN,
            max: POT_MAX,
            pot_type
        }
    }

    /// Read and process ADC samples with multisampling.
    ///
    /// Averages multiple samples for noise reduction, then applies EMA filtering
    /// and normalizes to 0.0-1.0 range.
    ///
    /// # Arguments
    /// * `samples` - Array of raw ADC readings from hardware (multisampling)
    ///
    /// # Returns
    /// Normalized value (0.0 to 1.0)
    pub fn read(&mut self, samples: &[u16]) -> f32 {
        // Calculate average of samples (multisampling reduces noise by √N)
        let sum: u32 = samples.iter().map(|&s| s as u32).sum();
        let avg = (sum / samples.len() as u32) as f32;
        
        // Apply EMA filter: filtered = filtered * alpha + new * (1 - alpha)
        self.filtered = self.filtered * self.alpha + avg * (1.0 - self.alpha);
        
        // Normalize to 0.0-1.0 range
        // let value = ((avg as u16).saturating_sub(self.min) as f32 / (self.max - self.min) as f32).clamp(0.0, 1.0);
        let value2 = ((self.filtered as u16).saturating_sub(self.min) as f32 / (self.max - self.min) as f32).clamp(0.0, 1.0);
        // if self.pot_type == PotType::Frequency {
        //     // info!("avg: {value}, ema: {value2}");
        //     info!("EMA: {value2}");
        // }
        value2
    }
}

/// Map normalized pot value (0.0-1.0) to frequency range (Hz).
///
/// Uses linear mapping for simplicity (exponential requires libm in no_std).
/// Range is defined by FREQUENCY_MIN and FREQUENCY_MAX in config.
///
/// # Arguments
/// * `normalized` - Input value from 0.0 to 1.0
///
/// # Returns
/// Frequency in Hz (FREQUENCY_MIN to FREQUENCY_MAX)
pub fn pot_to_frequency(normalized: f32) -> f32 {
    FREQUENCY_MIN + normalized * (FREQUENCY_MAX - FREQUENCY_MIN)
}

/// Map normalized pot value (0.0-1.0) to volume (0.0-1.0).
///
/// Direct linear mapping - pot position = volume level.
///
/// # Arguments
/// * `normalized` - Input value from 0.0 to 1.0
///
/// # Returns
/// Volume from 0.0 (silent) to 1.0 (max)
pub fn pot_to_volume(normalized: f32) -> f32 {
    normalized.clamp(0.0, 1.0)
}

/// Control task: reads both potentiometers and sends messages to synth.
///
/// - GPIO1 (freq pot) → SetFrequency message
/// - GPIO2 (vol pot) → SetVolume message
#[embassy_executor::task]
pub async fn control_task(
    mut producer: heapless::spsc::Producer<'static, Message>,
    mut adc_ctrl: crate::hardware::AdcController,
) {
    let mut freq_pot = Potentiometer::new(PotType::Frequency);
    // let mut vol_pot = Potentiometer::new(PotType::Volume);
    let mut freq_samples = [0u16; ADC_MULTISAMPLING_COUNT];
    // let mut vol_samples = [0u16; ADC_MULTISAMPLING_COUNT];

    loop {
        // Read frequency pot (GPIO1) with multisampling
        for sample in freq_samples.iter_mut() {
            *sample = adc_ctrl.adc.read_blocking(&mut adc_ctrl.freq_pin);
        }
        let freq_current_value = freq_pot.read(&freq_samples);
        if (freq_current_value - freq_pot.last_value).abs() >= POT_CHANGE_THRESHOLD {
            freq_pot.last_value = freq_current_value;
            let freq = pot_to_frequency(freq_current_value);
            info!("new freq: {freq}");
            producer.enqueue(Message::SetFrequency(freq)).ok();
        }
        // else {
        //     info!("not enough, diff: {}", (freq_current_value - freq_pot.last_value).abs());
        // }

        // Read volume pot (GPIO2) with multisampling
        // for sample in vol_samples.iter_mut() {
        //     *sample = adc_ctrl.adc.read_blocking(&mut adc_ctrl.vol_pin);
        // }
        // let vol_norm = vol_pot.read(&vol_samples);
        // let vol = pot_to_volume(vol_norm);
        // producer.enqueue(Message::SetVolume(vol)).ok();

        Timer::after(Duration::from_millis(ADC_POLL_INTERVAL_MS)).await;
    }
}

// / Button control task for a single voice.
// /
// / Handles button press with toggle logic:
// / - First press: Select voice
// / - Second press (when already selected): Toggle voice on/off
// /
// / # Arguments
// / * `producer` - Message queue producer for sending control messages
// / * `button` - GPIO input configured with pull-up (active-low)
// / * `led` - GPIO output for visual feedback
// / * `voice_idx` - Voice index (0-2)
// #[embassy_executor::task]
// pub async fn button_task(
//     mut producer: heapless::spsc::Producer<'static, Message, MESSAGE_QUEUE_SIZE>,
//     mut button: Input<'static>,
//     mut led: Output<'static>,
//     voice_idx: u8,
// ) {
//     let mut is_selected = false;

//     loop {
//         // Wait for button press (active-low, so wait for LOW)
//         button.wait_for_low().await;
//         info!("Button {} pressed!", voice_idx);

//         if !is_selected {
//             // First press: select voice
//             producer.enqueue(Message::SelectVoice(voice_idx)).ok();
//             led.set_high();
//             is_selected = true;
//             info!("Voice {} selected", voice_idx);
//         } else {
//             // Second press: toggle voice on/off
//             producer.enqueue(Message::ToggleVoice(voice_idx)).ok();
//             info!("Voice {} toggled", voice_idx);
//         }

//         // Wait for button release before next press
//         button.wait_for_high().await;
//         info!("Button {} released", voice_idx);
//     }
// }
