//! This shows how to transmit data continuously via I2S.
//!
//! Without an additional I2S sink device you can inspect the BCLK, WS
//! and DOUT with a logic analyzer.
//!
//! You can also connect e.g. a PCM510x to hear an annoying loud sine tone (full
//! scale), so turn down the volume before running this example.
//!
//! The following wiring is assumed:
//! - BCLK => GPIO7
//! - WS   => GPIO8
//! - DOUT => GPIO9

//% CHIPS: esp32 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicI32, Ordering};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    time::Rate,
    timer::timg::TimerGroup,
};
use log::info;
use synth::{audio_util::f32_to_i16_le, oscillator::Oscillator};

esp_bootloader_esp_idf::esp_app_desc!();

const SAMPLE_RATE: u32 = 44_100;
static COUNTER: AtomicI32 = AtomicI32::new(0);

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize logger
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    let dma_channel = peripherals.DMA_CH0;

    #[allow(clippy::manual_div_ceil)]
    let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 2052); // %12 == 0, matches docs example
    let i2s_tx = I2s::new(
        peripherals.I2S0,
        Standard::Philips,
        DataFormat::Data16Channel16,
        Rate::from_hz(SAMPLE_RATE),
        dma_channel,
    )
    .into_async()
    .i2s_tx
    .with_bclk(peripherals.GPIO7)
    .with_ws(peripherals.GPIO8)
    .with_dout(peripherals.GPIO9)
    .build(tx_descriptors);

    // Create oscillator at 220Hz (A3 note)
    const FREQUENCY: f32 = 20.0;
    let mut oscillator = Oscillator::new(FREQUENCY, SAMPLE_RATE as f32);

    // Log oscillator diagnostics
    // let (phase_inc, samples_per_cycle, actual_freq) = oscillator.diagnostics();
    // info!("Oscillator config:");
    // info!("  Phase increment: {}", phase_inc);
    // info!("  Samples per cycle: {}", samples_per_cycle);
    // info!("  Actual frequency: {} Hz", actual_freq);
    // info!("DMA buffer size: {} bytes ({} stereo samples)", tx_buffer.len(), tx_buffer.len() / 4);

    // // Test: generate a few samples to verify oscillator
    // info!("First 10 samples from oscillator:");
    // for i in 0..10 {
    //     let sample = oscillator.tick();
    //     info!("  Sample {}: {}", i, sample);
    // }

    let mut transaction = i2s_tx.write_dma_circular_async(tx_buffer).unwrap();

    loop {
        oscillator.set_frequency(oscillator.frequency+0.05);
        let _result = transaction.push_with(|buffer| {
            
            // Verify buffer meets minimum stereo frame size
            if buffer.len() < 4 {
                return 0;
            }
            
            COUNTER.fetch_add(1, Ordering::Release);

            // Get iterator that separates complete chunks from remainder
            let remainder = buffer.len() % 4;
            let mut chunks = buffer.chunks_exact_mut(4);

            // Process all complete 4-byte chunks (stereo i16: 2 bytes × 2 channels)
            for chunk in &mut chunks {
                // Generate one f32 sample from oscillator (-1.0 to 1.0)
                let sample = oscillator.tick();

                // Convert to i16 little-endian bytes
                let bytes = f32_to_i16_le(sample);

                // Write stereo: left channel (2 bytes)
                chunk[0] = bytes[0];
                chunk[1] = bytes[1];

                // Write stereo: right channel (duplicate mono for now)
                chunk[2] = bytes[0];
                chunk[3] = bytes[1];
            }
            
            buffer.len() - remainder
        }).await;
    }
}

// #[embassy_executor::task]
// async fn mytask(osc: Oscillator) {
//     loop {
//         Timer::after(Duration::from_secs(1)).await;
//         osc.set_frequency(osc.frequency + 50f32);
//     }
// }