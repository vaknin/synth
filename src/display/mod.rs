//! Display module for SSD1351 OLED (128×128 RGB).

pub mod task;

use crate::config::{
    DISPLAY_HEIGHT, DISPLAY_REFRESH_RATE_HZ, DISPLAY_WIDTH, WAVEFORM_BUFFER_SIZE,
    WAVEFORM_SAMPLES_PER_PIXEL, WAVEFORM_SCROLL_RATE,
};
use embassy_time::Delay;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::Text,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{gpio::Output, spi::master::SpiDmaBus};

use ssd1351::{
    display_async::DisplayAsync,
    mode::GraphicsModeAsync,
    prelude::SPIInterface,
    properties::{DisplayRotation, DisplaySize},
};
use static_cell::StaticCell;

const FRAMEBUFFER_SIZE: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 2) as usize;
static FRAMEBUFFER: StaticCell<[u8; FRAMEBUFFER_SIZE]> = StaticCell::new();

type SpiDeviceType = ExclusiveDevice<SpiDmaBus<'static, esp_hal::Async>, Output<'static>, Delay>;

/// Performance metrics for display profiling
pub struct DisplayMetrics {
    /// Time to flush via DMA (ms) - most important metric
    pub flush_ms: u64,
    /// Frame counter (wraps at u32::MAX)
    pub frame_count: u32,
    /// Last frame start timestamp for FPS calculation
    pub last_frame_start: Option<embassy_time::Instant>,
    /// Actual FPS (frames per second)
    pub fps: u32,
    /// Idle time (ms) - headroom remaining in frame budget
    pub idle_ms: u64,
    /// Idle percentage (0-100%) - headroom as % of frame budget
    pub idle_pct: u32,
    /// Cached diagnostic text (formatted every 8th frame, drawn every frame)
    pub cached_text: heapless::String<48>,
}

impl Default for DisplayMetrics {
    fn default() -> Self {
        Self {
            flush_ms: 0,
            frame_count: 0,
            last_frame_start: None,
            fps: 0,
            idle_ms: 0,
            idle_pct: 0,
            cached_text: heapless::String::new(),
        }
    }
}

/// SSD1351 OLED driver with oscilloscope rendering.
pub struct DisplayDriver {
    display: GraphicsModeAsync<SPIInterface<SpiDeviceType, Output<'static>>>,
    local_read_index: f32,
    pub cached_waveform: [i16; WAVEFORM_BUFFER_SIZE],
    animation_start: embassy_time::Instant,
    last_flush_ms: u64,
    pub metrics: DisplayMetrics,
}

impl DisplayDriver {
    /// Initialize display driver with async SPI + DMA
    pub async fn new(
        spi: SpiDmaBus<'static, esp_hal::Blocking>,
        cs: Output<'static>,
        dc: Output<'static>,
        mut rst: Output<'static>,
    ) -> Self {
        log::info!("Converting SPI to async mode...");
        let spi_async = spi.into_async();

        log::info!("Creating ExclusiveDevice with CS pin...");
        let spi_device =
            ExclusiveDevice::new(spi_async, cs, Delay).expect("Failed to create SPI device");
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
        log::info!("Test pattern flush took {}ms", duration.as_millis());

        Self {
            display,
            // Start at buffer beginning (same as audio write_index)
            // Display scrolls at WAVEFORM_SCROLL_RATE to naturally lag behind write head
            local_read_index: 0.0,
            cached_waveform: [0; WAVEFORM_BUFFER_SIZE],
            animation_start: embassy_time::Instant::now(),
            last_flush_ms: 0,
            metrics: DisplayMetrics::default(),
        }
    }

    /// Test animation with time-based movement and smooth wrapping.
    /// Time-based ensures animation speed is independent of DISPLAY_REFRESH_RATE_HZ.
    pub async fn draw_test_animation(&mut self) {
        // Time-based position (constant speed regardless of FPS)
        let elapsed = embassy_time::Instant::now() - self.animation_start;
        let elapsed_secs = elapsed.as_millis() as f32 / 1000.0;
        let pixels_per_second = 50.0; // Constant velocity
        let x_pos_float = (elapsed_secs * pixels_per_second) % DISPLAY_WIDTH as f32;
        let x_pos = x_pos_float as i32;

        // Clear framebuffer
        self.display.clear();

        use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};

        // Rectangle dimensions
        let rect_width = 20;
        let rect_height = 20;
        let y_pos = 50;

        let rect_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::GREEN)
            .build();

        // Draw main rectangle
        Rectangle::new(Point::new(x_pos, y_pos), Size::new(rect_width, rect_height))
            .into_styled(rect_style)
            .draw(&mut self.display)
            .ok();

        // Smooth wrapping: when rect extends past right edge, draw wrapped portion on left
        if x_pos + rect_width as i32 > DISPLAY_WIDTH as i32 {
            let wrapped_x = x_pos - DISPLAY_WIDTH as i32;
            Rectangle::new(
                Point::new(wrapped_x, y_pos),
                Size::new(rect_width, rect_height),
            )
            .into_styled(rect_style)
            .draw(&mut self.display)
            .ok();
        }

        // Draw flush time from previous frame at top
        use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
        use embedded_graphics::text::Text;

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        let mut text_buffer = heapless::String::<32>::new();
        use core::fmt::Write;
        write!(&mut text_buffer, "{}ms", self.last_flush_ms).ok();
        Text::new(&text_buffer, Point::new(2, 10), text_style)
            .draw(&mut self.display)
            .ok();

        // Flush and measure timing
        let flush_start = embassy_time::Instant::now();
        if let Err(e) = self.display.flush().await {
            log::error!("Flush failed: {:?}", e);
            panic!("Flush failed");
        }
        self.last_flush_ms = (embassy_time::Instant::now() - flush_start).as_millis();
    }

    /// Oscilloscope rendering: average mode with connected lines + diagnostic overlay.
    ///
    /// Performance: Drawing to RAM ~10µs (sync), flush via DMA ~13ms.
    pub async fn draw_waveform(&mut self) {
        let frame_start = embassy_time::Instant::now();

        // Calculate FPS from time since last frame
        if let Some(last_start) = self.metrics.last_frame_start {
            let frame_interval_ms = (frame_start - last_start).as_millis();
            if frame_interval_ms > 0 {
                self.metrics.fps = (1000 / frame_interval_ms) as u32;
            }
        }
        self.metrics.last_frame_start = Some(frame_start);

        // Time-based scrolling: constant velocity matching production rate
        self.local_read_index += WAVEFORM_SCROLL_RATE;

        // Wrap read position at buffer boundary
        if self.local_read_index >= WAVEFORM_BUFFER_SIZE as f32 {
            self.local_read_index -= WAVEFORM_BUFFER_SIZE as f32;
        }

        // Extract integer and fractional parts for smooth interpolation
        let scroll_pos_int = self.local_read_index as usize;
        let scroll_frac = self.local_read_index - scroll_pos_int as f32;

        // Clear
        self.display.clear();

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

        // === Diagnostic overlay (cached text approach) ===
        // UPDATE formatted string every 8th frame (reduces CPU)
        // DRAW cached string every frame (no flicker)
        // IMPORTANT: Must happen BEFORE flush, not after!
        if self.metrics.frame_count % 8 == 0 {
            use core::fmt::Write;
            self.metrics.cached_text.clear();
            write!(
                &mut self.metrics.cached_text,
                "{}fps {}ms {}%",
                self.metrics.fps, self.metrics.flush_ms, self.metrics.idle_pct,
            )
            .ok();
        }

        // Draw cached text every frame (top-left, yellow on green waveform = high contrast)
        let text_style = MonoTextStyle::new(&FONT_6X9, Rgb565::YELLOW);

        // Line 1: Performance metrics
        Text::new(&self.metrics.cached_text, Point::new(1, 8), text_style)
            .draw(&mut self.display)
            .ok();

        // Line 2: Frame counter (always shows, proves task is alive)
        let mut counter_buf = heapless::String::<16>::new();
        use core::fmt::Write;
        write!(&mut counter_buf, "#{}", self.metrics.frame_count).ok();
        Text::new(&counter_buf, Point::new(1, 17), text_style)
            .draw(&mut self.display)
            .ok();

        // === Flush to hardware ===
        let flush_start = embassy_time::Instant::now();
        if let Err(e) = self.display.flush().await {
            log::error!("Waveform flush failed: {:?}", e);
        }
        self.metrics.flush_ms = (embassy_time::Instant::now() - flush_start).as_millis();

        // Calculate total work time and idle headroom
        let work_ms = (embassy_time::Instant::now() - frame_start).as_millis();
        let frame_budget_ms: u64 = 1000 / DISPLAY_REFRESH_RATE_HZ as u64; // 33ms @ 30Hz
        self.metrics.idle_ms = frame_budget_ms.saturating_sub(work_ms);

        // Calculate idle as percentage of frame budget (0-100%)
        // Higher = more headroom = better
        // 15fps: 50ms/66ms = 75%, 30fps: 17ms/33ms = 51%, 45fps: 6ms/22ms = 27%
        if frame_budget_ms > 0 {
            self.metrics.idle_pct = ((self.metrics.idle_ms * 100) / frame_budget_ms) as u32;
        }

        self.metrics.frame_count = self.metrics.frame_count.wrapping_add(1);
    }
}
