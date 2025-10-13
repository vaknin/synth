//! Centralized configuration constants for the synth.
//! All magic numbers should live here to ensure consistency.

// === System Architecture ===

/// Stack size for Core 1 (APP_CPU) - Embassy executor + ADC polling task.
/// 4KB provides headroom for current 6-pot system (~2.5KB) and future 20-pot expansion (~3.5KB).
pub const APP_CORE_STACK_SIZE: usize = 4096; //4096 is a pretty good/safe value

// === Synth Engine ===

/// Number of simultaneously mixable voices.
pub const VOICE_COUNT: usize = 2;

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
/// Provides headroom even when all voices are at max volume.
pub const MASTER_GAIN: f32 = 0.90;

/// Bass voice gain (continuous drone, attenuated to prevent overpowering lead).
/// Lower than lead because: continuous (no envelope), psychoacoustic bass loudness,
/// and to leave space for melodic elements.
pub const BASS_GAIN: f32 = 0.55;

/// Lead voice gain (melodic prominence with headroom for overlapping notes).
/// With 12 overlapping notes at max density, this prevents clipping while
/// maintaining clarity over the bass foundation.
pub const LEAD_GAIN: f32 = 0.75;

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
/// Dual-core architecture allows high multisampling without audio impact.
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
/// 80 Hz allows bass fundamentals through while enabling deep filtering.
/// Moog ladder handles the full audible spectrum with no stability constraints.
pub const FILTER_CUTOFF_MIN: f32 = 80.0;

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
pub const FILTER_RESONANCE_MAX: f32 = 3.0;

/// Denormal prevention threshold for filter state variables.
/// Zeros out filter state when it decays below this value to prevent FPU slowdown.
/// 1e-15 is ~-300dB (completely inaudible, below -140dB noise floor) but well above
/// denormal range (< 1.18e-38). Critical for ESP32-S3 performance - denormals cause
/// 100× slowdown. Filter state reaches denormals in 2-26ms of silence.
pub const FILTER_DENORMAL_THRESHOLD: f32 = 1e-15;

// === LFO (Low-Frequency Oscillator) ===

/// Minimum LFO frequency (Hz).
/// 0.1 Hz = 10 second cycle (slow atmospheric evolution).
pub const LFO_FREQUENCY_MIN: f32 = 0.1;

/// Maximum LFO frequency (Hz).
/// 20 Hz = 0.05 second cycle (fast tremolo, rhythmic pulses).
pub const LFO_FREQUENCY_MAX: f32 = 30.0;

/// Default LFO frequency on initialization (Hz).
/// 0.1 Hz = 10 second cycle (gentle evolution, safe default).
pub const LFO_FREQUENCY_DEFAULT: f32 = 0.1;

/// Default LFO depth on initialization (0.0-1.0).
/// Zero depth = no modulation until pot message arrives (~20ms).
pub const LFO_DEPTH_DEFAULT: f32 = 0.0;

// === Generative Synthesis ===

/// Minimum octave range for generative voices (smoother, stepwise motion).
pub const GENERATIVE_OCTAVE_RANGE_MIN: f32 = 1.0;

/// Maximum octave range for generative voices (dramatic, wide jumps).
/// Determines lookup table size: ±(MAX × 12 semitones) = ±48 semitones.
pub const GENERATIVE_OCTAVE_RANGE_MAX: f32 = 4.0;

// === Voice Architecture ===

/// Octave multiplier (one octave = 2× frequency).
pub const OCTAVE_MULTIPLIER: f32 = 2.0;

/// Two octaves separation for harmonic voice alignment.
pub const TWO_OCTAVES: f32 = OCTAVE_MULTIPLIER * OCTAVE_MULTIPLIER;

/// Bass voice root frequency (F1 - deep but audible bass territory).
/// 45 Hz balances sub-bass power with melodic clarity and speaker translation.
pub const BASS_ROOT_HZ: f32 = 45.0;

/// Bass voice octave range (subtle, drone-like movement).
pub const BASS_OCTAVE_RANGE: f32 = 1.0;

/// Lead voice root frequency (C4 - 2 octaves above bass for harmonic alignment).
/// Both voices share same root note (C) for consonant polyrhythmic interplay.
pub const LEAD_ROOT_HZ: f32 = BASS_ROOT_HZ * TWO_OCTAVES;

/// Lead voice octave range (melodic jumps without excessive leaps).
/// 1.5 octaves balances expressiveness with stepwise melodic contour.
pub const LEAD_OCTAVE_RANGE: f32 = 1.5;

/// Initial euclidean hits (immediately overwritten by pot ADC values).
/// Set to 1 for safe, silent startup until pots are read.
pub const EUCLIDEAN_INITIAL_HITS: usize = 1;

/// Bass voice euclidean pattern length (fixed, no pot control).
/// Standard 4/4 bar subdivision for solid rhythmic foundation.
pub const BASS_EUCLIDEAN_LENGTH: usize = 8;

/// Lead voice euclidean pattern length (fixed, no pot control).
/// Creates clean 8:16 polyrhythm with bass for danceable grooves.
pub const LEAD_EUCLIDEAN_LENGTH: usize = 16;

/// Initial euclidean length for lead voice (overwritten by pot ADC).
pub const EUCLIDEAN_INITIAL_LENGTH: usize = 8;

// === Turing Machine ===

/// Minimum loop length in bits for Turing Machine shift register.
/// 2 bits = ultra-fast melodic cells (4 possible states).
pub const TURING_LOOP_LENGTH_MIN: usize = 2;

/// Maximum loop length in bits for Turing Machine shift register.
/// 32 bits = very long evolving sequences before repeat.
pub const TURING_LOOP_LENGTH_MAX: usize = 32;

/// Default loop length on initialization (bits).
/// 16 bits = medium-length melodic phrases.
pub const TURING_LOOP_LENGTH_DEFAULT: usize = 16;

/// Bass voice Turing loop length (fixed, no pot control).
/// 8 bits = 256 states for smooth evolving bass patterns.
pub const BASS_TURING_LOOP_LENGTH: usize = 8;

/// Lead voice Turing loop length (fixed, no pot control).
/// 20 bits = medium-long evolving phrases without excessive randomness.
pub const LEAD_TURING_LOOP_LENGTH: usize = 20;

/// Maximum mutation rate (probability per step, 0.0-0.5).
/// 0.5 = 50% chance to flip each bit (maximum useful chaos).
pub const TURING_MUTATION_MAX: f32 = 0.5;

/// Default mutation rate on initialization.
/// 0.05 = 5% mutation (slowly evolving, mostly stable).
pub const TURING_MUTATION_DEFAULT: f32 = 0.05;

/// Bass voice mutation rate (fixed, no pot control).
/// 2% = very stable drone, glacial evolution.
pub const BASS_MUTATION_RATE: f32 = 0.25;

/// Lead voice mutation rate (fixed, no pot control).
/// 25% = melodic exploration, restless but musical.
pub const LEAD_MUTATION_RATE: f32 = 0.25;

// === Harmonic Anchoring ===

/// Bass voice harmonic anchoring interval (steps).
/// Returns to root note every 8 steps for solid tonal foundation.
pub const BASS_ANCHOR_INTERVAL: usize = 8; // must be >= 1

/// Lead voice harmonic anchoring interval (steps).
/// Returns to root note every 16 steps for melodic resolution points.
pub const LEAD_ANCHOR_INTERVAL: usize = 16;

/// Bass Turing tick divisor (ticks every N clock steps).
/// 16 = bass changes pitch every 16 clock ticks (every ~2 seconds @ 120 BPM).
/// Slow evolution creates hypnotic, drifting bass foundation under rhythmic lead.
pub const BASS_TURING_TICK_DIVISOR: usize = 16;

/// Bass portamento coefficient (slower, more vocal-like glides).
/// 0.997 provides ~100ms glide for smooth harmonic transitions.
pub const BASS_PORTAMENTO_COEFF: f32 = 0.997;

/// Lead portamento coefficient (faster, more articulate).
/// 0.995 provides ~50ms glide for legato but responsive melodies.
pub const LEAD_PORTAMENTO_COEFF: f32 = 0.995;

// === Master Clock ===

/// Minimum tempo in beats per minute.
/// 80 BPM = deep techno, ambient territory.
pub const CLOCK_BPM_MIN: f32 = 20.0;

/// Maximum tempo in beats per minute.
/// 180 BPM = fast techno, high-energy performance.
pub const CLOCK_BPM_MAX: f32 = 220.0;

/// Default tempo on initialization (BPM).
/// 120 BPM = standard dance music tempo.
pub const CLOCK_BPM_DEFAULT: f32 = 120.0;

/// Clock step resolution (steps per beat).
/// 4 = 16th notes (standard for techno/electronic rhythms).
/// Change to 2 for 8th notes, 1 for quarter notes.
pub const CLOCK_STEPS_PER_BEAT: usize = 4;

/// Tempo slew time in milliseconds.
/// Time to smoothly transition between BPM values.
/// 500ms = responsive but smooth tempo changes for live performance.
pub const CLOCK_TEMPO_SLEW_MS: f32 = 500.0;

// === Envelope ===

/// Envelope attack time in milliseconds (fixed).
/// Fast attack for percussive, punchy articulation.
pub const ENVELOPE_ATTACK_MS: f32 = 10.0;

/// Default envelope decay time in milliseconds.
/// Sustained decay for flowing generative melodies with overlapping phrases.
pub const ENVELOPE_DEFAULT_DECAY_MS: f32 = 400.0;

/// Minimum envelope decay time for randomization.
/// Short end of dynamic decay range for rhythmic, percussive articulation.
pub const ENVELOPE_DECAY_MIN_MS: f32 = 1000.0;

/// Maximum envelope decay time for randomization.
/// Long end of dynamic decay range for sustained, flowing phrases.
pub const ENVELOPE_DECAY_MAX_MS: f32 = 1500.0;

/// Pot-controllable attack range: minimum (instant, percussive).
pub const ENVELOPE_ATTACK_POT_MIN_MS: f32 = 1.0;

/// Pot-controllable attack range: maximum (soft, pad-like).
pub const ENVELOPE_ATTACK_POT_MAX_MS: f32 = 200.0;

/// Pot-controllable decay range: minimum (tight techno stabs).
pub const ENVELOPE_DECAY_POT_MIN_MS: f32 = 10.0;

/// Pot-controllable decay range: maximum (long sustain for melodies).
pub const ENVELOPE_DECAY_POT_MAX_MS: f32 = 2000.0;

// === Rhythm-Synced Filter ===

/// Maximum filter envelope depth in Hz (pot-controlled from 0 to this value).
/// Filter jumps by this amount on Euclidean gate, then decays back to baseline.
pub const FILTER_ENVELOPE_DEPTH_MAX: f32 = 500.0;

/// Filter envelope decay coefficient (exponential decay).
/// 0.992 provides tempo-synced decay (~200ms at buffer rate ~172 Hz).
pub const FILTER_ENVELOPE_DECAY_COEFF: f32 = 0.992;
