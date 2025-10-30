//! Display refresh task (Core 1).

use super::DisplayDriver;
use crate::config::DISPLAY_REFRESH_RATE_HZ;
use crate::WaveformSignal;
use embassy_time::{Duration, Ticker};
use esp_hal::{gpio::Output, spi::master::SpiDmaBus};

/// OLED refresh task: phase-locked scrolling with drift-free timing.
///
/// **CRITICAL**: Accepts blocking SPI created on Core 0, then converts to async on Core 1.
/// This ensures DMA interrupts are properly bound to Core 1 where the async executor runs.
#[embassy_executor::task]
pub async fn display_task(
    spi: SpiDmaBus<'static, esp_hal::Blocking>,
    cs: Output<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
    _waveform_signal: &'static WaveformSignal,
) {
    log::info!("Display task starting on Core 1...");

    // Create display driver on Core 1 (converts blocking SPI to async here)
    let mut display = DisplayDriver::new(spi, cs, dc, rst).await;
    log::info!("Display initialized successfully");

    let mut ticker = Ticker::every(Duration::from_hz(DISPLAY_REFRESH_RATE_HZ as u64));
    let mut frame_count = 0u32;

    log::info!(
        "Display task entering main loop at {}Hz",
        DISPLAY_REFRESH_RATE_HZ
    );

    loop {
        // Wait for next frame time (phase-locked to DISPLAY_REFRESH_RATE_HZ)
        ticker.next().await;

        // Simple test: draw animated rectangle (no audio waveform)
        display.draw_test_animation(frame_count).await;

        frame_count += 1;

        // Log every 30 frames (1 second at 30 Hz)
        if frame_count % 30 == 0 {
            log::info!("Display test: frame={}", frame_count);
        }
    }
}
