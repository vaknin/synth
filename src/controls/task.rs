//! Control tasks: potentiometers and future encoders.

use crate::config::*;
use crate::controls::{
    map_bass_euclidean_hits, map_cutoff, map_lead_euclidean_hits, map_lead_euclidean_length,
    map_mutation_rate, map_portamento, map_resonance, map_tempo, CtrlSender, Potentiometer,
};
use crate::hardware::{AdcBus, AnyAdcPin};
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads all pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Current 8-pot setup (generative phase):
/// - Pot 1 (GPIO1): Filter Cutoff
/// - Pot 2 (GPIO2): Filter Resonance
/// - Pot 3 (GPIO3): Tempo
/// - Pot 4 (GPIO4): Mutation Rate
/// - Pot 5 (GPIO5): Portamento Time (pitch glide speed)
/// - Pot 6 (GPIO6): Lead Pattern Length
/// - Pot 7 (GPIO7): Bass Hit Density (pot min = mute, length fixed)
/// - Pot 8 (GPIO8): Lead Hit Density (pot min = mute)
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `adc_bus` - ADC bus with ADC peripheral (owned by this task)
/// * `pins` - Array of type-erased ADC pins (owned by this task)
#[embassy_executor::task]
pub async fn pot_task(sender: CtrlSender, mut adc_bus: AdcBus, mut pins: [AnyAdcPin; 8]) {
    // Create pot state objects with mapping functions
    let mut pots = [
        Potentiometer::new(map_cutoff),                // Pot 1 (GPIO1)
        Potentiometer::new(map_resonance),             // Pot 2 (GPIO2)
        Potentiometer::new(map_tempo),                 // Pot 3 (GPIO3)
        Potentiometer::new(map_mutation_rate),         // Pot 4 (GPIO4)
        Potentiometer::new(map_portamento),            // Pot 5 (GPIO5)
        Potentiometer::new(map_lead_euclidean_length), // Pot 6 (GPIO6)
        Potentiometer::new(map_bass_euclidean_hits),   // Pot 7 (GPIO7)
        Potentiometer::new(map_lead_euclidean_hits),   // Pot 8 (GPIO8)
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
