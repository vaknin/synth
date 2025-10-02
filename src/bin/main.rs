//! polyphonic synthesizer for ESP32-S3
//!
//! Architecture:
//! - Embassy MPSC channel for control messages (multiple producers → engine)
//! - Engine handles synthesis + rendering
//! - Embassy async for event-driven DMA

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as ChannelMutex;
use embassy_sync::channel::Channel;
use esp_backtrace as _;
use esp_hal::{dma_circular_buffers, gpio::{Input, InputConfig, Pull}, timer::timg::TimerGroup};
use synth::{config::*, controls::button_task, engine::Engine, hardware, message::Message};

esp_bootloader_esp_idf::esp_app_desc!();

/// Global MPSC channel for control → audio communication
static CHANNEL: Channel<ChannelMutex, Message, MESSAGE_QUEUE_SIZE> = Channel::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize logger
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    let dma_channel = peripherals.DMA_CH0;

    // Get channel endpoints
    let receiver = CHANNEL.receiver();
    let sender = CHANNEL.sender();

      // Setup 3 buttons (GPIO3, GPIO4, GPIO5)
    let config = InputConfig::default().with_pull(Pull::Up);
    let btn0 = Input::new(peripherals.GPIO3, config);
    // let btn1 = Input::new(peripherals.GPIO4, Pull::Up);
    // let btn2 = Input::new(peripherals.GPIO5, Pull::Up);

    // Spawn same task 3 times with different parameters!
    spawner.spawn(button_task(sender, btn0, 0)).unwrap();
    // spawner.spawn(button_task(sender.clone(), btn1, 1)).unwrap();
    // spawner.spawn(button_task(sender.clone(), btn2, 2)).unwrap();


    // Create synth engine with receiver
    sender.send(Message::ToggleVoice(0)).await;
    sender.send(Message::SelectVoice(0)).await;
    sender.send(Message::SetVolume(1.0)).await;

    let mut engine = Engine::new(SAMPLE_RATE as f32, receiver);

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

    // Initialize ADC for potentiometers (freq on GPIO1, vol on GPIO2)
    let (adc_bus, freq_pin, vol_pin) = hardware::setup_adc(
        peripherals.ADC1,
        peripherals.GPIO1,
        peripherals.GPIO2,
    );

    // Spawn pot task to read both potentiometers
    spawner
        .spawn(synth::controls::pot_task(sender, adc_bus, freq_pin, vol_pin))
        .unwrap();

    // Audio rendering loop
    loop {
        audio_stream
            .push_with(|buffer| engine.render(buffer))
            .await
            .ok();
    }
}

