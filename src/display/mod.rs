//! Display module for SSD1351 OLED (128×128 RGB).

pub mod task;

use crate::config::{
    DISPLAY_HEIGHT, DISPLAY_WIDTH, WAVEFORM_BUFFER_SIZE, WAVEFORM_SAFE_DISTANCE,
    WAVEFORM_SAMPLES_PER_PIXEL, WAVEFORM_SCROLL_RATE,
};
use crate::message::WaveformData;
use display_interface_spi::SPIInterface;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{gpio::Output, spi::master::SpiDmaBus};
use ssd1351::{
    display_async::DisplayAsync,
    mode::GraphicsModeAsync,
    properties::{DisplayRotation, DisplaySize},
};
use static_cell::StaticCell;

const FRAMEBUFFER_SIZE: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 2) as usize;
static FRAMEBUFFER: StaticCell<[u8; FRAMEBUFFER_SIZE]> = StaticCell::new();
type SpiDevice = ExclusiveDevice<SpiDmaBus<'static, esp_hal::Async>, Output<'static>, Delay>;

/// SSD1351 OLED driver with oscilloscope rendering (async SPI with DMA).
pub struct DisplayDriver {
    display: GraphicsModeAsync<SPIInterface<SpiDevice, Output<'static>>>,
    scroll_position_samples: f32,
    pub cached_waveform: WaveformData,
}

impl DisplayDriver {
    /// Initialize async display driver (converts SPI to async on THIS core, yields during init)
    ///
    /// IMPORTANT: Accepts SPI in Blocking mode and converts to Async internally.
    /// This ensures interrupt handlers are registered on the correct core (Core 1).
    pub async fn new(
        spi: SpiDmaBus<'static, esp_hal::Blocking>,
        cs: Output<'static>,
        dc: Output<'static>,
        mut rst: Output<'static>,
    ) -> Self {
        // Convert to Async mode on THIS core (Core 1) - registers interrupt handler here
        let spi_async = spi.into_async();

        let framebuffer = FRAMEBUFFER.init([0u8; FRAMEBUFFER_SIZE]);
        let spi_device = ExclusiveDevice::new(spi_async, cs, Delay).expect("Failed to create SPI device");
        let interface = SPIInterface::new(spi_device, dc);

        // Create async display instance
        let display_async = DisplayAsync::new(
            interface,
            DisplaySize::Display128x128,
            DisplayRotation::Rotate0,
        );

        let mut display = GraphicsModeAsync::new(display_async, framebuffer);

        // Reset and initialize (async)
        display.reset(&mut rst, &mut Delay).ok();
        display.init().await.ok();
        display.clear();

        Self {
            display,
            scroll_position_samples: (WAVEFORM_BUFFER_SIZE / 2) as f32,
            cached_waveform: WaveformData {
                samples: [0; WAVEFORM_BUFFER_SIZE],
                write_index: 0,
            },
        }
    }

    /// Oscilloscope rendering: average mode with connected lines (async).
    ///
    /// Performance: Drawing to RAM ~10µs (sync), flush via DMA 13.1ms (async, yields CPU).
    pub async fn draw_waveform(&mut self) {
        static mut FRAME_COUNT: u32 = 0;

        // Time-based scrolling: constant velocity matching production rate
        self.scroll_position_samples += WAVEFORM_SCROLL_RATE;

        // Wrap scroll position at buffer boundary
        if self.scroll_position_samples >= WAVEFORM_BUFFER_SIZE as f32 {
            self.scroll_position_samples -= WAVEFORM_BUFFER_SIZE as f32;
        }

        // Extract integer and fractional parts for smooth interpolation
        let scroll_pos_int = self.scroll_position_samples as usize;
        let scroll_frac = self.scroll_position_samples - scroll_pos_int as f32;

        // Safety: skip frame if not enough data ahead to safely read
        let distance = if self.cached_waveform.write_index >= scroll_pos_int {
            self.cached_waveform.write_index - scroll_pos_int
        } else {
            WAVEFORM_BUFFER_SIZE - scroll_pos_int + self.cached_waveform.write_index
        };

        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT % 60 == 0 {
                log::info!("draw_waveform: scroll_pos={}, write_idx={}, distance={}, safe_dist={}",
                    scroll_pos_int, self.cached_waveform.write_index, distance, WAVEFORM_SAFE_DISTANCE);
            }
        }

        if distance < WAVEFORM_SAFE_DISTANCE {
            unsafe {
                if FRAME_COUNT % 60 == 0 {
                    log::warn!("Skipping frame: insufficient data (distance={} < safe={})", distance, WAVEFORM_SAFE_DISTANCE);
                }
            }
            return;
        }

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
                let sample_a = self.cached_waveform.samples[idx] as f32;
                let sample_b = self.cached_waveform.samples[idx_next] as f32;
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

        // Async flush: DMA transfer yields CPU for 13.1ms, pot task runs concurrently!
        self.display.flush().await;
    }
}
