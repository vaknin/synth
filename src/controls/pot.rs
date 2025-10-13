//! Potentiometer reading with EMA filtering and parameter mapping.

use crate::config::*;
use crate::controls::CtrlSender;
use crate::message::Message;
use log::warn;
use micromath::F32Ext;

// === Curve System: Elegant, composable parameter mapping ===

/// Input response curve applied to normalized pot value (0.0-1.0).
/// Shapes how pot movement translates to parameter changes.
#[derive(Debug, Clone, Copy)]
pub enum Curve {
    /// Linear response: y = x (direct 1:1 mapping)
    Linear,
    /// Power law: y = x^n (using POT_EXPONENT_SCALE from config)
    /// Compresses low end, spreads high end - perceptual scaling
    Power,
    /// Quadratic with sign preservation: y = sign(x-0.5) × (x-0.5)²
    /// Slow near center (0.5), fast at edges - ideal for detune!
    Quadratic,
    /// Cubic: y = (x-0.5)³
    /// Even slower near center - for subtle modulation depth control
    Cubic,
}

/// Output range mapping strategy.
/// Defines how the curved input maps to the final parameter value.
#[derive(Debug, Clone, Copy)]
pub enum Range {
    /// Linear interpolation: min + (max - min) × curved_value
    /// Equal spacing across range
    Linear { min: f32, max: f32 },
    /// Geometric scaling: min × ratio^curved_value
    /// Equal spacing across octaves/decades - perfect for pitch!
    /// `ratio` = max/min (pre-calculated to eliminate division)
    Geometric { min: f32, ratio: f32 },
    /// Bipolar/centered: maps 0.0-1.0 to -range to +range
    /// Input 0.5 → 0.0 (center), useful for detune, pan, modulation
    Bipolar { range: f32 },
}

impl Curve {
    /// Apply curve to normalized input (0.0-1.0).
    #[inline]
    fn apply(self, normalized: f32) -> f32 {
        match self {
            Curve::Linear => normalized,
            Curve::Power => normalized.powi(POT_EXPONENT_SCALE),
            Curve::Quadratic => {
                // Center at 0.5, apply quadratic, preserve sign for bipolar ranges
                let centered = normalized - 0.5; // -0.5 to +0.5
                let sign = centered.signum();
                let abs_curved = (centered.abs() * 2.0).powi(2) / 2.0; // 0.0 to 0.5
                sign * abs_curved + 0.5 // Map back to 0.0-1.0
            }
            Curve::Cubic => {
                // Cubic naturally preserves sign
                let centered = normalized - 0.5; // -0.5 to +0.5
                (centered * 2.0).powi(3) / 2.0 + 0.5 // Map back to 0.0-1.0
            }
        }
    }
}

impl Range {
    /// Map curved value to output range.
    #[inline]
    fn map(self, curved: f32) -> f32 {
        match self {
            Range::Linear { min, max } => min + (max - min) * curved,
            // Use micromath for embedded-optimized powf (no std dependency)
            // Ratio is pre-calculated (max/min) to eliminate runtime division
            Range::Geometric { min, ratio } => min * ratio.powf(curved),
            Range::Bipolar { range } => (curved - 0.5) * (range * 2.0),
        }
    }
}

/// Generic parameter mapper: curve + range composition.
///
/// Separates concerns:
/// - Curve: shapes input response (how pot feels)
/// - Range: maps to output domain (parameter value)
///
/// This eliminates code duplication - frequency mappings all use Geometric range,
/// but can have different input curves for different perceptual goals.
#[inline]
pub fn map_range(normalized: f32, curve: Curve, range: Range) -> f32 {
    let curved = curve.apply(normalized);
    range.map(curved)
}

/// Potentiometer with filtering, deadband, and parameter mapping.
///
/// Each pot owns its own signal processing state (deadband, multisampling),
/// sample buffer, and behavior (mapping function). Hardware (ADC, pins)
/// is owned by the control task that polls all pots sequentially.
pub struct Potentiometer {
    /// Last sent normalized value (for deadband detection)
    last_sent: f32,
    /// Mapping function from normalized value to Message
    map_fn: fn(f32) -> Message,
    /// Sample buffer for multisampling (reused each poll)
    samples: [u16; ADC_MULTISAMPLING_COUNT],
    /// EMA filtered value (ADC-side smoothing)
    filtered: f32,
    /// EMA alpha coefficient (currently unused - testing single-stage smoothing)
    #[allow(dead_code)]
    alpha: f32,
    /// Debug counter for occasional logging (every N sends)
    debug_counter: u32,
}

impl Potentiometer {
    /// Create new potentiometer with mapping function and deadband.
    ///
    /// # Arguments
    /// * `map_fn` - Function to map normalized value (0.0-1.0) to Message
    pub fn new(map_fn: fn(f32) -> Message) -> Self {
        Self {
            last_sent: 0.0,
            map_fn,
            samples: [0u16; ADC_MULTISAMPLING_COUNT],
            filtered: (POT_MAX / 2) as f32,
            alpha: ADC_EMA_ALPHA,
            debug_counter: 0,
        }
    }

    /// Read, filter, and conditionally send message if value changed significantly.
    ///
    /// Performs complete signal chain:
    /// 1. Multisampling (reduces noise by √N)
    /// 2. Averaging
    /// 3. EMA filtering (smooth out remaining noise)
    /// 4. Normalization (POT_MIN..POT_MAX → 0.0..1.0)
    /// 5. Deadband check (only send if change >= threshold)
    /// 6. Message mapping and send
    ///
    /// # Arguments
    /// * `sender` - Embassy channel sender
    /// * `pin` - Type-erased ADC pin (contains ADC reference internally)
    pub async fn poll_and_send(
        &mut self,
        sender: CtrlSender,
        pin: &mut crate::hardware::AnyAdcPin,
        adc: &mut crate::hardware::Adc1,
    ) {
        // 1. Multisample: read N samples into internal buffer
        for sample in self.samples.iter_mut() {
            *sample = pin.read_blocking(adc);
        }

        // 2. Average samples (multisampling reduces noise)
        let sum: u32 = self.samples.iter().map(|&s| s as u32).sum();
        let avg = sum as f32 * ADC_MULTISAMPLING_COUNT_RECIPROCAL;

        // 3. Single-stage smoothing: multisampling only (no EMA)
        // Oscillator params: Multisampling (9×) → Slew (audio rate 44.1kHz) for zipper prevention
        // Filter params: Multisampling (9×) → Direct (no Slew, coefficients cached at buffer rate)
        // If electrical noise is audible, re-enable EMA by uncommenting line below
        self.filtered = self.filtered * self.alpha + avg * (1.0 - self.alpha);
        // self.filtered = avg; // Direct use of averaged value

        // 4. Normalize to 0.0-1.0 range using calibrated min/max, defensive clamping
        let normalized = (self.filtered * POT_MAX_RECIPROCAL).clamp(0.0, 1.0);

        // 5. Deadband check: only send if changed significantly
        if (normalized - self.last_sent).abs() >= POT_CHANGE_THRESHOLD {
            self.last_sent = normalized;
            let msg = (self.map_fn)(normalized);

            // DEBUG: Log occasionally to diagnose "short stubs" issue
            self.debug_counter += 1;
            if self.debug_counter >= 10000 {
                log::info!("Pot: {:?} (norm: {:.3})", msg, normalized);
                self.debug_counter = 0;
            }

            if let Err(e) = sender.try_send(msg) {
                warn!("Pot message dropped (queue full): {:?}", e);
            }
        }
    }
}

// === Parameter Mapping Functions ===
// Each function specifies its curve + range for clear, maintainable behavior.

/// Map normalized value (0.0-1.0) to arpeggiator root pitch frequency.
///
/// Uses Linear + Geometric: equal octaves per pot rotation (musically correct).
/// 4-768 Hz = 7.58 octaves, each 10% pot travel = ~0.758 octaves.
/// Note: Bass drone remains fixed at BASS_FIXED_FREQ (55 Hz), independent of this control.
pub fn map_arp_pitch(normalized: f32) -> Message {
    let freq = map_range(
        normalized,
        Curve::Linear,
        Range::Geometric {
            min: FREQUENCY_MIN,
            ratio: FREQUENCY_MAX / FREQUENCY_MIN, // 768/4 = 192 (const-folded)
        },
    );
    Message::SetArpPitch(freq)
}

/// Map normalized value (0.0-1.0) to oscillator level.
///
/// Uses Power + Linear for logarithmic amplitude response.
/// Matches human hearing perception - feels smooth and even across pot travel.
pub fn map_osc_level(osc_id: crate::message::OscillatorId, normalized: f32) -> Message {
    let level = map_range(normalized, Curve::Power, Range::Linear { min: 0.0, max: 1.0 });
    Message::SetOscLevel(osc_id, level)
}

/// Map normalized value (0.0-1.0) to detune in cents.
///
/// Uses Quadratic + Bipolar for fine control near center (0¢),
/// fast changes at edges (±DETUNE_RANGE_CENTS).
/// Perfect for dialing in subtle beating effects!
pub fn map_osc_detune(osc_id: crate::message::OscillatorId, normalized: f32) -> Message {
    let cents = map_range(
        normalized,
        Curve::Quadratic,
        Range::Bipolar {
            range: DETUNE_RANGE_CENTS,
        },
    );
    Message::SetOscDetune(osc_id, cents)
}

/// Map normalized value (0.0-1.0) to filter cutoff frequency.
///
/// Uses Linear + Geometric for perceptually even octave spacing.
/// 200-15000 Hz = 6.23 octaves, equal precision across entire range.
pub fn map_cutoff(normalized: f32) -> Message {
    use crate::config::{FILTER_CUTOFF_MAX, FILTER_CUTOFF_MIN};

    let freq = map_range(
        normalized,
        Curve::Linear,
        Range::Geometric {
            min: FILTER_CUTOFF_MIN,
            ratio: FILTER_CUTOFF_MAX / FILTER_CUTOFF_MIN, // 15000/200 = 75 (const-folded)
        },
    );
    Message::SetCutoff(freq)
}

/// Map normalized value (0.0-1.0) to filter resonance.
///
/// Uses Linear + Linear for simple, predictable response.
/// Resonance (Q factor) is already perceptually uniform.
pub fn map_resonance(normalized: f32) -> Message {
    use crate::config::{FILTER_RESONANCE_MAX, FILTER_RESONANCE_MIN};

    let resonance = map_range(
        normalized,
        Curve::Linear,
        Range::Linear {
            min: FILTER_RESONANCE_MIN,
            max: FILTER_RESONANCE_MAX,
        },
    );
    Message::SetResonance(resonance)
}

/// Map normalized value (0.0-1.0) to LFO rate.
///
/// Uses Linear + Geometric for equal decades per pot rotation.
/// 0.01-10 Hz = 3 decades (0.01→0.1→1.0→10), musically useful range.
pub fn map_lfo_rate(normalized: f32) -> Message {
    use crate::config::{LFO_FREQUENCY_MAX, LFO_FREQUENCY_MIN};

    let rate = map_range(
        normalized,
        Curve::Linear,
        Range::Geometric {
            min: LFO_FREQUENCY_MIN,
            ratio: LFO_FREQUENCY_MAX / LFO_FREQUENCY_MIN, // 10/0.01 = 1000 (const-folded)
        },
    );
    Message::SetFilterLfoRate(rate)
}

/// Map normalized value (0.0-1.0) to LFO depth.
///
/// Uses Linear + Linear for direct, predictable modulation amount.
/// Engine converts depth to Hz range (±2 octaves from base cutoff).
pub fn map_lfo_depth(normalized: f32) -> Message {
    let depth = map_range(
        normalized,
        Curve::Linear,
        Range::Linear { min: 0.0, max: 1.0 },
    );
    Message::SetFilterLfoDepth(depth)
}

/// Map normalized value (0.0-1.0) to arpeggiator rate.
///
/// Uses Power + Geometric for perceptually even pacing across 2 decades.
/// 0.1-8 Hz = 1 note per 10 seconds → 8 notes per second.
/// Power curve emphasizes slow rates (more control at low end for ambient/generative).
pub fn map_arp_rate(normalized: f32) -> Message {
    use crate::config::{ARP_RATE_MAX, ARP_RATE_MIN};

    let rate = map_range(
        normalized,
        Curve::Power,
        Range::Geometric {
            min: ARP_RATE_MIN,
            ratio: ARP_RATE_MAX / ARP_RATE_MIN, // 8/0.1 = 80 (const-folded)
        },
    );
    Message::SetArpRate(rate)
}

/// Map normalized value (0.0-1.0) to arpeggiator mode.
///
/// Divides range into 4 equal zones for 4 modes (Up/Down/UpDown/Random).
/// 0.0-0.25 = Up, 0.25-0.5 = Down, 0.5-0.75 = UpDown, 0.75-1.0 = Random
pub fn map_arp_mode(normalized: f32) -> Message {
    use crate::message::ArpMode;

    let mode_index = ((normalized * 4.0).floor() as usize).min(3);
    let mode = match mode_index {
        0 => ArpMode::Up,
        1 => ArpMode::Down,
        2 => ArpMode::UpDown,
        _ => ArpMode::Random,
    };
    Message::SetArpMode(mode)
}

/// Map normalized value (0.0-1.0) to arpeggiator root transpose.
///
/// Uses Quadratic + Bipolar for fine control near center (0 semitones),
/// fast changes at edges (±12 semitones).
/// Center at 0.5 = no transpose, pot min = -12, pot max = +12.
pub fn map_arp_transpose(normalized: f32) -> Message {
    let semitones = map_range(normalized, Curve::Quadratic, Range::Bipolar { range: 12.0 });
    Message::SetArpTranspose(semitones.round() as i8)
}

/// Map normalized value (0.0-1.0) to envelope attack time.
///
/// Uses Power + Linear for perceptual uniformity - longer times more usable at high end.
/// 1-200ms range: pot min = instant (percussive), pot max = soft (pad-like).
/// Per CLAUDE.md: ~50ms portamento for smooth pitch transitions.
pub fn map_envelope_attack(normalized: f32) -> Message {
    use crate::config::{ENVELOPE_ATTACK_POT_MAX_MS, ENVELOPE_ATTACK_POT_MIN_MS};

    let attack_ms = map_range(
        normalized,
        Curve::Power,
        Range::Linear {
            min: ENVELOPE_ATTACK_POT_MIN_MS,
            max: ENVELOPE_ATTACK_POT_MAX_MS,
        },
    );
    Message::SetEnvelopeAttack(attack_ms)
}

/// Map normalized value (0.0-1.0) to envelope decay time.
///
/// Uses Power + Linear for perceptual uniformity - emphasizes shorter, musical decays.
/// 50-2000ms range: pot min = tight staccato, pot max = long sustain.
/// Per CLAUDE.md: "Generative melodies need long decays (300-800ms)"
pub fn map_envelope_decay(normalized: f32) -> Message {
    use crate::config::{ENVELOPE_DECAY_POT_MAX_MS, ENVELOPE_DECAY_POT_MIN_MS};

    let decay_ms = map_range(
        normalized,
        Curve::Power,
        Range::Linear {
            min: ENVELOPE_DECAY_POT_MIN_MS,
            max: ENVELOPE_DECAY_POT_MAX_MS,
        },
    );
    Message::SetEnvelopeDecay(decay_ms)
}
