//! Hardware initialization and configuration.

use esp_hal::{
    analog::adc::{Adc, AdcCalCurve, AdcConfig, AdcPin, Attenuation}, dma::DmaDescriptor, i2s::master::{asynch::I2sWriteDmaTransferAsync, DataFormat, I2s, Standard}, peripherals::ADC1, time::Rate, Blocking
};
use crate::config::SAMPLE_RATE;

/// Slim controller: own only the ADC peripheral.
pub struct AdcBus {
    pub adc: Adc<'static, ADC1<'static>, Blocking>,
}

/// Type aliases to make signatures readable.
pub type PotPin<P> = AdcPin<P, ADC1<'static>, AdcCalCurve<ADC1<'static>>>;

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

/// Generic setup: you pass *any* two GPIOs that are ADC1-capable.
/// We return the ADC bus + two configured pins (with calibration).
pub fn setup_adc<PF, PV>(
    adc1: ADC1<'static>,
    gpio_freq: PF,
    gpio_vol: PV,
) -> (AdcBus, PotPin<PF>, PotPin<PV>) {
    let mut cfg = AdcConfig::new();

    // 11 dB → full 0–3.3V, with curve calibration
    let freq_pin = cfg.enable_pin_with_cal::<_, AdcCalCurve<_>>(gpio_freq, Attenuation::_11dB);
    let vol_pin  = cfg.enable_pin_with_cal::<_, AdcCalCurve<_>>(gpio_vol,  Attenuation::_11dB);

    let adc = Adc::new(adc1, cfg);
    (AdcBus { adc }, freq_pin, vol_pin)
}
