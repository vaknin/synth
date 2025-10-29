//! Hardware initialization and configuration.

use crate::{config::SAMPLE_RATE, AudioStream};
use esp_hal::{
    analog::adc::{Adc, AdcCalCurve, AdcPin},
    dma::{DmaDescriptor, DmaRxBuf, DmaTxBuf},
    dma_buffers,
    i2s::master::{DataFormat, I2s},
    interrupt::software::{SoftwareInterrupt, SoftwareInterruptControl},
    peripherals::{self, ADC1, SW_INTERRUPT},
    time::Rate,
    Blocking,
};

/// Slim controller: own only the ADC peripheral.
pub struct AdcBus {
    pub adc: Adc<'static, ADC1<'static>, Blocking>,
}

/// Type aliases to make signatures readable.
pub type PotPin<P> = AdcPin<P, ADC1<'static>, AdcCalCurve<ADC1<'static>>>;
pub type Adc1 = Adc<'static, ADC1<'static>, Blocking>;

// I2S peripheral type aliases (for audio setup)
type I2s0Peripheral = esp_hal::peripherals::I2S0<'static>;
type DmaChannel0 = esp_hal::peripherals::DMA_CH0<'static>;
type I2sBclkGpio = esp_hal::peripherals::GPIO37<'static>;
type I2sWsGpio = esp_hal::peripherals::GPIO35<'static>;
type I2sDoutGpio = esp_hal::peripherals::GPIO36<'static>;

// SPI Display peripheral type aliases (SSD1351 OLED)
type Spi2Peripheral = esp_hal::peripherals::SPI2<'static>;
type DmaChannel1 = esp_hal::peripherals::DMA_CH1<'static>;
type SpiDcGpio = esp_hal::peripherals::GPIO9<'static>; // DC (data/command select)
type SpiCsGpio = esp_hal::peripherals::GPIO10<'static>; // CS (chip select)
type SpiMosiGpio = esp_hal::peripherals::GPIO11<'static>; // MOSI (DIN on display)
type SpiSckGpio = esp_hal::peripherals::GPIO12<'static>; // SCK (CLK on display)
type SpiRstGpio = esp_hal::peripherals::GPIO13<'static>; // RST (reset)

// GPIO type aliases for ADC pins (2 pots - drone synth)
type GPIO1 = peripherals::GPIO1<'static>;
type GPIO2 = peripherals::GPIO2<'static>;

// Generate AnyAdcPin enum + setup_adc() function for 2 pots
adc_pins! {
    1: GPIO1,
    2: GPIO2,
}

/// Initialize I2S audio output and return ready-to-use DMA transaction.
///
/// Configures I2S in Philips standard stereo mode (16-bit samples),
/// sets up circular DMA transfer, and returns transaction ready for push_with().
///
/// # Pin Configuration (Non-ADC GPIOs)
/// - WS/LRCK (word select) => GPIO35
/// - DOUT/DIN (data out) => GPIO36
/// - BCLK (bit clock) => GPIO37
///
/// GPIO 35/36/37 have no ADC capability and don't conflict with I2C/SPI peripherals.
///
/// # Arguments
/// * `i2s0` - I2S0 peripheral
/// * `dma_channel` - DMA channel for circular buffer
/// * `ws_gpio` - WS/LCK pin
/// * `dout_gpio` - DOUT/DIN pin
/// * `bclk_gpio` - BCLK pin
/// * `tx_buffer` - DMA transmit buffer (from dma_circular_buffers! macro)
/// * `tx_descriptors` - DMA descriptors (from dma_circular_buffers! macro)
///
/// # Returns
/// Configured I2S DMA transaction ready for audio rendering
pub fn setup_audio(
    i2s0: I2s0Peripheral,
    dma_channel: DmaChannel0,
    ws_gpio: I2sWsGpio,
    dout_gpio: I2sDoutGpio,
    bclk_gpio: I2sBclkGpio,
    tx_buffer: &'static mut [u8],
    tx_descriptors: &'static mut [DmaDescriptor],
) -> AudioStream {
    use esp_hal::i2s::master::Config;

    let i2s_tx = I2s::new(
        i2s0,
        dma_channel,
        Config::new_tdm_philips()
            .with_data_format(DataFormat::Data16Channel16)
            .with_sample_rate(Rate::from_hz(SAMPLE_RATE)),
    )
    .unwrap()
    .into_async()
    .i2s_tx
    .with_bclk(bclk_gpio)
    .with_ws(ws_gpio)
    .with_dout(dout_gpio)
    .build(tx_descriptors);

    i2s_tx.write_dma_circular_async(tx_buffer).unwrap()
}

/// Initialize SPI for SSD1351 OLED display.
///
/// Configures SPI2 peripheral at 20 MHz for RGB OLED communication.
/// Uses IOMUX pins (GPIO10/11/12) for optimal high-speed performance.
/// Returns configured SPI bus and GPIO pins for SSD1351 driver.
///
/// # Pin Configuration
/// - DIN/MOSI => GPIO11 (SPI2 MOSI)
/// - CLK/SCK => GPIO12 (SPI2 SCK)
/// - CS => GPIO10 (chip select)
/// - DC => GPIO9 (data/command select)
/// - RST => GPIO13 (reset)
///
/// # Arguments
/// * `spi2` - SPI2 peripheral
/// * `dc_gpio` - DC pin (data/command select, GPIO9)
/// * `cs_gpio` - CS pin (chip select, GPIO10)
/// * `mosi_gpio` - MOSI pin (DIN on display, GPIO11)
/// * `sck_gpio` - SCK pin (CLK on display, GPIO12)
/// * `rst_gpio` - RST pin (reset, GPIO13)
///
/// # Returns
/// Tuple of (SPI bus, CS pin, DC pin, RST pin) ready for SSD1351 driver initialization
pub fn setup_spi_display(
    spi2: Spi2Peripheral,
    dma_channel: DmaChannel1,
    dc_gpio: SpiDcGpio,
    cs_gpio: SpiCsGpio,
    mosi_gpio: SpiMosiGpio,
    sck_gpio: SpiSckGpio,
    rst_gpio: SpiRstGpio,
) -> (
    esp_hal::spi::master::SpiDmaBus<'static, esp_hal::Blocking>,
    esp_hal::gpio::Output<'static>,
    esp_hal::gpio::Output<'static>,
    esp_hal::gpio::Output<'static>,
) {
    use esp_hal::gpio::{Level, Output, OutputConfig};
    use esp_hal::spi::master::{Config, Spi};
    use esp_hal::time::Rate;

    // Create DMA buffers (33KB for display framebuffer: 128×128×2 = 32768 bytes)
    // 10 descriptors: Each descriptor handles max 4092 bytes, 32768/4092 ≈ 8 needed
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(33000, 10);

    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).expect("Failed to create RX DMA buffer");
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).expect("Failed to create TX DMA buffer");

    // Configure SPI with DMA (in Blocking mode for cross-core transfer)
    // Will be converted to Async on Core 1 to register interrupt handler correctly
    let spi_bus = Spi::new(
        spi2,
        Config::default().with_frequency(Rate::from_hz(crate::config::DISPLAY_SPI_FREQUENCY_HZ)),
    )
    .expect("Failed to create SPI")
    .with_sck(sck_gpio)
    .with_mosi(mosi_gpio)
    .with_dma(dma_channel)
    .with_buffers(dma_rx_buf, dma_tx_buf);
    // NOTE: Do NOT call .into_async() here - peripheral must be Send to move to Core 1

    // Configure control pins as outputs
    let cs = Output::new(cs_gpio, Level::High, OutputConfig::default()); // CS idle high
    let dc = Output::new(dc_gpio, Level::Low, OutputConfig::default()); // DC default low (command mode)
    let rst = Output::new(rst_gpio, Level::High, OutputConfig::default()); // RST idle high (not in reset)

    (spi_bus, cs, dc, rst)
}

/// Initialize software interrupts for dual-core esp-rtos.
///
/// These interrupts are used internally by the RTOS kernel for context switching.
/// You never handle these yourself - they're plumbing for esp_rtos::start_second_core().
///
/// # Arguments
/// * `sw_interrupt` - SW_INTERRUPT peripheral
///
/// # Returns
/// Tuple of (SoftwareInterrupt<0>, SoftwareInterrupt<1>) for Core 0 and Core 1
pub fn setup_interrupts(
    sw_interrupt: SW_INTERRUPT<'static>,
) -> (SoftwareInterrupt<'static, 0>, SoftwareInterrupt<'static, 1>) {
    let sw_ints = SoftwareInterruptControl::new(sw_interrupt);
    (sw_ints.software_interrupt0, sw_ints.software_interrupt1)
}

// ADC setup function is generated by adc_pins! macro (see adc_pins! invocation above)
