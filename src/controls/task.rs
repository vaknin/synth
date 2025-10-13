//! Control tasks: potentiometers and future encoders.

use crate::config::*;
use crate::controls::{
    map_attack, map_cutoff, map_decay, map_filter_envelope_depth, map_lead_euclidean_hits,
    map_lfo_depth, map_lfo_rate, map_resonance, CtrlSender, Potentiometer,
};
use crate::hardware::{AdcBus, AnyAdcPin};
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads all pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Current 8-pot setup (bass drone + lead generative):
/// - Pot 1 (GPIO1): Filter Cutoff (main performance control)
/// - Pot 2 (GPIO2): Filter Resonance (Moog character)
/// - Pot 3 (GPIO3): Filter Envelope Depth (rhythm-synced plucks)
/// - Pot 4 (GPIO4): Filter LFO Rate (sweep speed)
/// - Pot 5 (GPIO5): Filter LFO Depth (modulation amount)
/// - Pot 6 (GPIO6): Envelope Attack (lead articulation)
/// - Pot 7 (GPIO7): Envelope Decay (lead sustain)
/// - Pot 8 (GPIO8): Lead Euclidean Hits (rhythm density)
///
/// Fixed generative parameters:
/// - Bass: Continuous drone, see BASS_* constants in config.rs
/// - Lead: Euclidean rhythm, see LEAD_* constants in config.rs
/// - Tempo: CLOCK_BPM_DEFAULT (hardcoded for now)
/// - Scale: Phrygian (dark, hypnotic, perfect for techno)
/// - Portamento: See BASS_PORTAMENTO_COEFF and LEAD_PORTAMENTO_COEFF
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
        Potentiometer::new(map_filter_envelope_depth), // Pot 3 (GPIO3)
        Potentiometer::new(map_lfo_rate),              // Pot 4 (GPIO4)
        Potentiometer::new(map_lfo_depth),             // Pot 5 (GPIO5)
        Potentiometer::new(map_attack),                // Pot 6 (GPIO6)
        Potentiometer::new(map_decay),                 // Pot 7 (GPIO7)
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
