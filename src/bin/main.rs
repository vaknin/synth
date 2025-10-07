//! polyphonic synthesizer for ESP32-S3
//!
//! Architecture:
//! - Embassy MPSC channel for control messages (multiple producers → engine)
//! - Engine handles synthesis + rendering
//! - Embassy async for event-driven DMA

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use esp_backtrace as _;
use esp_hal::{dma_circular_buffers, timer::timg::TimerGroup};
use synth::{config::*, engine::Engine, hardware, AudioStream, SynthChannel};

esp_bootloader_esp_idf::esp_app_desc!();

/// Global MPSC channel for control → audio communication

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize logger
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    let dma_channel = peripherals.DMA_CH0;

    // MPSC Channel initialization
    static CHANNEL: SynthChannel = Channel::new();
    let consumer = CHANNEL.receiver();
    let producer = CHANNEL.sender();

    // Initialize synth with test messages (Phase 1: Oscillator control)
    producer.send(Message::SetMasterPitch(77.0)).await;
    producer
        .send(Message::SetOscLevel(
            synth::message::OscillatorId::Osc1,
            0.9,
        ))
        .await;

    // Initialize I2S audio hardware
    #[allow(clippy::manual_div_ceil)]
    let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, DMA_BUFFER_SIZE);

    // Initialize ADC for potentiometers (freq on GPIO1, vol on GPIO2)
    let (adc_bus, freq_pin, vol_pin) =
        hardware::setup_adc(peripherals.ADC1, peripherals.GPIO1, peripherals.GPIO2);

    // Spawn pot task to read both potentiometers
    spawner
        .spawn(synth::controls::pot_task(
            producer, adc_bus, freq_pin, vol_pin,
        ))
        .unwrap();

    let mut audio_stream: AudioStream = hardware::setup_audio(
        peripherals.I2S0,
        dma_channel,
        peripherals.GPIO7,
        peripherals.GPIO8,
        peripherals.GPIO9,
        tx_buffer,
        tx_descriptors,
    );

    // Create the synth engine
    let mut engine = Engine::new(consumer);

    // Main loop
    loop {
        engine.stream_audio(&mut audio_stream).await;
    }
}
