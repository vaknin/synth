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
    let mut dequeue_count = 0u32;

    log::info!("Display task entering main loop at {}Hz", DISPLAY_REFRESH_RATE_HZ);

    loop {
        log::info!("Frame {} start", frame_count);

        // Update cached waveform data if available
        if let Some(new_data) = waveform_consumer.dequeue() {
            dequeue_count += 1;
            display.cached_waveform = new_data;
            log::info!("  Dequeued waveform #{}, write_index={}", dequeue_count, new_data.write_index);
        } else {
            log::warn!("  No waveform data available in queue");
        }

        // ALWAYS draw (async - yields during 13.1ms DMA flush)
        log::info!("  Drawing waveform...");
        display.draw_waveform().await;
        log::info!("  Draw complete");

        frame_count += 1;
        if frame_count % 30 == 0 {
            log::info!("=== Display stats: frame={}, dequeued={}, write_index={} ===",
                frame_count, dequeue_count, display.cached_waveform.write_index);
        }

        // Wait for next tick (phase-locked, compensates for processing time)
        log::info!("  Waiting for next tick...");
        ticker.next().await;
    }
}
