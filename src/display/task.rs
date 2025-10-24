//! Display refresh task (Core 1).

use super::DisplayDriver;
use crate::config::DISPLAY_REFRESH_INTERVAL_MS;
use crate::WaveformConsumer;
use embassy_time::{Duration, Timer};

/// OLED refresh task: time-based scrolling with velocity correction.
#[embassy_executor::task]
pub async fn display_task(mut display: DisplayDriver, mut waveform_consumer: WaveformConsumer) {
    let frame_duration = Duration::from_millis(DISPLAY_REFRESH_INTERVAL_MS);

    loop {
        // Update cached waveform data if available
        if let Some(new_data) = waveform_consumer.dequeue() {
            display.cached_waveform = new_data;
        }

        // ALWAYS draw (time-based scrolling, independent of data arrival)
        display.draw_waveform();

        // Wait for next frame
        Timer::after(frame_duration).await;
    }
}
