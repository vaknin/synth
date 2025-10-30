//! Display refresh task.

use super::DisplayDriver;
use crate::config::DISPLAY_REFRESH_RATE_HZ;
use crate::WaveformSignal;
use embassy_time::{Duration, Ticker};
use esp_hal::{gpio::Output, spi::master::SpiDmaBus};

/// OLED refresh task with test animation.
///
/// Step 1: Runs on Core 0 as Embassy task (validates task architecture)
/// Step 2+: Will move to Core 1 for dual-core operation
#[embassy_executor::task]
pub async fn display_test_task(
    spi: SpiDmaBus<'static, esp_hal::Blocking>,
    cs: Output<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) {
    // Create display driver (converts blocking SPI to async)
    let mut display = DisplayDriver::new(spi, cs, dc, rst).await;

    let mut ticker = Ticker::every(Duration::from_hz(DISPLAY_REFRESH_RATE_HZ as u64));

    loop {
        ticker.next().await;
        display.draw_test_animation().await;
    }
}

/// OLED refresh task with waveform rendering (for dual-core operation).
///
/// Reads waveform data from Signal and renders oscilloscope view.
/// Currently unused - will be activated in Step 2+ when moving to Core 1.
#[embassy_executor::task]
pub async fn display_task(
    spi: SpiDmaBus<'static, esp_hal::Blocking>,
    cs: Output<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
    waveform_signal: &'static WaveformSignal,
) {
    log::info!("Display task starting on Core 1...");

    let mut display = DisplayDriver::new(spi, cs, dc, rst).await;
    log::info!("Display initialized successfully");

    let mut ticker = Ticker::every(Duration::from_hz(DISPLAY_REFRESH_RATE_HZ as u64));

    log::info!(
        "Display task entering waveform loop at {}Hz",
        DISPLAY_REFRESH_RATE_HZ
    );

    loop {
        ticker.next().await;

        // Read latest waveform snapshot from Signal
        let waveform = waveform_signal.wait().await;
        display.cached_waveform.copy_from_slice(&waveform.samples);

        // Render oscilloscope
        display.draw_waveform().await;
    }
}
