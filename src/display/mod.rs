//! Display module for SSD1351 OLED (128×128 RGB).

pub mod task;

use crate::config::{
    DISPLAY_HEIGHT, DISPLAY_WIDTH, WAVEFORM_BUFFER_SIZE, WAVEFORM_SAMPLES_PER_PIXEL,
    WAVEFORM_SCROLL_RATE,
};
// ASYNC VERSION imports
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Delay;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use esp_hal::{gpio::Output, spi::master::SpiDmaBus};

// ASYNC VERSION: Using async ssd1351 fork
use ssd1351::{
    display_async::DisplayAsync,
    mode::GraphicsModeAsync,
    prelude::SPIInterface,
    properties::{DisplayRotation, DisplaySize},
};

use static_cell::StaticCell;

const FRAMEBUFFER_SIZE: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 2) as usize;
static FRAMEBUFFER: StaticCell<[u8; FRAMEBUFFER_SIZE]> = StaticCell::new();

// ASYNC VERSION types
type SpiBusType = Mutex<NoopRawMutex, SpiDmaBus<'static, esp_hal::Async>>;
type SpiDeviceType = SpiDevice<'static, NoopRawMutex, SpiDmaBus<'static, esp_hal::Async>, Output<'static>>;

/// SSD1351 OLED driver with oscilloscope rendering.
pub struct DisplayDriver {
    // ASYNC VERSION
    display: GraphicsModeAsync<SPIInterface<SpiDeviceType, Output<'static>>>,
    local_read_index: f32,
    pub cached_waveform: [i16; WAVEFORM_BUFFER_SIZE],
}

impl DisplayDriver {
    /// Initialize display driver - ASYNC VERSION
    ///
    /// Converts blocking SPI to async and wraps in Embassy SpiDevice
    pub async fn new(
        spi: SpiDmaBus<'static, esp_hal::Blocking>,
        cs: Output<'static>,
        dc: Output<'static>,
        mut rst: Output<'static>,
    ) -> Self {
        // ASYNC VERSION
        log::info!("Converting SPI to async mode on Core 1...");
        let spi_async = spi.into_async();
        log::info!("Wrapping SPI bus in Mutex for embassy SpiDevice...");
        static SPI_BUS: StaticCell<SpiBusType> = StaticCell::new();
        let spi_bus = SPI_BUS.init(Mutex::new(spi_async));
        log::info!("Creating embassy SpiDevice with CS pin...");
        let spi_device = SpiDevice::new(spi_bus, cs);
        let interface = SPIInterface::new(spi_device, dc);

        log::info!("Initializing framebuffer...");
        let framebuffer = FRAMEBUFFER.init([0u8; FRAMEBUFFER_SIZE]);

        log::info!("Creating DisplayAsync...");
        let display_async = DisplayAsync::new(
            interface,
            DisplaySize::Display128x128,
            DisplayRotation::Rotate0,
        );

        log::info!("Creating GraphicsModeAsync with buffer...");
        let mut display = GraphicsModeAsync::new(display_async, framebuffer);

        log::info!("Resetting and initializing display...");
        if let Err(e) = display.reset(&mut rst, &mut Delay) {
            log::error!("Display reset failed: {:?}", e);
            panic!("Display reset failed");
        }
        if let Err(e) = display.init().await {
            log::error!("Display init failed: {:?}", e);
            panic!("Display init failed");
        }
        log::info!("Display initialized successfully");

        // Draw test pattern to RAM buffer, then flush
        log::info!("Drawing test pattern (buffered)...");
        use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};

        Rectangle::new(Point::new(0, 0), Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .stroke_color(Rgb565::RED)
                    .stroke_width(2)
                    .build(),
            )
            .draw(&mut display)
            .ok();

        Line::new(Point::new(0, 0), Point::new(127, 127))
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 2))
            .draw(&mut display)
            .ok();
        Line::new(Point::new(127, 0), Point::new(0, 127))
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::BLUE, 2))
            .draw(&mut display)
            .ok();

        log::info!("Flushing test pattern to display (async)...");
        let start = embassy_time::Instant::now();
        if let Err(e) = display.flush().await {
            log::error!("Test pattern flush failed: {:?}", e);
            panic!("Test pattern flush failed");
        }
        let duration = embassy_time::Instant::now() - start;
        log::info!(
            "ASYNC: Test pattern flush took {}ms",
            duration.as_millis()
        );

        Self {
            display,
            // Start at buffer beginning (same as audio write_index)
            // Display scrolls at WAVEFORM_SCROLL_RATE to naturally lag behind write head
            local_read_index: 0.0,
            cached_waveform: [0; WAVEFORM_BUFFER_SIZE],
        }
    }

    /// Simple test animation: bouncing rectangle - ASYNC VERSION
    pub async fn draw_test_animation(&mut self, frame: u32) {
        // Log every frame for first 5, then periodically
        if frame < 5 {
            log::info!("Frame {}: Starting test animation", frame);
        }

        // Draw to RAM buffer (fast) - sync in buffered mode
        self.display.clear(); // clear to black (only touches RAM)

        use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};

        let x_pos = ((frame * 2) % DISPLAY_WIDTH) as i32;

        Rectangle::new(Point::new(x_pos, 50), Size::new(20, 20))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::GREEN)
                    .build(),
            )
            .draw(&mut self.display)
            .ok();

        // ASYNC flush
        if frame < 5 {
            log::info!("Frame {}: Calling async flush()...", frame);
        }

        let start = embassy_time::Instant::now();
        if let Err(e) = self.display.flush().await {
            log::error!("Frame {}: Flush failed: {:?}", frame, e);
            panic!("Flush failed");
        }
        let duration = embassy_time::Instant::now() - start;

        // Calculate effective SPI clock from timing
        let duration_ms = duration.as_millis();
        let bytes_transferred = FRAMEBUFFER_SIZE;
        let effective_mhz = if duration_ms > 0 {
            (bytes_transferred as u64 * 8) / (duration_ms * 1000)
        } else {
            0
        };

        if frame < 10 || frame % 30 == 0 {
            log::info!(
                "Frame {}: FLUSH completed: {}ms ({}KB) → {}MHz (expect 20MHz @ 13ms)",
                frame,
                duration_ms,
                bytes_transferred / 1024,
                effective_mhz
            );
        }
    }

    /// Oscilloscope rendering: average mode with connected lines.
    ///
    /// Performance: Drawing to RAM ~10µs (sync), flush via DMA.
    /// ASYNC VERSION
    pub async fn draw_waveform(&mut self) {
        static mut FRAME_COUNT: u32 = 0;
        static mut FLUSH_START_MS: u64 = 0;

        // Time-based scrolling: constant velocity matching production rate
        self.local_read_index += WAVEFORM_SCROLL_RATE;

        // Wrap read position at buffer boundary
        if self.local_read_index >= WAVEFORM_BUFFER_SIZE as f32 {
            self.local_read_index -= WAVEFORM_BUFFER_SIZE as f32;
        }

        // Extract integer and fractional parts for smooth interpolation
        let scroll_pos_int = self.local_read_index as usize;
        let scroll_frac = self.local_read_index - scroll_pos_int as f32;

        // Periodic logging with sample inspection
        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT % 60 == 0 {
                // Sample a few values from the cached buffer to verify data
                let s0 = self.cached_waveform[0];
                let s256 = self.cached_waveform[256];
                let s512 = self.cached_waveform[512];
                let s768 = self.cached_waveform[768];
                log::info!(
                    "draw: read_idx={}, samples[0,256,512,768]=[{},{},{},{}]",
                    scroll_pos_int,
                    s0,
                    s256,
                    s512,
                    s768
                );
            }
        }

        self.display.clear(); // clear to black (only touches RAM)

        let center_y = (DISPLAY_HEIGHT / 2) as i32;
        let scale = 64.0 / i16::MAX as f32;
        let style = PrimitiveStyle::with_stroke(Rgb565::GREEN, 1);

        // Fixed-point accumulator (16.16) for even sample distribution
        let samples_per_pixel_fixed = (WAVEFORM_SAMPLES_PER_PIXEL * 65536.0) as u32;
        let mut accumulator: u32 = 0;
        let step: u32 = 1 << 16;

        let mut sample_idx = 0;
        let total_samples = (WAVEFORM_SAMPLES_PER_PIXEL * DISPLAY_WIDTH as f32) as usize;

        // Track previous Y coordinate for drawing connected lines
        let mut prev_y: Option<i32> = None;

        for pixel_x in 0..DISPLAY_WIDTH as usize {
            let mut sample_sum: i32 = 0;
            let mut samples_read = 0;

            // Accumulate samples for this pixel column
            loop {
                if sample_idx >= total_samples {
                    break;
                }

                let idx = (scroll_pos_int + sample_idx) % WAVEFORM_BUFFER_SIZE;
                let idx_next = (idx + 1) % WAVEFORM_BUFFER_SIZE;

                // Linear interpolation for smooth scrolling
                let sample_a = self.cached_waveform[idx] as f32;
                let sample_b = self.cached_waveform[idx_next] as f32;
                let sample_interp = sample_a + (sample_b - sample_a) * scroll_frac;

                sample_sum += sample_interp as i32;
                samples_read += 1;

                accumulator += step;
                sample_idx += 1;

                if accumulator >= samples_per_pixel_fixed {
                    accumulator -= samples_per_pixel_fixed;
                    break;
                }
            }

            if samples_read > 0 {
                let avg_sample = (sample_sum / samples_read) as i16;
                let y_screen = center_y - (avg_sample as f32 * scale) as i32;

                if let Some(py) = prev_y {
                    Line::new(
                        Point::new((pixel_x - 1) as i32, py),
                        Point::new(pixel_x as i32, y_screen),
                    )
                    .into_styled(style)
                    .draw(&mut self.display)
                    .ok();
                } else {
                    embedded_graphics::Pixel(Point::new(pixel_x as i32, y_screen), Rgb565::GREEN)
                        .draw(&mut self.display)
                        .ok();
                }

                prev_y = Some(y_screen);
            }
        }

        // Flush (ASYNC VERSION)
        unsafe {
            FLUSH_START_MS = embassy_time::Instant::now().as_millis();
        }

        if let Err(e) = self.display.flush().await {
            log::error!("Waveform flush failed: {:?}", e);
        }

        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT % 60 == 0 {
                let duration_ms = embassy_time::Instant::now().as_millis() - FLUSH_START_MS;
                log::info!("Display flush: {}ms", duration_ms);
            }
        }
    }
}
