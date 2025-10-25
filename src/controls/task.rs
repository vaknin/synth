//! Control tasks: potentiometers polling for drone synth.

use crate::config::*;
use crate::controls::{map_cutoff, map_resonance, CtrlSender, Potentiometer};
use crate::hardware::{AdcBus, AnyAdcPin};
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads 2 pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Simplified 2-pot setup for drone synth:
/// - Pot 0 (GPIO1): Filter Cutoff
/// - Pot 1 (GPIO2): Filter Resonance
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `adc_bus` - ADC bus with ADC peripheral (owned by this task)
/// * `pins` - Array of 2 type-erased ADC pins (owned by this task)
#[embassy_executor::task]
pub async fn pot_task(sender: CtrlSender, mut adc_bus: AdcBus, mut pins: [AnyAdcPin; 2]) {
    // Create pot state objects with mapping functions
    let mut pots = [
        Potentiometer::new(map_cutoff),    // GPIO1
        Potentiometer::new(map_resonance), // GPIO2
    ];

    let mut poll_count: u32 = 0;

    loop {
        poll_count += 1;
        if poll_count % 25 == 0 {
            log::warn!("🎛️ Pot task running: poll #{}", poll_count);
        }

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
