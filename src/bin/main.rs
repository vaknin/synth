#![no_std]
#![no_main]

// use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::{Circle, PrimitiveStyle};
use embedded_graphics::text::Alignment::Center;
use embedded_graphics::text::Text;
use embedded_graphics::{mono_font, prelude::*};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::spi::master::Config;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, spi::master::Spi};
use ssd1351::builder::Builder;
use ssd1351::mode::GraphicsMode;
use ssd1351::prelude::SPIInterface;
use synth::display::hello_world;
use {esp_backtrace as _, esp_println as _};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    hello_world();

    // TODO: Spawn some tasks
    let _ = spawner;

    let rst = peripherals.GPIO13;
    let dc = peripherals.GPIO9;
    let sck = peripherals.GPIO12;
    let mosi = peripherals.GPIO11;
    let cs = peripherals.GPIO10;

    let dc_output = Output::new(dc, esp_hal::gpio::Level::Low, OutputConfig::default());
    let cs_output = Output::new(cs, esp_hal::gpio::Level::High, OutputConfig::default());
    let mut rst_output = Output::new(rst, esp_hal::gpio::Level::High, OutputConfig::default());

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default().with_frequency(Rate::from_mhz(15)),
    )
    .expect("Failed to create SPI")
    .with_sck(sck)
    .with_mosi(mosi);

    let spi_device = ExclusiveDevice::new(spi, cs_output, Delay).unwrap();
    let interface = SPIInterface::new(spi_device, dc_output);

    let mut display: GraphicsMode<_> = Builder::new().connect_interface(interface).into();

    display.reset(&mut rst_output, &mut Delay).unwrap();
    display.init().unwrap();

    // let rect = Rectangle::new(Point::new_equal(0), Size::new_equal(128));
    // let style_blue = PrimitiveStyle::with_fill(Rgb565::BLUE);
    // rect.into_styled(style_blue).draw(&mut display).unwrap();

    // let style_green = PrimitiveStyle::with_fill(Rgb565::YELLOW);
    // Circle::with_center(Point::new_equal(64), 10)
    //     .into_styled(style_green)
    //     .draw(&mut display)
    //     .unwrap();

    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    embedded_graphics::text::Text::with_alignment(
        "Hello\nWorld!",
        Point::new_equal(64),
        text_style,
        Center,
    )
    .draw(&mut display)
    .unwrap();

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
