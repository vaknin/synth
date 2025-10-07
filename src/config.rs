//! Centralized configuration constants for the synth.
//! All magic numbers should live here to ensure consistency.

// === Synth Engine ===

/// Number of simultaneously mixable voices.
pub const VOICE_COUNT: usize = 3;

/// Audio sample rate in Hz.
///
/// 44.1 kHz keeps compatibility with consumer audio equipment.
pub const SAMPLE_RATE: u32 = 44_100;

/// Universal parameter smoothing coefficient (0.0 to 1.0).
/// Used for all time-varying parameters (frequency, volume, filters, etc.)
/// Higher values = more smoothing = slower response. Prevents clicks and zipper noise.
/// 0.99 provides ~2-5ms transition at 44.1kHz sample rate.
pub const SMOOTHING_COEFF: f32 = 0.999;

// === Voice Defaults ===

/// Starting frequency for all voices on initialization (Hz).
pub const DEFAULT_FREQUENCY: f32 = 77.0;

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
/// 1024 bytes = 256 stereo i16 samples = 5.8ms latency @ 44.1kHz
pub const DMA_BUFFER_SIZE: usize = 1024;

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
pub const ADC_EMA_ALPHA: f32 = 0.8; // lower -> responsiveness

// --- Potentiometer Scaling ---

/// Potentiometer minimum millivolt value.
pub const POT_MIN: u16 = 0;

/// Potentiometer maximum millivolt value (~3.156 V with 11 dB attenuation on ESP32-S3).
pub const POT_MAX: u16 = 3156;

/// Normalized threshold before treating a potentiometer change as meaningful.
/// 0.001 roughly maps to ~1 Hz increments across the usable range.
pub const POT_CHANGE_THRESHOLD: f32 = 0.0005;

/// Exponent used when shaping the potentiometer response curve.
pub const POT_EXPONENT_SCALE: i32 = 2;

/// Minimum frequency target for potentiometer control (Hz).
pub const FREQUENCY_MIN: f32 = 30.0;

/// Maximum frequency target for potentiometer control (Hz).
pub const FREQUENCY_MAX: f32 = 1024.0;

/// Detune range in cents (applied as ±DETUNE_RANGE_CENTS).
/// 100 cents = 1 semitone, 200 cents = 2 semitones.
/// Larger values create more dramatic detuning effects for drone synthesis.
/// Range: pot center = 0¢, pot min = -DETUNE_RANGE_CENTS, pot max = +DETUNE_RANGE_CENTS
pub const DETUNE_RANGE_CENTS: f32 = 100.0;

// === State Variable Filter ===

/// Minimum cutoff frequency for State Variable Filter (Hz).
/// 20Hz is below the threshold of human hearing (useful for removing DC offset).
pub const FILTER_CUTOFF_MIN: f32 = 20.0;

/// Maximum cutoff frequency for State Variable Filter (Hz).
/// Limited to 0.4× sample_rate (17640Hz @ 44.1kHz) for Chamberlin SVF numerical stability.
/// The frequency coefficient f = 2πfc/fs must stay below 0.5 to prevent filter blow-up.
pub const FILTER_CUTOFF_MAX: f32 = SAMPLE_RATE as f32 * 0.4;

/// Default filter cutoff on initialization (Hz).
/// Set to maximum (wide open) so filter doesn't attenuate audio until pot is adjusted.
/// This prevents silence if GPIO5 pot starts at minimum position.
pub const FILTER_CUTOFF_DEFAULT: f32 = FILTER_CUTOFF_MAX;

/// Default filter resonance on initialization (dimensionless Q factor).
/// 0.7 provides gentle resonance without excessive peaking or ringing.
/// Range: 0.1 (heavily damped) to 10.0 (self-oscillation).
pub const FILTER_RESONANCE_DEFAULT: f32 = 0.7;

/// Filter integrator state limit (dimensionless, ±full scale amplitude).
/// Prevents unbounded growth of lowpass/bandpass integrator states at high resonance.
/// ±5.0 allows up to 5× amplification beyond normalized ±1.0 input (headroom for resonance peaks).
pub const FILTER_INTEGRATOR_LIMIT: f32 = 5.0;
