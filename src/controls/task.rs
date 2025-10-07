//! Control tasks: potentiometers and future encoders.

use crate::config::*;
use crate::controls::{map_master_pitch, map_osc_level, CtrlSender, Potentiometer};
use crate::hardware::{AdcBus, PotPin};
use crate::message::OscillatorId;
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads all pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Phase 1: Temporary 2-pot setup for testing
/// - Pot 0 (GPIO1): Master Pitch (30Hz-1024Hz)
/// - Pot 1 (GPIO2): Osc1 Level (0.0-1.0)
///
/// TODO: Expand to 7 pots for full Phase 1 oscillator control
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `adc_bus` - ADC bus with ADC peripheral (owned by this task)
/// * `pitch_pin` - Master pitch potentiometer pin (GPIO1)
/// * `level_pin` - Osc1 level potentiometer pin (GPIO2)
#[embassy_executor::task]
pub async fn pot_task(
    sender: CtrlSender,
    mut adc_bus: AdcBus,
    mut pitch_pin: PotPin<esp_hal::peripherals::GPIO1<'static>>,
    mut level_pin: PotPin<esp_hal::peripherals::GPIO2<'static>>,
) {
    // Create pot state objects with mapping functions and deadbands
    let mut pitch_pot = Potentiometer::new(map_master_pitch);
    let mut level_pot = Potentiometer::new(|n| map_osc_level(OscillatorId::Osc1, n));

    loop {
        // Poll master pitch pot (GPIO1)
        pitch_pot
            .poll_and_send(sender, &mut adc_bus.adc, &mut pitch_pin)
            .await;

        // Poll Osc1 level pot (GPIO2)
        level_pot
            .poll_and_send(sender, &mut adc_bus.adc, &mut level_pin)
            .await;

        // Wait before next poll cycle
        Timer::after(Duration::from_millis(ADC_POLL_INTERVAL_MS)).await;
    }
}
