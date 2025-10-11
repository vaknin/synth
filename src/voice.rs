//! Generative voice combining rhythm, melody, and synthesis.
//!
//! Each voice generates its own patterns via Euclidean rhythms and Turing Machine,
//! then renders audio through oscillator and envelope.

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

    /// Create bass voice (C2, saw wave, E(3,8)).
    pub fn bass() -> Self {
        Self::new(3, 8, 65.0, 2.0, Waveform::Saw)
    }

    /// Create lead voice (C4, square wave, E(5,13)).
    pub fn lead() -> Self {
        Self::new(5, 13, 260.0, 3.0, Waveform::Square)
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
