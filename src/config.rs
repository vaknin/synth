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
/// 0.99 provides ~10ms settling time at 44.1kHz - settles within pot update interval (20ms).
pub const SMOOTHING_COEFF: f32 = 0.99;

/// Slew limiter settling threshold (amplitude units).
/// When error between current and target falls below this threshold, snap to target and mark settled.
/// Threshold: 1/32768 ≈ 0.0000305 (-90 dB, well below 16-bit quantization noise floor).
/// Relaxed from 1/65536 for faster settling and reduced CPU load on ESP32-S3.
/// Optimization: Skips EMA computation when settled (parameters are static 90%+ of the time).
pub const SLEW_SETTLE_THRESHOLD: f32 = 1.0 / 32768.0;

// === Voice Defaults ===

/// Starting frequency for all voices on initialization (Hz).
pub const DEFAULT_FREQUENCY: f32 = 77.0;

// === Output Level Management ===

/// Minimum level in decibels for metering and UI.
pub const MIN_DB: f32 = -60.0;

/// Maximum level in decibels for metering and UI.
pub const MAX_DB: f32 = 0.0;

/// Master output gain applied after voice mixing (0.0 to 1.0).
/// Provides headroom even when all voices are at max volume (0.95 ≈ -0.45 dB).
pub const MASTER_GAIN: f32 = 0.95;

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
/// 2048 bytes = 512 stereo i16 samples = 11.6ms latency @ 44.1kHz
/// Larger buffer provides more CPU time per callback, reducing underrun risk.
pub const DMA_BUFFER_SIZE: usize = 4084;

// === Messaging ===

/// Capacity of the control message queue (Core 1 → Core 0).
/// Sized for future expansion to ~20 potentiometers:
/// - Worst case: 20 pots change simultaneously = 20 messages/cycle
/// - At 50 Hz poll rate (20ms), 32 messages = 1.6 cycles buffering
/// - Provides headroom for control bursts without dropped messages
/// - Memory cost: 32 × ~12 bytes = 384 bytes (negligible on ESP32-S3)
pub const MESSAGE_QUEUE_SIZE: usize = 32;

// === Control & Input ===

// --- ADC Sampling ---

/// ADC polling interval in milliseconds.
/// 20ms provides responsive control (~50 polls/sec) - imperceptible to humans.
/// With dual-core architecture, ADC blocking on Core 1 doesn't affect audio on Core 0.
pub const ADC_POLL_INTERVAL_MS: u64 = 20;

/// Number of ADC samples to average per reading (multisampling) for noise reduction.
/// 9 samples provides √9 = 3× noise reduction for clean, stable readings.
/// Combined with EMA filter (alpha=0.8) for maximum signal quality.
/// Dual-core architecture allows high multisampling without audio impact.
pub const ADC_MULTISAMPLING_COUNT: usize = 9;

/// EMA filter alpha coefficient for ADC smoothing (0.0 to 1.0).
/// Higher values add smoothing; lower values respond faster to changes.
pub const ADC_EMA_ALPHA: f32 = 0.8;

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

// === Potentiometer Resonance Mapping ===

/// Minimum resonance value from pot (musical range, not safety clamp).
/// 0.3 provides subtle filtering with minimal resonant peak.
pub const POT_RESONANCE_MIN: f32 = 0.3;

/// Maximum resonance value from pot (musical range, not safety clamp).
/// 8.0 enables dramatic self-oscillation for tonal effects and drone synthesis.
/// Exponential mapping keeps lower range usable while providing drama at high end.
pub const POT_RESONANCE_MAX: f32 = 8.0;

// === State Variable Filter ===

/// Minimum cutoff frequency for State Variable Filter (Hz).
/// 60 Hz provides musical low-end filtering without over-darkening.
/// Can still remove fundamentals (osc range: 30-1024 Hz) for subtractive textures.
/// Previous 20 Hz was too extreme - compressed useful pot range into top 80% of travel.
pub const FILTER_CUTOFF_MIN: f32 = 60.0;

/// Chamberlin SVF stability margin (must be < 1.0 for stable operation).
/// The coefficient f = 2π × fc/fs must stay below 1.0. Using 0.95 provides 5% safety margin
/// to prevent runaway oscillation at maximum cutoff while still reaching ~6668Hz.
pub const FILTER_STABILITY_MARGIN: f32 = 0.95;

/// Maximum cutoff frequency for State Variable Filter (Hz).
/// Derived from Chamberlin SVF stability: f < 1.0, where f = 2π × fc / fs
/// With stability margin: fc_max = (fs / 2π) × 0.95 ≈ 6668Hz @ 44.1kHz
/// Theoretical limit is 7019Hz, but margin prevents instability at pot maximum.
pub const FILTER_CUTOFF_MAX: f32 =
    SAMPLE_RATE as f32 / (2.0 * core::f32::consts::PI) * FILTER_STABILITY_MARGIN;

/// Default filter cutoff on initialization (Hz).
/// 2 kHz provides balanced starting point - high enough for harmonic clarity,
/// low enough for warmth. User can sweep in both directions from this sweet spot.
/// Previous default (MAX) was too bright and gave no indication filter was working.
pub const FILTER_CUTOFF_DEFAULT: f32 = 2000.0;

/// Minimum filter resonance (dimensionless Q factor).
/// 0.1 provides heavily damped filtering with no peak at cutoff.
pub const FILTER_RESONANCE_MIN: f32 = 0.1;

/// Maximum filter resonance (dimensionless Q factor).
/// 10.0 enables strong self-oscillation for tonal effects.
pub const FILTER_RESONANCE_MAX: f32 = 10.0;

/// Default filter resonance on initialization (dimensionless Q factor).
/// 0.7 provides gentle resonance without excessive peaking or ringing.
pub const FILTER_RESONANCE_DEFAULT: f32 = 0.7;

/// Filter integrator state limit (dimensionless, ±full scale amplitude).
/// Prevents unbounded growth of lowpass/bandpass integrator states at high resonance.
/// ±5.0 allows up to 5× amplification beyond normalized ±1.0 input (headroom for resonance peaks).
pub const FILTER_INTEGRATOR_LIMIT: f32 = 5.0;
