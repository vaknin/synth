//! Prototype synthesizer for ESP32-S3
//! MVP testbed for evaluating features before final pocket synth design
//!
//! Architecture:
//! - Dual-core: Audio streaming on Core 0 (main), ADC polling + display on Core 1 (secondary)
//! - Embassy MPSC channel for control messages (Core 1 → Core 0)
//! - Embassy Signal for waveform data (Core 0 → Core 1, latest-value semantics)
//! - Engine handles synthesis + rendering
//! - Embassy async for event-driven DMA

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{channel::Channel, watch::Watch};
use esp_backtrace as _;
use esp_hal::{dma_circular_buffers, system::Stack, timer::timg::TimerGroup};
use static_cell::StaticCell;
use synth::{config::*, engine::Engine, hardware, AudioStream, SynthChannel, WaveformSignal};

esp_bootloader_esp_idf::esp_app_desc!();

/// Global MPSC channel for control → audio communication (Core 1 → Core 0)
static CHANNEL: SynthChannel = Channel::new();

/// Global Watch for waveform data (Core 0 → Core 1, latest-value semantics, non-blocking reads)
static WAVEFORM_SIGNAL: WaveformSignal = Watch::new();

/// Stack for Core 1 (APP_CPU) - ADC polling + display tasks.
static CORE_1_STACK: StaticCell<Stack<CORE_1_STACK_SIZE>> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // Initialize logger at INFO level
    esp_println::logger::init_logger(log::LevelFilter::Info);

    log::info!("=== ESP32-S3 Synth - Step 3B: Audio + Display + Pots ===");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize embassy timer and start scheduler
    let timer0 = TimerGroup::new(peripherals.TIMG0).timer0;
    esp_rtos::start(timer0);

    // ========== PHASE 2.1: ENABLE DUAL-CORE ==========

    // Setup software interrupts for dual-core RTOS
    let (software_interrupt_0, software_interrupt_1) =
        hardware::setup_interrupts(peripherals.SW_INTERRUPT);

    // MPSC Channel (thread-safe across cores)
    let consumer = CHANNEL.receiver();
    let producer = CHANNEL.sender();

    // Waveform Signal reference (shared across cores)
    let waveform_signal: &'static WaveformSignal = &WAVEFORM_SIGNAL;

    // Extract CPU_CTRL FIRST before moving any other peripherals
    let cpu_ctrl = peripherals.CPU_CTRL;

    // Extract I2S peripherals (will be initialized AFTER Core 1 starts)
    let i2s0 = peripherals.I2S0;
    let dma_ch0 = peripherals.DMA_CH0;
    let gpio35 = peripherals.GPIO35;
    let gpio36 = peripherals.GPIO36;
    let gpio37 = peripherals.GPIO37;

    // Extract raw ADC peripherals (will be initialized on Core 1 inside pot_task)
    let adc1 = peripherals.ADC1;
    let gpio1 = peripherals.GPIO1; // Pot 0: Filter Cutoff
    let gpio2 = peripherals.GPIO2; // Pot 1: Filter Resonance

    // ========== DISPLAY SETUP (Peripherals created on Core 0, moved to Core 1) ==========

    log::info!("Setting up SPI display...");
    let (spi, cs, dc, rst) = hardware::setup_spi_display(
        peripherals.SPI2,
        peripherals.DMA_CH1,
        peripherals.GPIO9,  // DC
        peripherals.GPIO10, // CS
        peripherals.GPIO11, // MOSI
        peripherals.GPIO12, // SCK
        peripherals.GPIO13, // RST
    );
    log::info!("SPI display peripherals configured");

    // ========== PHASE 2.2: START CORE 1 WITH DISPLAY TASK ==========

    // Initialize Core 1 stack (must happen after esp_rtos::start)
    let core_1_stack = CORE_1_STACK.init(Stack::new());

    log::info!("Starting Core 1 (spawning display task)...");

    // Start Core 1 (APP_CPU) with display task
    esp_rtos::start_second_core(
        cpu_ctrl,
        software_interrupt_0,
        software_interrupt_1,
        core_1_stack,
        move || {
            // This closure runs on Core 1
            use esp_rtos::embassy::Executor;
            use static_cell::StaticCell;
            static EXECUTOR: StaticCell<Executor> = StaticCell::new();

            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                log::info!("Core 1 executor started");

                // Spawn pot_task with raw peripherals (ADC init happens inside task on Core 1)
                // log::info!("Spawning pot_task...");
                // spawner
                //     .spawn(synth::controls::task::pot_task(
                //         producer, adc1, gpio1, gpio2,
                //     ))
                //     .expect("Failed to spawn pot task on Core 1");
                // log::info!("pot_task spawned");

                // Spawn waveform display task on Core 1 (converts blocking SPI to async on Core 1)
                // log::info!("Spawning display_task...");
                // spawner
                //     .spawn(synth::display::task::display_task(
                //         spi,
                //         cs,
                //         dc,
                //         rst,
                //         waveform_signal,
                //     ))
                //     .expect("Failed to spawn display task on Core 1");
                // log::info!("display_task spawned");

                log::info!("Core 1: Both tasks spawned successfully");
            });
        },
    );

    // Core 0 (PRO_CPU) continues here - initialize audio hardware AFTER Core 1 started
    #[allow(clippy::manual_div_ceil)]
    let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, DMA_BUFFER_SIZE);

    log::info!("Setting up I2S audio (Core 0, DMA_CH0)...");
    let mut audio_stream: AudioStream = hardware::setup_audio(
        i2s0,
        dma_ch0,
        gpio35,
        gpio36,
        gpio37,
        tx_buffer,
        tx_descriptors,
    );
    log::info!("I2S audio ready");

    log::info!("Core 0: Initializing Engine...");
    let mut engine = Engine::new(consumer, waveform_signal);
    log::info!("Core 0: Engine ready, entering audio loop");

    // Core 0 main loop: audio rendering with I2S DMA
    loop {
        engine.stream_audio(&mut audio_stream).await;
    }
}
