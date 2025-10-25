//! Display refresh task (Core 1).

use super::DisplayDriver;
use crate::config::DISPLAY_REFRESH_RATE_HZ;
use crate::WaveformConsumer;
use embassy_time::{Duration, Ticker};

/// OLED refresh task: phase-locked scrolling with drift-free timing.
#[embassy_executor::task]
pub async fn display_task(mut display: DisplayDriver, mut waveform_consumer: WaveformConsumer) {
    let mut ticker = Ticker::every(Duration::from_hz(DISPLAY_REFRESH_RATE_HZ as u64));

    loop {
        // Update cached waveform data if available
        if let Some(new_data) = waveform_consumer.dequeue() {
            display.cached_waveform = new_data;
        }

        // ALWAYS draw (time-based scrolling, independent of data arrival)
        display.draw_waveform();

        // Wait for next tick (phase-locked, compensates for processing time)
        ticker.next().await;
    }
}
