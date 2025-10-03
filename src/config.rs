//! Centralized configuration constants for the synth.
//! All magic numbers should live here to ensure consistency.

// === Synth Engine ===

/// Number of simultaneously mixable voices.
pub const VOICE_COUNT: usize = 3;

/// Audio sample rate in Hz.
///
/// 44.1 kHz keeps compatibility with consumer audio equipment.
pub const SAMPLE_RATE: u32 = 44_100;

/// Parameter smoothing coefficient for volume and other time-varying controls (0.0 to 1.0).
/// Higher values slow the response and help eliminate zipper noise from abrupt changes.
pub const VOLUME_SMOOTHING_COEFF: f32 = 0.99;

// === Voice Defaults ===

/// Starting frequency for all voices on initialization (Hz).
pub const STARTING_FREQUENCY: f32 = 77.0;

/// Default volume level for voices (0.0 to 1.0).
pub const DEFAULT_VOICE_VOLUME: f32 = 0.9;

// === Output Level Management ===

/// Minimum level in decibels for metering and UI.
pub const MIN_DB: f32 = -60.0;

/// Maximum level in decibels for metering and UI.
pub const MAX_DB: f32 = 0.0;

/// Master output gain applied after voice mixing (0.0 to 1.0).
/// Provides headroom even when all voices are at max volume (0.95 ≈ -0.45 dB).
pub const MASTER_GAIN: f32 = 0.85;

// === Wavetable ===

/// Wavetable size (must remain a power of two for fast wrapping).
pub const WAVETABLE_SIZE: usize = 1024;

/// Wavetable size as f32 (cached for hot paths to avoid repeated casts).
pub const WAVETABLE_SIZE_F32: f32 = WAVETABLE_SIZE as f32;

/// Wavetable index mask for rapid wrapping (SIZE - 1, valid because SIZE is a power of two).
pub const WAVETABLE_MASK: usize = WAVETABLE_SIZE - 1;

// === DMA & Streaming ===

/// DMA circular buffer size in bytes.
/// Must be divisible by 4 (stereo `i16` frame size) to keep channels aligned.
pub const DMA_BUFFER_SIZE: usize = 2044; // 1024, 2044 is solid

// === Messaging ===

/// Capacity of the control message queue.
pub const MESSAGE_QUEUE_SIZE: usize = 8;

// === Control & Input ===

// --- ADC Sampling ---

/// ADC polling interval in milliseconds.
pub const ADC_POLL_INTERVAL_MS: u64 = 20; // 15ms is stable

/// Number of ADC samples to average per reading (multisampling) for noise reduction.
pub const ADC_MULTISAMPLING_COUNT: usize = 9;

/// EMA filter alpha coefficient for ADC smoothing (0.0 to 1.0).
/// Higher values add smoothing; lower values respond faster to changes.
pub const ADC_EMA_ALPHA: f32 = 0.6; // lower -> responsiveness

// --- Potentiometer Scaling ---

/// Potentiometer minimum millivolt value.
pub const POT_MIN: u16 = 0;

/// Potentiometer maximum millivolt value (~3.156 V with 11 dB attenuation on ESP32-S3).
pub const POT_MAX: u16 = 3156;

/// Normalized threshold before treating a potentiometer change as meaningful.
/// 0.001 roughly maps to ~1 Hz increments across the usable range.
pub const POT_CHANGE_THRESHOLD: f32 = 0.001;

/// Exponent used when shaping the potentiometer response curve.
pub const POT_EXPONENT_SCALE: i32 = 2;

/// Minimum frequency target for potentiometer control (Hz).
pub const FREQUENCY_MIN: f32 = 30.0;

/// Maximum frequency target for potentiometer control (Hz).
pub const FREQUENCY_MAX: f32 = 1024.0;
