//! Display refresh task (Core 1).

use super::DisplayDriver;
use crate::config::DISPLAY_REFRESH_RATE_HZ;
use crate::WaveformConsumer;
use embassy_time::{Duration, Ticker};
use esp_hal::{gpio::Output, spi::master::SpiDmaBus};

/// OLED refresh task: phase-locked scrolling with drift-free timing.
///
/// IMPORTANT: Accepts SPI in Blocking mode and creates DisplayDriver on Core 1.
/// This ensures async conversion and interrupt handler registration happen on Core 1.
#[embassy_executor::task]
pub async fn display_task(
    spi: SpiDmaBus<'static, esp_hal::Blocking>,
    cs: Output<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
    mut waveform_consumer: WaveformConsumer,
) {
    log::info!("Display task starting...");

    // Create display driver on Core 1 (converts SPI to Async internally)
    let mut display = DisplayDriver::new(spi, cs, dc, rst).await;
    log::info!("Display initialized successfully");

    let mut ticker = Ticker::every(Duration::from_hz(DISPLAY_REFRESH_RATE_HZ as u64));
    let mut frame_count = 0u32;

    loop {
        // Update cached waveform data if available
        if let Some(new_data) = waveform_consumer.dequeue() {
            display.cached_waveform = new_data;
            if frame_count % 30 == 0 {
                log::info!("Waveform data received, write_index={}", new_data.write_index);
            }
        }

        // ALWAYS draw (async - yields during 13.1ms DMA flush)
        display.draw_waveform().await;

        frame_count += 1;
        if frame_count % 30 == 0 {
            log::info!("Display frame {}, write_index={}", frame_count, display.cached_waveform.write_index);
        }

        // Wait for next tick (phase-locked, compensates for processing time)
        ticker.next().await;
    }
}
