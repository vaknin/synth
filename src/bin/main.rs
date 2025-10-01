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

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    time::Rate,
    timer::timg::TimerGroup,
};
use heapless::spsc::Queue;
use synth::{
    audio_util::f32_to_i16_le,
    config::*,
    engine::Engine,
    message::Message,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // Initialize logger
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    let dma_channel = peripherals.DMA_CH0;

    // Create SPSC queue for control → audio messages
    static MESSAGE_QUEUE: static_cell::StaticCell<Queue<Message, MESSAGE_QUEUE_SIZE>> = static_cell::StaticCell::new();
    let queue = MESSAGE_QUEUE.init(Queue::new());
    let (mut producer, mut consumer) = queue.split();

    // Create synth engine
    let mut engine = Engine::new(SAMPLE_RATE as f32);
    producer.enqueue(Message::ToggleVoice(0)).ok();
    producer.enqueue(Message::SelectVoice(0)).ok();
    producer.enqueue(Message::SetVolume(1.)).ok();

    #[allow(clippy::manual_div_ceil)]
    let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, DMA_BUFFER_SIZE); // %12 == 0, matches docs example 2040,4096,1024
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

    // TODO: Add ADC setup for pots (frequency + volume control)
    // TODO: Add GPIO button handling for voice selection/toggle

    let mut transaction = i2s_tx.write_dma_circular_async(tx_buffer).unwrap();

    loop {
        transaction.push_with(|buffer| {
            // Verify buffer meets minimum stereo frame size
            if buffer.len() < 4 {
                return 0;
            }

            // --- MESSAGE PROCESSING ---
            // Drain all pending messages from queue and process sequentially
            while let Some(msg) = consumer.dequeue() {
                engine.process_message(msg);
            }

            // --- AUDIO GENERATION ---
            let remainder = buffer.len() % 4;
            let mut chunks = buffer.chunks_exact_mut(4);

            // Process all complete 4-byte chunks (stereo i16: 2 bytes × 2 channels)
            for chunk in &mut chunks {
                // Generate mixed sample from all voices
                let sample = engine.tick();

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
        }).await.ok();
    }
}