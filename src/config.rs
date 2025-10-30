//! Centralized configuration constants for the synth.
//! All magic numbers should live here to ensure consistency.

// === System Architecture ===

/// Stack size for Core 0 (PRO_CPU) - Main audio loop.
/// Configured by esp-rtos runtime, typically 8-16KB (not directly configurable in code).
/// Core 0 runs: Engine audio rendering + I2S DMA callbacks.
/// This constant is for documentation only - actual Core 0 stack is controlled by esp-rtos.
pub const CORE_0_STACK_SIZE_ESTIMATE: usize = 16384; // 16KB (runtime default)

/// Stack size for Core 1 (APP_CPU) - Embassy executor + ADC polling + display rendering.
/// Increased to 96KB to accommodate ADC init (with calibration data) + display framebuffer.
/// Stack usage breakdown:
/// - Embassy executor overhead (~2-4KB)
/// - pot_task: ADC init on Core 1 stack (~13KB calibration data)
/// - display_task SPI DMA setup + peripherals (~2KB)
/// - DisplayDriver initialization: 32KB framebuffer + ~2KB temporaries
/// - XTensa call stack depth + alignment padding (~5-10KB)
/// Total estimated ~54KB, 96KB provides safe headroom.
pub const CORE_1_STACK_SIZE: usize = 98304; // 96KB

// === Synth Engine ===

/// Number of simultaneously mixable voices.
pub const VOICE_COUNT: usize = 3;

/// Audio sample rate in Hz.
///
/// 44.1 kHz keeps compatibility with consumer audio equipment.
pub const SAMPLE_RATE: u32 = 44_100;

/// Sample rate as f32 (pre-computed to avoid runtime casts).
pub const SAMPLE_RATE_F32: f32 = SAMPLE_RATE as f32;

/// Buffer callback rate in Hz (DMA interrupt frequency).
/// Each stereo frame = 4 bytes (2 channels × 2 bytes per i16 sample).
/// Calculated as: SAMPLE_RATE / (DMA_BUFFER_SIZE / 4)
pub const BUFFER_RATE: f32 = SAMPLE_RATE as f32 / (DMA_BUFFER_SIZE / 4) as f32;

/// Universal parameter smoothing coefficient (0.0 to 1.0).
/// Used for all time-varying parameters (frequency, volume, filters, etc.)
/// Higher values = more smoothing = slower response. Prevents clicks and zipper noise.
/// 0.99 provides ~10ms settling time at 44.1kHz - settles within pot update interval (20ms).
pub const SMOOTHING_COEFF: f32 = 0.99;

/// Default portamento smoothing coefficient for oscillator pitch transitions.
/// Slower than SMOOTHING_COEFF for expressive, vocal-like melodic glides.
/// 0.995 provides ~50ms settling time at 44.1kHz for smooth generative melodies.
pub const PORTAMENTO_SMOOTHING_COEFF: f32 = 0.995;

/// Minimum portamento smoothing coefficient (fast transitions).
/// 0.95 provides ~10ms settling time for snappy, percussive melodies.
pub const PORTAMENTO_COEFF_MIN: f32 = 0.95;

/// Maximum portamento smoothing coefficient (slow transitions).
/// 0.998 provides ~150ms settling time for vocal-like, legato phrases.
pub const PORTAMENTO_COEFF_MAX: f32 = 0.998;

/// Slew limiter settling threshold (amplitude units).
/// When error between current and target falls below this threshold, snap to target and mark settled.
/// Threshold: 1/32768 ≈ 0.0000305 (-90 dB, well below 16-bit quantization noise floor).
/// Relaxed from 1/65536 for faster settling and reduced CPU load on ESP32-S3.
/// Optimization: Skips EMA computation when settled (parameters are static 90%+ of the time).
pub const SLEW_SETTLE_THRESHOLD: f32 = 1.0 / 32768.0;

// === Output Level Management ===

/// Minimum level in decibels for metering and UI.
pub const MIN_DB: f32 = -60.0;

/// Maximum level in decibels for metering and UI.
pub const MAX_DB: f32 = 0.0;

/// Oscillator silence threshold in linear amplitude (0.0-1.0).
/// Derived from MIN_DB: 10^(-60/20) = 0.001 (cannot use pow in const context).
/// Oscillators below this level skip waveform generation for CPU savings.
/// -60dB is well below audibility and allows significant optimization.
pub const OSC_SILENCE_THRESHOLD: f32 = 0.001;

/// PolyBLEP anti-aliasing threshold frequency (Hz).
/// Below this frequency, square/saw waves skip PolyBLEP as aliasing is inaudible.
/// At 1000 Hz, first aliased harmonic is the 23rd (-27dB down) at 21.1 kHz - barely audible.
/// Below 1000 Hz, aliases are even weaker and effectively inaudible for low-frequency synthesis.
/// This threshold covers the entire FREQUENCY_MAX range (768 Hz) with headroom.
pub const POLYBLEP_THRESHOLD_HZ: f32 = 1000.0;

/// Phase increment threshold for PolyBLEP optimization.
/// Oscillators with phase_inc below this skip PolyBLEP anti-aliasing.
/// Calculated as: POLYBLEP_THRESHOLD_HZ / SAMPLE_RATE
pub const POLYBLEP_THRESHOLD_PHASE_INC: f32 = POLYBLEP_THRESHOLD_HZ / SAMPLE_RATE as f32;

/// Master output gain applied after voice mixing (0.0 to 1.0).
/// Provides headroom even when all voices are at max volume (0.95 ≈ -0.45 dB).
pub const MASTER_GAIN: f32 = 0.90;

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
/// VALUES THAT WORK (Don't remove this line): 2044, 4084, 1024
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
/// IMPORTANT: poll_and_send() uses yield_now() between samples (not Timer::after).
pub const ADC_MULTISAMPLING_COUNT: usize = 9;

/// Reciprocal of ADC_MULTISAMPLING_COUNT for division elimination.
pub const ADC_MULTISAMPLING_COUNT_RECIPROCAL: f32 = 1.0 / ADC_MULTISAMPLING_COUNT as f32;

/// EMA filter alpha coefficient for ADC smoothing (0.0 to 1.0).
/// Higher values add smoothing; lower values respond faster to changes.
pub const ADC_EMA_ALPHA: f32 = 0.8;

// --- Potentiometer Scaling ---

/// Potentiometer minimum millivolt value.
pub const POT_MIN: u16 = 0;

/// Potentiometer maximum millivolt value (~3.156 V with 11 dB attenuation on ESP32-S3).
pub const POT_MAX: u16 = 3156;

/// Reciprocal of POT_MAX for division elimination.
pub const POT_MAX_RECIPROCAL: f32 = 1.0 / POT_MAX as f32;

/// Normalized threshold before treating a potentiometer change as meaningful.
/// 0.001 roughly maps to ~1 Hz increments across the usable range.
pub const POT_CHANGE_THRESHOLD: f32 = 0.0005;

/// Exponent used when shaping the potentiometer response curve.
pub const POT_EXPONENT_SCALE: i32 = 2;

/// Minimum frequency target for potentiometer control (Hz).
pub const FREQUENCY_MIN: f32 = 4.0;

/// Maximum frequency target for potentiometer control (Hz).
pub const FREQUENCY_MAX: f32 = 768.0;

/// Detune range in cents (applied as ±DETUNE_RANGE_CENTS).
/// 100 cents = 1 semitone, 200 cents = 2 semitones.
/// Larger values create more dramatic detuning effects for rich, evolving timbres.
/// Range: pot center = 0¢, pot min = -DETUNE_RANGE_CENTS, pot max = +DETUNE_RANGE_CENTS
pub const DETUNE_RANGE_CENTS: f32 = 1200.0 * 2.0;

/// Conversion factor from cents to octaves (1/1200).
/// Used in cents-to-frequency-ratio formula: ratio = 2^(cents * CENTS_TO_OCTAVES)
/// Pre-calculated to eliminate division in hot path (compiler optimization).
pub const CENTS_TO_OCTAVES: f32 = 1.0 / 1200.0;

// === Moog Ladder Filter ===

/// Minimum cutoff frequency for Moog Ladder Filter (Hz).
/// 20 Hz reaches sub-bass territory for extreme filtering effects.
/// Moog ladder handles the full audible spectrum with no stability constraints.
pub const FILTER_CUTOFF_MIN: f32 = FREQUENCY_MIN * 50.0;

/// Maximum cutoff frequency for Moog Ladder Filter (Hz).
/// Extended to 20 kHz to cover the full audible spectrum.
/// Moog ladder uses tan-based warping with saturation, stable across entire range.
/// At 20 kHz: g = tan(π × 20k/44.1k) ≈ 6.8, well within safe limits.
pub const FILTER_CUTOFF_MAX: f32 = 15000.0;

/// Minimum filter resonance (dimensionless Q factor).
/// 0.1 provides heavily damped filtering with no peak at cutoff.
pub const FILTER_RESONANCE_MIN: f32 = 0.1;

/// Maximum filter resonance (dimensionless Q factor).
/// 4.0 enables classic Moog self-oscillation - the "sweet spot" before harsh distortion.
/// At k≈4.0, loop gain reaches unity and filter whistles musically.
/// Higher values overdrive the feedback path, losing the characteristic Moog tone.
pub const FILTER_RESONANCE_MAX: f32 = 2.0;

/// Denormal prevention threshold for filter state variables.
/// Zeros out filter state when it decays below this value to prevent FPU slowdown.
/// 1e-15 is ~-300dB (completely inaudible, below -140dB noise floor) but well above
/// denormal range (< 1.18e-38). Critical for ESP32-S3 performance - denormals cause
/// 100× slowdown. Filter state reaches denormals in 2-26ms of silence.
pub const FILTER_DENORMAL_THRESHOLD: f32 = 1e-15;

// === LFO (Low-Frequency Oscillator) ===

/// Minimum LFO frequency (Hz).
/// 0.01 Hz = 100 second cycle (ultra-slow pad evolution).
pub const LFO_FREQUENCY_MIN: f32 = 0.01;

/// Maximum LFO frequency (Hz).
/// 10 Hz = 0.1 second cycle (rhythmic pulses, tremolo effects).
pub const LFO_FREQUENCY_MAX: f32 = 10.0;

/// Default LFO frequency on initialization (Hz).
/// 0.1 Hz = 10 second cycle (gentle evolution, safe default).
pub const LFO_FREQUENCY_DEFAULT: f32 = 0.1;

/// Default LFO depth on initialization (0.0-1.0).
/// Zero depth = no modulation until pot message arrives.
pub const LFO_DEPTH_DEFAULT: f32 = 0.0;

// === Generative Synthesis ===

/// Minimum octave range for generative voices (smoother, stepwise motion).
pub const GENERATIVE_OCTAVE_RANGE_MIN: f32 = 1.0;

/// Maximum octave range for generative voices (dramatic, wide jumps).
/// Determines lookup table size: ±(MAX × 12 semitones) = ±48 semitones.
pub const GENERATIVE_OCTAVE_RANGE_MAX: f32 = 4.0;

// === Display (SSD1351 OLED) ===

pub const DISPLAY_WIDTH: u32 = 128;
pub const DISPLAY_HEIGHT: u32 = 128;

/// Display refresh rate
pub const DISPLAY_REFRESH_RATE_HZ: u32 = 45;

/// Using IOMUX pins (GPIO10/11/12) for optimal high-speed performance.
/// Note: Exceeds datasheet spec - test stability thoroughly. Use 20_000_000 for safe mode.
/// 23Mhz seems to work fine.
pub const DISPLAY_SPI_FREQUENCY_HZ: u32 = 22_500_000;

// === Waveform Visualization ===

/// Circular buffer: 1024 samples (8× display width), ~2KB RAM.
pub const WAVEFORM_BUFFER_SIZE: usize = (DISPLAY_WIDTH as usize) * 8;

/// Oscilloscope zoom (time/div): 20ms shows ~4 cycles of 220Hz sine (clean), 50ms (wide), 100ms+ (too zoomed out).
/// Smaller = waveform appears wider/slower. Independent of refresh rate.
pub const WAVEFORM_TIME_WINDOW_MS: u32 = 20;

/// Decimation factor: fixed rate ensuring >= 1 sample/pixel at TIME_WINDOW.
/// Independent of refresh rate for consistent zoom across all configurations.
pub const WAVEFORM_DECIMATION_FACTOR: u32 = {
    let min_decimated_rate_needed = (DISPLAY_WIDTH * 1000) / WAVEFORM_TIME_WINDOW_MS;
    SAMPLE_RATE / min_decimated_rate_needed
};

/// Samples per pixel (uses decimated rate, not raw 44.1kHz).
/// Derived from TIME_WINDOW_MS, independent of refresh rate.
pub const WAVEFORM_SAMPLES_PER_PIXEL: f32 =
    ((SAMPLE_RATE / WAVEFORM_DECIMATION_FACTOR) as f32 * WAVEFORM_TIME_WINDOW_MS as f32 / 1000.0)
        / DISPLAY_WIDTH as f32;

/// Scroll rate for time-based continuous scrolling (samples per frame).
/// Matches decimated sample production rate: decimated_rate ÷ refresh_rate.
/// Ensures scroll velocity perfectly matches audio production at any refresh rate.
pub const WAVEFORM_SCROLL_RATE: f32 =
    (SAMPLE_RATE / WAVEFORM_DECIMATION_FACTOR) as f32 / DISPLAY_REFRESH_RATE_HZ as f32;
