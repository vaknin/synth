//! Control tasks: potentiometers, buttons, and future encoders.

use crate::config::*;
use crate::controls::{map_freq, map_vol, CtrlSender, Potentiometer};
use crate::hardware::{AdcBus, PotPin};
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads all pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Sequential polling is standard for ADC inputs because:
/// - Only one ADC peripheral exists (can't poll in parallel)
/// - Potentiometers are slow-changing (15ms poll rate is plenty)
/// - ADC read takes ~1-2μs vs 15ms sleep → overhead is negligible
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `adc_bus` - ADC bus with ADC peripheral (owned by this task)
/// * `freq_pin` - Frequency potentiometer pin (GPIO1)
/// * `vol_pin` - Volume potentiometer pin (GPIO2)
#[embassy_executor::task]
pub async fn pot_task(
    sender: CtrlSender,
    mut adc_bus: AdcBus,
    mut freq_pin: PotPin<esp_hal::peripherals::GPIO1<'static>>,
    mut vol_pin: PotPin<esp_hal::peripherals::GPIO2<'static>>,
) {
    // Create pot state objects with mapping functions and deadbands
    let mut freq_pot = Potentiometer::new(map_freq);
    let mut vol_pot = Potentiometer::new(map_vol);

    loop {
        // Poll frequency pot (GPIO1)
        freq_pot
            .poll_and_send(sender, &mut adc_bus.adc, &mut freq_pin)
            .await;

        // Poll volume pot (GPIO2)
        vol_pot
            .poll_and_send(sender, &mut adc_bus.adc, &mut vol_pin)
            .await;

        // Wait before next poll cycle
        Timer::after(Duration::from_millis(ADC_POLL_INTERVAL_MS)).await;
    }
}
