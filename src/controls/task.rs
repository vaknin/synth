//! Control tasks: potentiometers and future encoders.

use crate::config::*;
use crate::controls::{
    map_attack, map_bass_euclidean_hits, map_cutoff, map_decay, map_filter_envelope_depth,
    map_lead_euclidean_hits, map_resonance, map_tempo, CtrlSender, Potentiometer,
};
use crate::hardware::{AdcBus, AnyAdcPin};
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: sequentially reads all pots.
///
/// Owns the ADC peripheral and all pot pins. Each pot has independent
/// signal processing state (EMA filter, deadband) but shares the hardware.
///
/// Current 8-pot setup (curated generative instrument):
/// - Pot 1 (GPIO1): Filter Cutoff (main performance control)
/// - Pot 2 (GPIO2): Filter Resonance (Moog character)
/// - Pot 3 (GPIO3): Filter Envelope Depth (rhythm-synced pluck intensity)
/// - Pot 4 (GPIO4): Envelope Attack (1-200ms, percussive to pad)
/// - Pot 5 (GPIO5): Envelope Decay (10-2000ms, tight stabs to sustain)
/// - Pot 6 (GPIO6): Bass Hit Density (0-8, length fixed at 8)
/// - Pot 7 (GPIO7): Lead Hit Density (0-16, length fixed at 16)
/// - Pot 8 (GPIO8): Tempo (global clock, 20-220 BPM)
///
/// Fixed generative parameters (optimal values, no pots):
/// - Bass: 8-step pattern, 10-bit Turing, 4% mutation
/// - Lead: 16-step pattern, 20-bit Turing, 28% mutation
/// - Scale: Phrygian (dark, hypnotic, perfect for techno)
/// - Portamento: 0.996 coefficient (~60ms glide, melodic but electronic)
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
        Potentiometer::new(map_attack),                // Pot 4 (GPIO4)
        Potentiometer::new(map_decay),                 // Pot 5 (GPIO5)
        Potentiometer::new(map_bass_euclidean_hits),   // Pot 6 (GPIO6)
        Potentiometer::new(map_lead_euclidean_hits),   // Pot 7 (GPIO7)
        Potentiometer::new(map_tempo),                 // Pot 8 (GPIO8)
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
