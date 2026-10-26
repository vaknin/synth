//! Control tasks: potentiometers polling for drone synth.
//!
//! # Embassy Task Parameter Guidelines
//!
//! When spawning Embassy tasks, keep parameters small (<1KB) to avoid executor stack overflow.
//! Large peripheral structures (especially ADC with calibration ~13KB) must be initialized
//! INSIDE the task, not passed as parameters.
//!
//! Pattern:
//! ```rust
//! // ✅ DO: Pass raw peripherals (~8 bytes each)
//! #[embassy_executor::task]
//! pub async fn my_task(raw: RAW_PERIPHERAL) {
//!     let initialized = hardware::setup(raw);  // Init on task stack
//!     // ...
//! }
//!
//! // ❌ DON'T: Pass initialized peripherals with calibration data
//! #[embassy_executor::task]
//! pub async fn my_task(initialized: LARGE_TYPE) {  // May overflow spawn frame!
//!     // ...
//! }
//! ```

use crate::config::*;
use crate::controls::{map_cutoff, map_resonance, CtrlSender, Potentiometer};
use crate::hardware;
use embassy_time::{Duration, Timer};

/// Potentiometer polling task: initializes ADC internally on Core 1.
///
/// Takes raw GPIO peripherals and configures ADC inside the task,
/// avoiding large parameter capture (ADC calibration data ~13KB).
///
/// Simplified 2-pot setup for drone synth:
/// - Pot 0 (GPIO1): Filter Cutoff
/// - Pot 1 (GPIO2): Filter Resonance
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `adc1` - Raw ADC1 peripheral (not initialized)
/// * `gpio1` - Raw GPIO1 peripheral for Pot 0
/// * `gpio2` - Raw GPIO2 peripheral for Pot 1
#[embassy_executor::task]
pub async fn pot_task(
    sender: CtrlSender,
    adc1: esp_hal::peripherals::ADC1<'static>,
    gpio1: esp_hal::peripherals::GPIO1<'static>,
    gpio2: esp_hal::peripherals::GPIO2<'static>,
) {
    // Initialize ADC inside task (runs on Core 1 task stack, not executor stack)
    let (mut adc_bus, mut pins) = hardware::setup_adc(adc1, gpio1, gpio2);

    let mut pots = [
        Potentiometer::new(map_cutoff),
        Potentiometer::new(map_resonance),
    ];

    loop {
        // Poll both pots with explicit yields
        // Yielding after ADC reads prevents starving display_task on Core 1 executor
        for (pot, pin) in pots.iter_mut().zip(pins.iter_mut()) {
            pot.poll_and_send_yielding(sender, pin, &mut adc_bus.adc)
                .await;
        }

        // Timer::after is also a yield point - display_task can run here
        Timer::after(Duration::from_millis(ADC_POLL_INTERVAL_MS)).await;
    }
}
