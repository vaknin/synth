//! Hardware initialization and configuration.

use esp_hal::{
    analog::adc::{Adc, AdcCalCurve, AdcConfig, Attenuation},
    dma::DmaDescriptor,
    i2s::master::{asynch::I2sWriteDmaTransferAsync, DataFormat, I2s, Standard},
    time::Rate,
};
use crate::config::SAMPLE_RATE;

/// Initialize I2S audio output and return ready-to-use DMA transaction.
///
/// Configures I2S in Philips standard stereo mode (16-bit samples),
/// sets up circular DMA transfer, and returns transaction ready for push_with().
///
/// # Pin Configuration
/// - BCLK (bit clock) => GPIO7
/// - WS (word select) => GPIO8
/// - DOUT (data out) => GPIO9
///
/// # Arguments
/// * `i2s0` - I2S0 peripheral
/// * `dma_channel` - DMA channel for circular buffer
/// * `gpio7` - BCLK pin
/// * `gpio8` - WS pin
/// * `gpio9` - DOUT pin
/// * `tx_buffer` - DMA transmit buffer (from dma_circular_buffers! macro)
/// * `tx_descriptors` - DMA descriptors (from dma_circular_buffers! macro)
///
/// # Returns
/// Configured I2S DMA transaction ready for audio rendering
pub fn setup_audio(
    i2s0: esp_hal::peripherals::I2S0<'static>,
    dma_channel: esp_hal::peripherals::DMA_CH0<'static>,
    gpio7: esp_hal::peripherals::GPIO7<'static>,
    gpio8: esp_hal::peripherals::GPIO8<'static>,
    gpio9: esp_hal::peripherals::GPIO9<'static>,
    tx_buffer: &'static mut [u8],
    tx_descriptors: &'static mut [DmaDescriptor],
) -> I2sWriteDmaTransferAsync<'static, &'static mut [u8]> {
    let i2s_tx = I2s::new(
        i2s0,
        Standard::Philips,
        DataFormat::Data16Channel16,
        Rate::from_hz(SAMPLE_RATE),
        dma_channel,
    )
    .into_async()
    .i2s_tx
    .with_bclk(gpio7)
    .with_ws(gpio8)
    .with_dout(gpio9)
    .build(tx_descriptors);

    i2s_tx.write_dma_circular_async(tx_buffer).unwrap()
}

/// ADC controller for potentiometer inputs.
pub struct AdcController {
    pub adc: Adc<'static, esp_hal::peripherals::ADC1<'static>, esp_hal::Blocking>,
    pub freq_pin: esp_hal::analog::adc::AdcPin<
        esp_hal::peripherals::GPIO1<'static>,
        esp_hal::peripherals::ADC1<'static>,
        AdcCalCurve<esp_hal::peripherals::ADC1<'static>>,
    >,
    pub vol_pin: esp_hal::analog::adc::AdcPin<
        esp_hal::peripherals::GPIO2<'static>,
        esp_hal::peripherals::ADC1<'static>,
        AdcCalCurve<esp_hal::peripherals::ADC1<'static>>,
    >,
}

/// Initialize ADC for reading potentiometers.
///
/// Configures ADC1 with 11dB attenuation for full 0-3.3V range.
/// Sets up two analog pins for frequency and volume control.
///
/// # Pin Configuration
/// - GPIO1 → ADC1_CH0 (frequency potentiometer)
/// - GPIO2 → ADC1_CH1 (volume potentiometer)
///
/// # Arguments
/// * `adc1` - ADC1 peripheral
/// * `gpio1` - Frequency pot pin
/// * `gpio2` - Volume pot pin
///
/// # Returns
/// AdcController with configured ADC and pins
pub fn setup_adc(
    adc1: esp_hal::peripherals::ADC1<'static>,
    gpio1: esp_hal::peripherals::GPIO1<'static>,
    gpio2: esp_hal::peripherals::GPIO2<'static>,
) -> AdcController {
    let mut adc_config = AdcConfig::new();

    // Configure pins with 11dB attenuation AND curve calibration
    // AdcCalCurve uses polynomial error correction for better accuracy
    // Using patched version with Horner's method to fix overflow bug
    let freq_pin = adc_config.enable_pin_with_cal::<_, AdcCalCurve<_>>(gpio1, Attenuation::_11dB);
    let vol_pin = adc_config.enable_pin_with_cal::<_, AdcCalCurve<_>>(gpio2, Attenuation::_11dB);

    let adc = Adc::new(adc1, adc_config);

    AdcController { adc, freq_pin, vol_pin }
}
