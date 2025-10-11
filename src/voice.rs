//! Generative voice combining rhythm, melody, and synthesis.
//!
//! Each voice generates its own patterns via Euclidean rhythms and Turing Machine,
//! then renders audio through oscillator and envelope.

use crate::config::{
    BASS_EUCLIDEAN_LENGTH, BASS_OCTAVE_RANGE, BASS_ROOT_HZ, ENVELOPE_DECAY_MAX_MS,
    ENVELOPE_DECAY_MIN_MS, EUCLIDEAN_INITIAL_HITS, EUCLIDEAN_INITIAL_LENGTH, LEAD_OCTAVE_RANGE,
    LEAD_ROOT_HZ,
};
use crate::envelope::ADEnvelope;
use crate::euclidean::EuclideanRhythm;
use crate::oscillator::{Oscillator, Waveform};
use crate::quantizer::Quantizer;
use crate::turing::TuringMachine;

/// Self-contained generative voice.
///
/// Combines rhythm generation (Euclidean), melody generation (Turing Machine + Quantizer),
/// and sound synthesis (Oscillator + Envelope).
pub struct Voice {
    /// Euclidean rhythm generator (decides when notes trigger)
    pub euclidean: EuclideanRhythm,

    /// Turing Machine melody generator (produces evolving pitch sequences)
    pub turing: TuringMachine,

    /// Scale quantizer (snaps Turing output to musical scales)
    pub quantizer: Quantizer,

    /// Primary oscillator (base pitch)
    pub osc1: Oscillator,

    /// Secondary oscillator (detuned +5 cents for thickness)
    pub osc2: Oscillator,

    /// Amplitude envelope (attack-decay articulation)
    pub envelope: ADEnvelope,
}

impl Voice {
    /// Private constructor (reused by bass() and lead()).
    fn new(
        euclidean_hits: usize,
        euclidean_length: usize,
        root_freq: f32,
        octave_range: f32,
        waveform: Waveform,
    ) -> Self {
        // Oscillator 1: base pitch
        let mut osc1 = Oscillator::new(waveform);
        osc1.set_level(1.0);
        osc1.set_frequency(root_freq);

        // Oscillator 2: detuned +5 cents for thickness
        let mut osc2 = Oscillator::new(waveform);
        osc2.set_level(1.0);
        osc2.set_detune_cents(5.0); // Detune persists across all frequency changes
        osc2.set_frequency(root_freq);

        Self {
            euclidean: EuclideanRhythm::new(euclidean_hits, euclidean_length),
            turing: TuringMachine::new(),
            quantizer: Quantizer::new(root_freq, octave_range),
            osc1,
            osc2,
            envelope: ADEnvelope::new(),
        }
    }

    /// Create bass voice (A1 root, saw wave, deep techno character).
    ///
    /// Root: 55 Hz (A1) - sub-bass territory for club systems
    /// Range: 1.5 octaves - tight melodic movement
    /// Pattern length: fixed, hits controlled by pot
    pub fn bass() -> Self {
        Self::new(
            EUCLIDEAN_INITIAL_HITS,
            BASS_EUCLIDEAN_LENGTH,
            BASS_ROOT_HZ,
            BASS_OCTAVE_RANGE,
            Waveform::Saw,
        )
    }

    /// Create lead voice (E4 root, square wave, bright melodic character).
    ///
    /// Root: 330 Hz (E4) - cuts through mix
    /// Range: 3.0 octaves - dramatic melodic jumps
    /// Initial rhythm overwritten by pots within ~20ms
    pub fn lead() -> Self {
        Self::new(
            EUCLIDEAN_INITIAL_HITS,
            EUCLIDEAN_INITIAL_LENGTH,
            LEAD_ROOT_HZ,
            LEAD_OCTAVE_RANGE,
            Waveform::Square,
        )
    }

    /// Process one clock step (rhythm + melody generation).
    ///
    /// Called when global clock ticks (~5-12 Hz for 16th notes at 80-180 BPM).
    ///
    /// Flow:
    /// 1. Euclidean rhythm decides if gate fires
    /// 2. If yes: Turing Machine generates new pitch value
    /// 3. Quantizer snaps to scale
    /// 4. Oscillator frequency updated (with internal smoothing)
    /// 5. Envelope triggered (attack phase)
    #[inline]
    pub fn step(&mut self) {
        // Check if Euclidean pattern fires a gate this step
        let gate = self.euclidean.tick();

        if gate {
            // Generate new pitch from Turing Machine (normalized -1.0 to 1.0)
            let turing_value = self.turing.tick();

            // Quantize to musical scale
            let pitch_hz = self.quantizer.quantize(turing_value);

            // Update both oscillators (osc2 auto-applies +5¢ detune)
            self.osc1.set_frequency(pitch_hz);
            self.osc2.set_frequency(pitch_hz);

            // Randomize envelope decay based on Turing output
            let variation = (turing_value + 1.0) * 0.5; // Map -1.0→1.0 to 0.0→1.0
            let decay_ms = ENVELOPE_DECAY_MIN_MS + variation * (ENVELOPE_DECAY_MAX_MS - ENVELOPE_DECAY_MIN_MS);
            self.envelope.set_decay_ms(decay_ms);

            // Trigger envelope attack
            self.envelope.trigger();
        }
    }

    /// Render one audio sample at 44.1 kHz.
    ///
    /// SUPER HOT PATH - called for every audio sample.
    ///
    /// # Returns
    /// Audio sample in range [-1.0, 1.0] modulated by envelope
    #[inline(always)]
    pub fn render(&mut self) -> f32 {
        // Mix both oscillators (osc2 is +5¢ detuned for thickness)
        let mix = (self.osc1.tick() + self.osc2.tick()) * 0.5;

        // Get envelope level (0.0 = silent, 1.0 = full volume)
        let env_level = self.envelope.tick();

        // Apply envelope to mixed oscillators
        mix * env_level
    }
}
