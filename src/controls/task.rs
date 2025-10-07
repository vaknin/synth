//! Control tasks: potentiometers and future encoders.

use crate::config::*;
use crate::controls::{map_master_pitch, map_osc_detune, map_osc_level, map_resonance, CtrlSender, Potentiometer};
use crate::hardware::{AdcBus, AnyAdcPin};
use crate::message::OscillatorId;
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads all pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Current 6-pot setup:
/// - Pot 0 (GPIO1): Master Pitch (30Hz-1024Hz)
/// - Pot 1 (GPIO2): Osc1 Level (0.0-1.0)
/// - Pot 2 (GPIO3): Osc2 Level (0.0-1.0)
/// - Pot 3 (GPIO4): Osc2 Detune (-100¢ to +100¢)
/// - Pot 4 (GPIO5): Filter Cutoff (20Hz-7kHz)
/// - Pot 5 (GPIO6): Filter Resonance (0.1-1.2)
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `adc_bus` - ADC bus with ADC peripheral (owned by this task)
/// * `pins` - Array of type-erased ADC pins (owned by this task)
#[embassy_executor::task]
pub async fn pot_task(sender: CtrlSender, mut adc_bus: AdcBus, mut pins: [AnyAdcPin; 6]) {
    // Create pot state objects with mapping functions
    let mut pots = [
        Potentiometer::new(map_master_pitch),
        Potentiometer::new(|n| map_osc_level(OscillatorId::Osc1, n)),
        Potentiometer::new(|n| map_osc_level(OscillatorId::Osc2, n)),
        Potentiometer::new(|n| map_osc_detune(OscillatorId::Osc2, n)),
        Potentiometer::new(crate::controls::map_cutoff),
        Potentiometer::new(map_resonance),
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
