//! Button input handling with async edge detection.

use crate::controls::CtrlSender;
use crate::message::Message;
use esp_hal::gpio::Input;
use log::warn;

/// Button control task for a single voice.
///
/// Uses async GPIO edge detection (wait_for_low/high) instead of polling.
/// Each button gets its own task since GPIO events are independent.
///
/// Press behavior:
/// - Press button → Send SelectVoice(idx) message
/// - Engine handles toggle logic based on current state
///
/// # Arguments
/// * `sender` - Embassy channel sender for control messages
/// * `button` - GPIO input configured with pull-up (active-low)
/// * `voice_idx` - Voice index (0-2)
#[embassy_executor::task(pool_size = 3)]
pub async fn button_task(sender: CtrlSender, mut button: Input<'static>, voice_idx: u8) {
    loop {
        // Wait for button press (active-low, so wait for LOW)
        button.wait_for_low().await;

        // Send selection message - Engine will handle toggle logic
        if let Err(e) = sender.try_send(Message::ToggleVoice(voice_idx)) {
            warn!("Button message dropped (queue full): {:?}", e);
        }

        // Wait for button release before accepting next press
        button.wait_for_high().await;
    }
}
