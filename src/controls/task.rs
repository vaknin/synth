//! Control tasks: potentiometers and future encoders.

use crate::config::*;
use crate::controls::{
    map_bass_euclidean_hits, map_bass_euclidean_length, map_cutoff, map_lead_euclidean_hits,
    map_lead_euclidean_length, map_mutation_rate, map_resonance, map_tempo, CtrlSender,
    Potentiometer,
};
use crate::hardware::{AdcBus, AnyAdcPin};
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads all pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Current 8-pot setup (generative phase):
/// - Pot 0 (GPIO1): Filter Cutoff
/// - Pot 1 (GPIO2): Filter Resonance
/// - Pot 2 (GPIO3): Bass Pattern Length (4-16 steps)
/// - Pot 3 (GPIO4): Bass Hit Density (1-16 hits)
/// - Pot 4 (GPIO5): Lead Pattern Length (3-17 steps)
/// - Pot 5 (GPIO6): Lead Hit Density (1-17 hits)
/// - Pot 6 (GPIO7): Tempo (80-180 BPM)
/// - Pot 7 (GPIO8): Mutation Rate (0-50%)
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `adc_bus` - ADC bus with ADC peripheral (owned by this task)
/// * `pins` - Array of type-erased ADC pins (owned by this task)
#[embassy_executor::task]
pub async fn pot_task(sender: CtrlSender, mut adc_bus: AdcBus, mut pins: [AnyAdcPin; 8]) {
    // Create pot state objects with mapping functions
    let mut pots = [
        Potentiometer::new(map_cutoff),
        Potentiometer::new(map_resonance),
        Potentiometer::new(map_bass_euclidean_length),
        Potentiometer::new(map_bass_euclidean_hits),
        Potentiometer::new(map_lead_euclidean_length),
        Potentiometer::new(map_lead_euclidean_hits),
        Potentiometer::new(map_tempo),
        Potentiometer::new(map_mutation_rate),
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
