//! Prototype synthesizer for ESP32-S3
//! MVP testbed for evaluating features before final pocket synth design
//!
//! Architecture:
//! - Dual-core: Audio streaming on Core 0 (main), ADC polling on Core 1 (secondary)
//! - Embassy MPSC channel for control messages (Core 1 → Core 0)
//! - Engine handles synthesis + rendering
//! - Embassy async for event-driven DMA

#![no_std]
#![no_main]

use core::ptr::addr_of_mut;
use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use esp_backtrace as _;
use esp_hal::{
    dma_circular_buffers,
    system::{CpuControl, Stack},
    timer::{timg::TimerGroup, AnyTimer},
};
use synth::{config::*, engine::Engine, hardware, AudioStream, SynthChannel};

esp_bootloader_esp_idf::esp_app_desc!();

/// Global MPSC channel for control → audio communication (Core 1 → Core 0)
static CHANNEL: SynthChannel = Channel::new();

/// Stack for Core 1 (APP_CPU) - ADC polling task.
/// 16-byte aligned per XTensa LX7 requirements.
static mut APP_CORE_STACK: Stack<APP_CORE_STACK_SIZE> = Stack::new();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // Initialize logger
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize embassy with TWO timers (one per core) converted to AnyTimer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let timer0: AnyTimer = timg0.timer0.into();
    let timer1: AnyTimer = timg0.timer1.into();
    esp_hal_embassy::init([timer0, timer1]);

    let dma_channel = peripherals.DMA_CH0;

    // MPSC Channel (thread-safe across cores)
    let consumer = CHANNEL.receiver();
    let producer = CHANNEL.sender();

    // Initialize I2S audio hardware
    #[allow(clippy::manual_div_ceil)]
    let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, DMA_BUFFER_SIZE);

    // Initialize ADC hardware (will be moved to Core 1)
    let (adc_bus, pins) = hardware::setup_adc(
        peripherals.ADC1,
        peripherals.GPIO1,
        peripherals.GPIO2,
        peripherals.GPIO3,
        peripherals.GPIO4,
        peripherals.GPIO5,
        peripherals.GPIO6,
        peripherals.GPIO7,
        peripherals.GPIO8,
    );

    // Start Core 1 (APP_CPU) for ADC polling - offload blocking ADC reads to secondary core
    let mut cpu_control = CpuControl::new(peripherals.CPU_CTRL);
    let _guard = cpu_control
        .start_app_core(unsafe { &mut *addr_of_mut!(APP_CORE_STACK) }, move || {
            // This closure runs on Core 1 - dedicated to ADC polling
            // Blocking ADC reads happen here, isolated from audio rendering on Core 0
            use static_cell::StaticCell;
            static EXECUTOR: StaticCell<esp_hal_embassy::Executor> = StaticCell::new();

            let executor = EXECUTOR.init(esp_hal_embassy::Executor::new());
            executor.run(|spawner| {
                // Spawn pot task on Core 1
                spawner
                    .spawn(synth::controls::pot_task(producer, adc_bus, pins))
                    .ok();
            });
        })
        .unwrap();

    // Core 0 (PRO_CPU) continues here - initialize audio hardware
    // I2S on GPIO35/36/37 (WS/DOUT/BCLK)
    let mut audio_stream: AudioStream = hardware::setup_audio(
        peripherals.I2S0,
        dma_channel,
        peripherals.GPIO35,
        peripherals.GPIO36,
        peripherals.GPIO37,
        tx_buffer,
        tx_descriptors,
    );

    // Create synth engine on Core 0
    let mut engine = Engine::new(consumer);

    // Main audio loop runs on Core 0 - zero interference from ADC blocking on Core 1
    loop {
        engine.stream_audio(&mut audio_stream).await;
    }
}
