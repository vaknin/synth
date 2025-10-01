//! polyphonic synthesizer for ESP32-S3
//!
//! Architecture:
//! - Lock-free SPSC queue for control messages
//! - Engine handles synthesis + rendering
//! - Embassy async for event-driven DMA

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{dma_circular_buffers, timer::timg::TimerGroup};
use heapless::spsc::Queue;
use synth::{config::*, engine::Engine, hardware, message::Message};

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
    producer.enqueue(Message::SelectVoice(0)).ok();
    producer.enqueue(Message::ToggleVoice(0)).ok();
    producer.enqueue(Message::SetVolume(1.0)).ok();

    producer.enqueue(Message::SelectVoice(1)).ok();
    producer.enqueue(Message::ToggleVoice(1)).ok();
    producer.enqueue(Message::SetFrequency(180.0)).ok();
    producer.enqueue(Message::SetVolume(0.35)).ok();

    producer.enqueue(Message::SelectVoice(2)).ok();
    producer.enqueue(Message::ToggleVoice(2)).ok();
    producer.enqueue(Message::SetFrequency(440.0)).ok();
    producer.enqueue(Message::SetVolume(0.15)).ok();

    // Initialize I2S audio hardware
    #[allow(clippy::manual_div_ceil)]
    let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, DMA_BUFFER_SIZE);
    let mut audio_stream = hardware::setup_audio(
        peripherals.I2S0,
        dma_channel,
        peripherals.GPIO7,
        peripherals.GPIO8,
        peripherals.GPIO9,
        tx_buffer,
        tx_descriptors,
    );

    // TODO: Add ADC setup for pots (frequency + volume control)
    // TODO: Add GPIO button handling for voice selection/toggle

    // Audio rendering loop
    loop {
        audio_stream
            .push_with(|buffer| engine.render(&mut consumer, buffer))
            .await
            .ok();
    }
}