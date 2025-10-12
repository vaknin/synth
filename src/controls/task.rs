//! Control tasks: potentiometers and future encoders.

use crate::config::*;
use crate::controls::{
    map_arp_mode, map_arp_rate, map_arp_transpose, map_cutoff, map_lfo_depth, map_lfo_rate,
    map_master_pitch, map_resonance, CtrlSender, Potentiometer,
};
use crate::hardware::{AdcBus, AnyAdcPin};
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads all pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Current 8-pot setup:
/// - Pot 1 (GPIO1): Filter Cutoff
/// - Pot 2 (GPIO2): Filter Resonance
/// - Pot 3 (GPIO3): Filter LFO Rate
/// - Pot 4 (GPIO4): Filter LFO Depth
/// - Pot 5 (GPIO5): Arp Rate
/// - Pot 6 (GPIO6): Arp Mode (Up/Down/UpDown/Random)
/// - Pot 7 (GPIO7): Root Transpose (±12 semitones)
/// - Pot 8 (GPIO8): Master Pitch
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `adc_bus` - ADC bus with ADC peripheral (owned by this task)
/// * `pins` - Array of type-erased ADC pins (owned by this task)
#[embassy_executor::task]
pub async fn pot_task(sender: CtrlSender, mut adc_bus: AdcBus, mut pins: [AnyAdcPin; 8]) {
    // Create pot state objects with mapping functions
    let mut pots = [
        Potentiometer::new(map_cutoff),       // Pot 1 (GPIO1)
        Potentiometer::new(map_resonance),    // Pot 2 (GPIO2)
        Potentiometer::new(map_lfo_rate),     // Pot 3 (GPIO3)
        Potentiometer::new(map_lfo_depth),    // Pot 4 (GPIO4)
        Potentiometer::new(map_arp_rate),     // Pot 5 (GPIO5)
        Potentiometer::new(map_arp_mode),     // Pot 6 (GPIO6)
        Potentiometer::new(map_arp_transpose), // Pot 7 (GPIO7)
        Potentiometer::new(map_master_pitch), // Pot 8 (GPIO8)
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
