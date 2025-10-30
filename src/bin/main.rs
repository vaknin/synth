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
use embassy_sync::{channel::Channel, signal::Signal};
use esp_backtrace as _;
use esp_hal::{dma_circular_buffers, system::Stack, timer::timg::TimerGroup};
use static_cell::StaticCell;
use synth::{config::*, engine::Engine, hardware, AudioStream, SynthChannel, WaveformSignal};

esp_bootloader_esp_idf::esp_app_desc!();

/// Global MPSC channel for control → audio communication (Core 1 → Core 0)
static CHANNEL: SynthChannel = Channel::new();

/// Global Signal for waveform data (Core 0 → Core 1, latest-value semantics)
static WAVEFORM_SIGNAL: WaveformSignal = Signal::new();

/// Stack for Core 1 (APP_CPU) - ADC polling + display tasks.
static APP_CORE_STACK: StaticCell<Stack<APP_CORE_STACK_SIZE>> = StaticCell::new();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    // Initialize logger at INFO level
    esp_println::logger::init_logger(log::LevelFilter::Info);

    log::info!("=== ESP32-S3 SPI Display Test (Single-Core) ===");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize embassy timer and start scheduler
    let timer0 = TimerGroup::new(peripherals.TIMG0).timer0;
    esp_rtos::start(timer0);

    // ========== EVERYTHING BELOW COMMENTED OUT FOR SIMPLE SPI TEST ==========

    // // Setup software interrupts for dual-core RTOS
    // let (software_interrupt_0, software_interrupt_1) =
    //     hardware::setup_interrupts(peripherals.SW_INTERRUPT);

    // // MPSC Channel (thread-safe across cores)
    // let consumer = CHANNEL.receiver();
    // let producer = CHANNEL.sender();

    // // Waveform Signal reference (shared across cores)
    // let waveform_signal: &'static WaveformSignal = &WAVEFORM_SIGNAL;

    // // Initialize I2S audio hardware (Core 0: DMA_CH0)
    // #[allow(clippy::manual_div_ceil)]
    // let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, DMA_BUFFER_SIZE);

    // log::info!("Setting up I2S audio (Core 0, DMA_CH0)...");
    // let mut audio_stream: AudioStream = hardware::setup_audio(
    //     peripherals.I2S0,
    //     peripherals.DMA_CH0,
    //     peripherals.GPIO35, // WS
    //     peripherals.GPIO36, // DOUT
    //     peripherals.GPIO37, // BCLK
    //     tx_buffer,
    //     tx_descriptors,
    // );
    // log::info!("I2S audio ready");

    // // Extract CPU_CTRL FIRST before moving any other peripherals
    // let cpu_ctrl = peripherals.CPU_CTRL;

    // // Initialize ADC hardware for 2 pots (will be moved to Core 1)
    // let (adc_bus, pins) = hardware::setup_adc(
    //     peripherals.ADC1,
    //     peripherals.GPIO1, // Pot 0: Filter Cutoff
    //     peripherals.GPIO2, // Pot 1: Filter Resonance
    // );

    // ========== SIMPLE SPI TEST ON CORE 0 ==========

    log::info!("Setting up SPI display (Core 0, blocking mode test)...");
    let (mut spi, cs, dc, mut rst) = hardware::setup_spi_display(
        peripherals.SPI2,
        peripherals.DMA_CH1,
        peripherals.GPIO9,  // DC
        peripherals.GPIO10, // CS
        peripherals.GPIO11, // MOSI
        peripherals.GPIO12, // SCK
        peripherals.GPIO13, // RST
    );
    log::info!("SPI display peripherals configured");

    // Test SPI in ASYNC mode on Core 0
    log::info!("Testing SPI with async display init...");

    use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
    use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
    use embassy_time::Delay;
    use ssd1351::{
        display_async::DisplayAsync,
        mode::GraphicsModeAsync,
        prelude::SPIInterface,
        properties::{DisplayRotation, DisplaySize},
    };
    use static_cell::StaticCell;

    // Allocate framebuffer
    const FRAMEBUFFER_SIZE: usize = (128 * 128 * 2) as usize;
    static FRAMEBUFFER: StaticCell<[u8; FRAMEBUFFER_SIZE]> = StaticCell::new();
    let framebuffer = FRAMEBUFFER.init([0u8; FRAMEBUFFER_SIZE]);

    log::info!("Converting SPI to async mode...");
    let spi_async = spi.into_async();

    log::info!("Wrapping SPI bus in Mutex for embassy SpiDevice...");
    type SpiBusType = Mutex<NoopRawMutex, esp_hal::spi::master::SpiDmaBus<'static, esp_hal::Async>>;
    static SPI_BUS: StaticCell<SpiBusType> = StaticCell::new();
    let spi_bus = SPI_BUS.init(Mutex::new(spi_async));

    log::info!("Creating embassy SpiDevice with CS pin...");
    let spi_device = SpiDevice::new(spi_bus, cs);
    let interface = SPIInterface::new(spi_device, dc);

    log::info!("Creating DisplayAsync...");
    let display_async = DisplayAsync::new(
        interface,
        DisplaySize::Display128x128,
        DisplayRotation::Rotate0,
    );

    log::info!("Creating GraphicsModeAsync with buffer...");
    let mut graphics = GraphicsModeAsync::new(display_async, framebuffer);

    log::info!("Resetting display...");
    if let Err(e) = graphics.reset(&mut rst, &mut Delay) {
        log::error!("Display reset failed: {:?}", e);
        panic!("Display reset failed");
    }

    log::info!("Initializing display (async)...");
    if let Err(e) = graphics.init().await {
        log::error!("Display init failed: {:?}", e);
        panic!("Display init failed");
    }

    log::info!("Starting animated test...");
    use embassy_time::{Duration, Instant, Ticker};
    use embedded_graphics::{
        mono_font::{ascii::FONT_6X10, MonoTextStyle},
        pixelcolor::Rgb565,
        prelude::*,
        primitives::{PrimitiveStyleBuilder, Rectangle},
        text::Text,
    };

    let mut frame = 0u32;
    let mut last_flush_ms = 0u64;
    let mut ticker = Ticker::every(Duration::from_hz(DISPLAY_REFRESH_RATE_HZ.into())); // 30 FPS

    log::info!("Entering animation loop (30 FPS)...");

    loop {
        ticker.next().await;

        // Clear screen
        graphics.clear();

        // Measure flush time
        let flush_start = Instant::now();

        // Draw moving rectangle (bounces left-right)
        let x_pos = ((frame * 2) % 128) as i32;
        Rectangle::new(Point::new(x_pos, 50), Size::new(20, 20))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::GREEN)
                    .build(),
            )
            .draw(&mut graphics)
            .ok();

        // Draw flush time text at top
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        let mut text_buffer = heapless::String::<32>::new();
        use core::fmt::Write;
        write!(&mut text_buffer, "{}ms", last_flush_ms).ok();
        Text::new(&text_buffer, Point::new(2, 10), text_style)
            .draw(&mut graphics)
            .ok();

        // Flush and measure
        if let Err(e) = graphics.flush().await {
            log::error!("Frame {}: Flush failed: {:?}", frame, e);
            panic!("Flush failed");
        }

        let flush_duration = Instant::now() - flush_start;
        last_flush_ms = flush_duration.as_millis();

        frame += 1;
    }

    // ========== DUAL-CORE CODE COMMENTED OUT ==========

    // // Initialize Core 1 stack (must happen after esp_rtos::start)
    // let app_core_stack = APP_CORE_STACK.init(Stack::new());

    // log::info!("Starting Core 1 (ADC + Display tasks)...");

    // // Start Core 1 (APP_CPU) for ADC polling + display rendering
    // esp_rtos::start_second_core(
    //     cpu_ctrl,
    //     software_interrupt_0,
    //     software_interrupt_1,
    //     app_core_stack,
    //     move || {
    //         // This closure runs on Core 1 - ADC polling + display rendering
    //         // Isolated from audio rendering on Core 0
    //         use esp_rtos::embassy::Executor;
    //         use static_cell::StaticCell;
    //         static EXECUTOR: StaticCell<Executor> = StaticCell::new();

    //         let executor = EXECUTOR.init(Executor::new());
    //         executor.run(|spawner| {
    //             log::info!("Core 1 executor started");

    //             // Spawn pot task on Core 1
    //             spawner
    //                 .spawn(synth::controls::task::pot_task(producer, adc_bus, pins))
    //                 .ok();

    //             // Spawn display task on Core 1 (converts blocking SPI to async on Core 1)
    //             spawner
    //                 .spawn(synth::display::task::display_task(
    //                     spi, cs, dc, rst, waveform_signal,
    //                 ))
    //                 .ok();

    //             log::info!("Core 1 tasks spawned (pot + display)");
    //         });
    //     },
    // );

    // log::info!("Core 0 continuing with audio engine...");

    // // Create synth engine on Core 0
    // let mut engine = Engine::new(consumer, waveform_signal);

    // log::info!("Engine initialized, starting audio loop...");

    // // Main audio loop runs on Core 0 - zero interference from ADC/display on Core 1
    // loop {
    //     engine.stream_audio(&mut audio_stream).await;
    // }
}
