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
/// - Pot 1 (GPIO1): Filter Cutoff (main performance control, 200-15000 Hz)
/// - Pot 2 (GPIO2): Filter Resonance (Moog character, 0.1-4.0)
/// - Pot 3 (GPIO3): Filter Envelope Depth (rhythm-synced plucks, 0-500 Hz)
/// - Pot 4 (GPIO4): Filter LFO Rate (sweep speed, 0.01-10 Hz)
/// - Pot 5 (GPIO5): Filter LFO Depth (modulation amount, 0-100%)
/// - Pot 6 (GPIO6): Envelope Attack (lead articulation, 1-200ms)
/// - Pot 7 (GPIO7): Envelope Decay (lead sustain, 10-2000ms)
/// - Pot 8 (GPIO8): Lead Euclidean Hits (rhythm density, 0-16)
///
/// Fixed generative parameters:
/// - Bass: Continuous drone, 10-bit Turing, 2% mutation, ticks every 16 clocks
/// - Lead: 16-step Euclidean, 20-bit Turing, 25% mutation, ticks every clock
/// - Tempo: 120 BPM (hardcoded for now)
/// - Scale: Phrygian (dark, hypnotic, perfect for techno)
/// - Portamento: Bass 0.997 (~100ms), Lead 0.995 (~50ms)
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
