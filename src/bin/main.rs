//! polyphonic synthesizer for ESP32-S3
//!
//! Architecture:
//! - Lock-free SPSC queue for control messages
//! - Engine handles synthesis + rendering
//! - Embassy async for event-driven DMA

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as ChannelMutex;


// use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    dma_circular_buffers,
    gpio::{Input, Output, Pull},
    timer::timg::TimerGroup,
};
// use heapless::spsc::Queue;
use synth::{config::*, engine::Engine, hardware, message::Message};

esp_bootloader_esp_idf::esp_app_desc!();

pub type CtrlSender   = Sender<'static, ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;
pub type CtrlReceiver = Receiver<'static, ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;

static CHANNEL: Channel<ChannelMutex, Message, MESSAGE_QUEUE_SIZE> = Channel::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize logger
    esp_println::logger::init_logger_from_env();
    
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    let dma_channel = peripherals.DMA_CH0;
    
    // Create MPSC Channel
    let consumer = CHANNEL.receiver();
    let freq_producer = CHANNEL.sender();
    let vol_producer = CHANNEL.sender();

    // Create synth engine
    let mut engine = Engine::new(SAMPLE_RATE as f32);

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
    let adc_ctrl = hardware::setup_adc(peripherals.ADC1, peripherals.GPIO1, peripherals.GPIO2);

    // Spawn control task to read both potentiometers
    spawner.spawn(synth::controls::control_task(producer, adc_ctrl)).unwrap();

    // Audio rendering loop
    loop {
        audio_stream
            .push_with(|buffer| engine.render(&mut consumer, buffer))
            .await
            .ok();
    }
}

