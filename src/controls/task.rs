//! Control tasks: potentiometers and future encoders.

use crate::config::*;
use crate::controls::{
    map_bass_cutoff, map_bass_lfo_depth, map_bass_lfo_rate, map_bass_resonance, map_decay,
    map_lead_cutoff, map_lead_euclidean_hits, map_lead_resonance, CtrlSender, Potentiometer,
};
use crate::hardware::{AdcBus, AnyAdcPin};
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads all pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Current 8-pot setup (per-voice filters for spectral separation):
///
/// **Bass Zone (GPIO1-4):**
/// - Pot 1 (GPIO1): Bass Filter Cutoff
/// - Pot 2 (GPIO2): Bass Filter Resonance
/// - Pot 3 (GPIO3): Bass LFO Rate (evolving drone)
/// - Pot 4 (GPIO4): Bass LFO Depth
///
/// **Lead Zone (GPIO5-8):**
/// - Pot 5 (GPIO5): Lead Filter Cutoff
/// - Pot 6 (GPIO6): Lead Filter Resonance
/// - Pot 7 (GPIO7): Lead Envelope Decay (attack hardcoded to 5ms)
/// - Pot 8 (GPIO8): Lead Euclidean Hits (rhythm density)
///
/// Fixed generative parameters:
/// - Tempo: CLOCK_BPM_DEFAULT
/// - Scale: Phrygian (dark, hypnotic)
/// - Portamento: BASS_PORTAMENTO_COEFF, LEAD_PORTAMENTO_COEFF
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `adc_bus` - ADC bus with ADC peripheral (owned by this task)
/// * `pins` - Array of type-erased ADC pins (owned by this task)
#[embassy_executor::task]
pub async fn pot_task(sender: CtrlSender, mut adc_bus: AdcBus, mut pins: [AnyAdcPin; 8]) {
    // Create pot state objects with mapping functions (split bass/lead zones)
    let mut pots = [
        // Bass Zone (GPIO1-4)
        Potentiometer::new(map_bass_cutoff),         // Pot 1 (GPIO1)
        Potentiometer::new(map_bass_resonance),      // Pot 2 (GPIO2)
        Potentiometer::new(map_bass_lfo_rate),       // Pot 3 (GPIO3)
        Potentiometer::new(map_bass_lfo_depth),      // Pot 4 (GPIO4)
        // Lead Zone (GPIO5-8)
        Potentiometer::new(map_lead_cutoff),         // Pot 5 (GPIO5)
        Potentiometer::new(map_lead_resonance),      // Pot 6 (GPIO6)
        Potentiometer::new(map_decay),               // Pot 7 (GPIO7)
        Potentiometer::new(map_lead_euclidean_hits), // Pot 8 (GPIO8)
    ];

    loop {
        // Poll each pot sequentially
        // Note: This task runs on Core 1 (isolated from audio on Core 0)
        // Blocking ADC reads here don't affect audio streaming on the other core
        for (pot, pin) in pots.iter_mut().zip(pins.iter_mut()) {
            pot.poll_and_send(sender, pin, &mut adc_bus.adc).await;
        }

        // Wait before next poll cycle
        Timer::after(Duration::from_millis(ADC_POLL_INTERVAL_MS)).await;
    }
}
