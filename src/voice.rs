//! Generative voice combining rhythm, melody, and synthesis.
//!
//! Each voice generates its own patterns via Euclidean rhythms and Turing Machine,
//! then renders audio through oscillator and envelope.

use crate::config::{
    BASS_ANCHOR_INTERVAL, BASS_MUTATION_RATE, BASS_OCTAVE_RANGE, BASS_PORTAMENTO_COEFF,
    BASS_ROOT_HZ, BASS_TURING_LOOP_LENGTH, BASS_TURING_TICK_DIVISOR, EUCLIDEAN_INITIAL_HITS,
    LEAD_ANCHOR_INTERVAL, LEAD_EUCLIDEAN_LENGTH, LEAD_MUTATION_RATE, LEAD_OCTAVE_RANGE,
    LEAD_PORTAMENTO_COEFF, LEAD_ROOT_HZ, LEAD_TURING_LOOP_LENGTH, OSC_SILENCE_THRESHOLD,
};
use crate::envelope::ADEnvelope;
use crate::euclidean::EuclideanRhythm;
use crate::oscillator::{Oscillator, Waveform};
use crate::quantizer::Quantizer;
use crate::turing::TuringMachine;

/// Self-contained generative voice.
///
/// Two modes:
/// - Bass: Continuous drone (no Euclidean), slow Turing evolution, no envelope
/// - Lead: Rhythmic sequencer (Euclidean + Turing + envelope)
pub struct Voice {
    /// Euclidean rhythm generator (None for bass drone, Some for lead)
    pub euclidean: Option<EuclideanRhythm>,

    /// Turing Machine melody generator (produces evolving pitch sequences)
    pub turing: TuringMachine,

    /// Scale quantizer (snaps Turing output to musical scales)
    pub quantizer: Quantizer,

    /// Single oscillator (one per voice for clean spectral separation)
    pub osc: Oscillator,

    /// Amplitude envelope (None for bass drone, Some for lead articulation)
    pub envelope: Option<ADEnvelope>,

    /// Step counter for harmonic anchoring
    step_count: usize,

    /// Anchor interval (return to root every N steps)
    anchor_interval: usize,

    /// Root frequency for harmonic anchoring
    root_freq: f32,

    /// Turing tick divisor (bass ticks every N clocks, lead ticks every clock)
    turing_tick_divisor: usize,

    /// Clock counter for Turing tick divisor
    clock_counter: usize,
}

impl Voice {
    /// Private constructor (reused by bass() and lead()).
    fn new(
        euclidean: Option<(usize, usize)>, // (hits, length) or None for bass
        root_freq: f32,
        octave_range: f32,
        waveform: Waveform,
        turing_loop_length: usize,
        mutation_rate: f32,
        anchor_interval: usize,
        turing_tick_divisor: usize,
        portamento_coeff: f32,
        use_envelope: bool,
    ) -> Self {
        // Single oscillator per voice (clean spectral separation)
        let mut osc = Oscillator::new(waveform);
        osc.set_frequency(root_freq);
        osc.set_portamento_coeff(portamento_coeff);

        // Initialize Turing Machine
        let mut turing = TuringMachine::new();
        turing.set_loop_length(turing_loop_length);
        turing.set_mutation_rate(mutation_rate);

        // Initialize Euclidean rhythm if requested
        let euclidean_rhythm = euclidean.map(|(hits, length)| EuclideanRhythm::new(hits, length));

        // Initialize envelope if requested
        let envelope = if use_envelope {
            Some(ADEnvelope::new())
        } else {
            None
        };

        Self {
            euclidean: euclidean_rhythm,
            turing,
            quantizer: Quantizer::new(root_freq, octave_range),
            osc,
            envelope,
            step_count: 0,
            anchor_interval,
            root_freq,
            turing_tick_divisor,
            clock_counter: 0,
        }
    }

    /// Create bass voice (continuous drone with slow evolution).
    ///
    /// - Root: BASS_ROOT_HZ (deep bass territory)
    /// - Range: BASS_OCTAVE_RANGE octaves (subtle harmonic drift)
    /// - Turing: BASS_TURING_LOOP_LENGTH-bit loop, BASS_MUTATION_RATE mutation
    /// - Turing tick divisor: BASS_TURING_TICK_DIVISOR (slow evolution)
    /// - Anchoring: Root note every BASS_ANCHOR_INTERVAL Turing ticks
    /// - Euclidean: None (continuous sound)
    /// - Envelope: None (always full level)
    /// - Portamento: BASS_PORTAMENTO_COEFF (smooth glide)
    /// - Waveform: Sine (pure sub-bass foundation, no harmonics competing with lead)
    pub fn bass() -> Self {
        Self::new(
            None, // No Euclidean - continuous drone
            BASS_ROOT_HZ,
            BASS_OCTAVE_RANGE,
            Waveform::Sine, // Pure sine - clean low end
            BASS_TURING_LOOP_LENGTH,
            BASS_MUTATION_RATE,
            BASS_ANCHOR_INTERVAL,
            BASS_TURING_TICK_DIVISOR,
            BASS_PORTAMENTO_COEFF,
            false, // No envelope - continuous
        )
    }

    /// Create lead voice (rhythmic sequencer with envelope articulation).
    ///
    /// - Root: LEAD_ROOT_HZ (two octaves above bass)
    /// - Range: LEAD_OCTAVE_RANGE octaves (melodic without excessive jumps)
    /// - Turing: LEAD_TURING_LOOP_LENGTH-bit loop, LEAD_MUTATION_RATE mutation, ticks every clock
    /// - Anchoring: Root note every LEAD_ANCHOR_INTERVAL steps
    /// - Euclidean: Yes (rhythmic patterns via pot control)
    /// - Envelope: Yes (attack-decay articulation)
    /// - Portamento: LEAD_PORTAMENTO_COEFF (articulate glide)
    /// - Waveform: Saw (bright harmonics for melodic presence)
    pub fn lead() -> Self {
        Self::new(
            Some((EUCLIDEAN_INITIAL_HITS, LEAD_EUCLIDEAN_LENGTH)), // Euclidean rhythm
            LEAD_ROOT_HZ,
            LEAD_OCTAVE_RANGE,
            Waveform::Saw, // Bright saw for melodic lead
            LEAD_TURING_LOOP_LENGTH,
            LEAD_MUTATION_RATE,
            LEAD_ANCHOR_INTERVAL,
            1, // Tick every clock
            LEAD_PORTAMENTO_COEFF,
            true, // Use envelope
        )
    }

    /// Check if voice is audible (oscillator AND envelope active).
    ///
    /// Performance-critical: called every audio sample to skip silent voice rendering.
    /// For lead, skips rendering when envelope is Idle (between notes).
    /// Trades ~1.5ms of inaudible tail rendering for guaranteed attack preservation.
    #[inline]
    pub fn is_audible(&self) -> bool {
        // Check oscillator level first (short-circuit if silent)
        let osc_audible = self.osc.level.current() >= OSC_SILENCE_THRESHOLD;

        // If voice has envelope (lead), check if envelope is active (not Idle)
        // Must use stage check, not level check, to avoid cutting off note attacks
        if let Some(envelope) = &self.envelope {
            osc_audible && envelope.is_active()
        } else {
            // Bass voice (no envelope) - just check oscillator
            osc_audible
        }
    }

    /// Process one clock step (rhythm + melody generation).
    ///
    /// Two modes:
    /// - **Bass (no Euclidean)**: Ticks Turing every N clocks, updates pitch continuously
    /// - **Lead (with Euclidean)**: Ticks Turing every clock, triggers envelope on hits
    ///
    /// Anchoring: Both voices return to root every N Turing ticks to prevent stagnation
    #[inline]
    pub fn step(&mut self) {
        // Advance clock counter
        self.clock_counter += 1;

        // Check if we should tick Turing this clock cycle
        let should_tick_turing = (self.clock_counter % self.turing_tick_divisor) == 0;

        // Check Euclidean gate (lead) or always generate when Turing ticks (bass)
        let should_generate_note = if let Some(euclidean) = &mut self.euclidean {
            // Lead: Tick Euclidean and check if this step fires
            let gate = euclidean.tick();
            gate && should_tick_turing
        } else {
            // Bass: Generate note every time Turing ticks (no rhythm, continuous)
            should_tick_turing
        };

        if should_generate_note {
            // Check harmonic anchoring (every N Turing ticks)
            let should_anchor = self.step_count % self.anchor_interval == 0;
            self.step_count = self.step_count.wrapping_add(1);

            // Generate pitch: anchor to root or Turing + quantize
            let pitch_hz = if should_anchor {
                self.root_freq
            } else {
                let turing_value = self.turing.tick();
                // Bass: remap [-1, 1] → [0, 1] to keep notes above root (audible range)
                // Lead: use full [-1, 1] range for melodic motion above/below root
                let adjusted_value = if self.euclidean.is_none() {
                    // Bass mode: only go up from root (prevents sub-audible frequencies)
                    (turing_value + 1.0) * 0.5
                } else {
                    // Lead mode: full bipolar range
                    turing_value
                };
                self.quantizer.quantize(adjusted_value)
            };

            // Update oscillator (portamento smooths the transition)
            self.osc.set_frequency(pitch_hz);

            // Trigger envelope if present (lead only)
            if let Some(envelope) = &mut self.envelope {
                envelope.trigger();
            }
        }
    }

    /// Render one audio sample at audio rate.
    ///
    /// SUPER HOT PATH - called for every audio sample.
    ///
    /// # Returns
    /// Audio sample in range [-1.0, 1.0], modulated by envelope (lead) or at full level (bass)
    #[inline(always)]
    pub fn render(&mut self) -> f32 {
        // Skip rendering if voice is silent (CPU savings + prevents muted voices from triggering filter)
        if !self.is_audible() {
            return 0.0;
        }

        // Render single oscillator (clean spectral role per voice)
        let sample = self.osc.tick();

        // Apply envelope if present (lead), otherwise full level (bass drone)
        if let Some(envelope) = &mut self.envelope {
            let env_level = envelope.tick();
            sample * env_level
        } else {
            sample // Bass: always full level, continuous sound
        }
    }
}
