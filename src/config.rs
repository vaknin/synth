//! Centralized configuration constants for the synth.
//! All magic numbers should live here to ensure consistency.

/// Number of simultaneous voices
pub const VOICE_COUNT: usize = 3;

/// Audio sample rate in Hz
pub const SAMPLE_RATE: u32 = 44_100;

/// Starting frequency for all voices on initialization (Hz)
pub const STARTING_FREQUENCY: f32 = 77.0;

/// Default volume level for voices (0.0 to 1.0)
pub const DEFAULT_VOLUME: f32 = 0.9;

/// Master output gain applied after voice mixing (0.0 to 1.0)
/// Provides headroom even when all voices are at max volume
/// 0.65 ≈ -3.7dB headroom, 0.75 ≈ -2.5dB headroom
pub const MASTER_GAIN: f32 = 0.85;

/// DMA circular buffer size in bytes
/// Must be divisible by 4 (stereo i16 frame size)
pub const DMA_BUFFER_SIZE: usize = 2044; // 1024, 2044 is solid

/// Message queue capacity (number of messages)
pub const MESSAGE_QUEUE_SIZE: usize = 32;

// --- ADC Configuration ---
//
// Hardware: Connect 100nF ceramic bypass capacitor to ADC input pads
// for improved noise immunity (ESP-IDF recommendation)

/// ADC polling interval in milliseconds
pub const ADC_POLL_INTERVAL_MS: u64 = 15; //15ms

/// Number of ADC samples to average per reading (multisampling)
/// Reduces noise by factor of √N (4 samples ≈ 2x noise reduction)
/// Balance between noise reduction and processing overhead
pub const ADC_MULTISAMPLING_COUNT: usize = 9;

/// EMA filter alpha coefficient for ADC smoothing (0.0 to 1.0)
/// Higher values = more smoothing, less responsiveness
/// 0.8 gives good balance between noise rejection and responsiveness
pub const ADC_EMA_ALPHA: f32 = 0.6; // lower -> responsiveness

/// Potentiometer minimum mV value
/// ESP32-S3 at 11dB attenuation reads 0 at 0.00V
pub const POT_MIN: u16 = 0;

/// Potentiometer maximum mV value, i.e. around ~3.156V
pub const POT_MAX: u16 = 3156;

// --- Control Mapping ---

/// Minimum frequency for potentiometer control (Hz)
/// Sub-bass range lower limit
pub const FREQUENCY_MIN: f32 = 30.0;

/// Maximum frequency for potentiometer control (Hz)
/// Upper midrange limit for musical control
pub const FREQUENCY_MAX: f32 = 1024.0;

pub const POT_CHANGE_THRESHOLD: f32 = 0.001; // 0.001 -> ~1Hz granularity