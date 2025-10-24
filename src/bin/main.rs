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

use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use esp_backtrace as _;
use esp_hal::{dma_circular_buffers, system::Stack, timer::timg::TimerGroup};
use static_cell::StaticCell;
use synth::{config::*, engine::Engine, hardware, AudioStream, SynthChannel, WaveformQueue};

esp_bootloader_esp_idf::esp_app_desc!();

/// Global MPSC channel for control → audio communication (Core 1 → Core 0)
static CHANNEL: SynthChannel = Channel::new();

/// Global SPSC queue for waveform data (Core 0 → Core 1)
static WAVEFORM_QUEUE: StaticCell<WaveformQueue> = StaticCell::new();

/// Stack for Core 1 (APP_CPU) - ADC polling task.
static APP_CORE_STACK: StaticCell<Stack<APP_CORE_STACK_SIZE>> = StaticCell::new();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    // Initialize logger
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize embassy timer and start scheduler
    let timer0 = TimerGroup::new(peripherals.TIMG0).timer0;
    esp_rtos::start(timer0);

    // Setup software interrupts for dual-core RTOS
    let (software_interrupt_0, software_interrupt_1) =
        hardware::setup_interrupts(peripherals.SW_INTERRUPT);

    let dma_channel = peripherals.DMA_CH0;

    // MPSC Channel (thread-safe across cores)
    let consumer = CHANNEL.receiver();
    let producer = CHANNEL.sender();

    // SPSC Queue for waveform data (Core 0 → Core 1)
    let waveform_queue = WAVEFORM_QUEUE.init(WaveformQueue::new());
    let (waveform_producer, waveform_consumer) = waveform_queue.split();

    // Initialize I2S audio hardware
    #[allow(clippy::manual_div_ceil)]
    let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, DMA_BUFFER_SIZE);

    // Initialize ADC hardware for 2 pots (will be moved to Core 1)
    let (adc_bus, pins) = hardware::setup_adc(
        peripherals.ADC1,
        peripherals.GPIO1, // Pot 0: Filter Cutoff
        peripherals.GPIO2, // Pot 1: Filter Resonance
    );

    // Initialize SPI display hardware (will be moved to Core 1)
    let (spi, cs, dc, rst) = hardware::setup_spi_display(
        peripherals.SPI2,
        peripherals.GPIO9,  // DC
        peripherals.GPIO10, // CS
        peripherals.GPIO11, // MOSI
        peripherals.GPIO12, // SCK
        peripherals.GPIO13, // RST
    );

    // Create display driver
    let display = synth::display::DisplayDriver::new(spi, cs, dc, rst);

    // Initialize Core 1 stack (must happen after esp_rtos::start)
    let app_core_stack = APP_CORE_STACK.init(Stack::new());

    // Start Core 1 (APP_CPU) for ADC polling using esp-rtos scheduler
    esp_rtos::start_second_core(
        peripherals.CPU_CTRL,
        software_interrupt_0,
        software_interrupt_1,
        app_core_stack,
        move || {
            // This closure runs on Core 1 - dedicated to ADC polling
            // Blocking ADC reads happen here, isolated from audio rendering on Core 0
            use esp_rtos::embassy::Executor;
            use static_cell::StaticCell;
            static EXECUTOR: StaticCell<Executor> = StaticCell::new();

            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                // Spawn pot task on Core 1
                spawner
                    .spawn(synth::controls::pot_task(producer, adc_bus, pins))
                    .ok();

                // Spawn display task on Core 1 (runs alongside pot polling)
                spawner
                    .spawn(synth::display::task::display_task(
                        display,
                        waveform_consumer,
                    ))
                    .ok();
            });
        },
    );

    // Core 0 (PRO_CPU) continues here - initialize audio hardware
    // I2S on GPIO35/36/37 to avoid conflicts with ADC, I2C, SPI
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
    let mut engine = Engine::new(consumer, waveform_producer);

    // Main audio loop runs on Core 0 - zero interference from ADC blocking on Core 1
    loop {
        engine.stream_audio(&mut audio_stream).await;
    }
}
