//! Generative voice combining rhythm, melody, and synthesis.
//!
//! Each voice generates its own patterns via Euclidean rhythms and Turing Machine,
//! then renders audio through oscillator and envelope.

use crate::config::{
    BASS_ANCHOR_INTERVAL, BASS_EUCLIDEAN_LENGTH, BASS_MUTATION_RATE, BASS_OCTAVE_RANGE,
    BASS_ROOT_HZ, BASS_TURING_LOOP_LENGTH, ENVELOPE_DECAY_MAX_MS, ENVELOPE_DECAY_MIN_MS,
    EUCLIDEAN_INITIAL_HITS, LEAD_ANCHOR_INTERVAL, LEAD_EUCLIDEAN_LENGTH, LEAD_MUTATION_RATE,
    LEAD_OCTAVE_RANGE, LEAD_ROOT_HZ, LEAD_TURING_LOOP_LENGTH,
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

    /// Step counter for harmonic anchoring
    step_count: usize,

    /// Anchor interval (return to root every N steps)
    anchor_interval: usize,

    /// Root frequency for harmonic anchoring
    root_freq: f32,
}

impl Voice {
    /// Private constructor (reused by bass() and lead()).
    fn new(
        euclidean_hits: usize,
        euclidean_length: usize,
        root_freq: f32,
        octave_range: f32,
        waveform: Waveform,
        turing_loop_length: usize,
        mutation_rate: f32,
        anchor_interval: usize,
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

        // Initialize Turing Machine with fixed loop length and mutation rate
        let mut turing = TuringMachine::new();
        turing.set_loop_length(turing_loop_length);
        turing.set_mutation_rate(mutation_rate);

        Self {
            euclidean: EuclideanRhythm::new(euclidean_hits, euclidean_length),
            turing,
            quantizer: Quantizer::new(root_freq, octave_range),
            osc1,
            osc2,
            envelope: ADEnvelope::new(),
            step_count: 0,
            anchor_interval,
            root_freq,
        }
    }

    /// Create bass voice (C2 root, saw wave, deep techno character).
    ///
    /// Root: 65 Hz (C2) - sub-bass territory for club systems
    /// Range: 1.5 octaves - tight melodic movement
    /// Turing: 10-bit loop, 4% mutation - stable, grounding phrases
    /// Anchoring: Root note every 8 steps - solid tonal foundation
    pub fn bass() -> Self {
        Self::new(
            EUCLIDEAN_INITIAL_HITS,
            BASS_EUCLIDEAN_LENGTH,
            BASS_ROOT_HZ,
            BASS_OCTAVE_RANGE,
            Waveform::Saw,
            BASS_TURING_LOOP_LENGTH,
            BASS_MUTATION_RATE,
            BASS_ANCHOR_INTERVAL,
        )
    }

    /// Create lead voice (C4 root, square wave, bright melodic character).
    ///
    /// Root: 260 Hz (C4) - 2 octaves above bass for harmonic alignment
    /// Range: 3.0 octaves - dramatic melodic jumps
    /// Turing: 20-bit loop, 28% mutation - evolving, melodic exploration
    /// Anchoring: Root note every 16 steps - resolution points for catchiness
    pub fn lead() -> Self {
        Self::new(
            EUCLIDEAN_INITIAL_HITS,
            LEAD_EUCLIDEAN_LENGTH,
            LEAD_ROOT_HZ,
            LEAD_OCTAVE_RANGE,
            Waveform::Square,
            LEAD_TURING_LOOP_LENGTH,
            LEAD_MUTATION_RATE,
            LEAD_ANCHOR_INTERVAL,
        )
    }

    /// Process one clock step (rhythm + melody generation).
    ///
    /// Called when global clock ticks (~5-12 Hz for 16th notes at 80-180 BPM).
    ///
    /// Flow:
    /// 1. Check harmonic anchoring (every N steps → root note)
    /// 2. Euclidean rhythm decides if gate fires
    /// 3. If yes: Generate pitch (anchor → root, else → Turing + quantize)
    /// 4. Oscillator frequency updated (with internal smoothing)
    /// 5. Envelope triggered (attack phase)
    #[inline]
    pub fn step(&mut self) {
        // Check harmonic anchoring FIRST (happens every N steps regardless of gate)
        let should_anchor = self.step_count % self.anchor_interval == 0;
        self.step_count = self.step_count.wrapping_add(1);

        // Check if Euclidean pattern fires a gate this step
        let gate = self.euclidean.tick();

        if gate {
            // Determine pitch: anchor to root or generate from Turing
            let pitch_hz = if should_anchor {
                // Force root note for harmonic anchoring (creates "catchy" resolution)
                self.root_freq
            } else {
                // Normal generative melody: Turing Machine → Quantizer
                let turing_value = self.turing.tick();
                self.quantizer.quantize(turing_value)
            };

            // Update both oscillators (osc2 auto-applies +5¢ detune)
            self.osc1.set_frequency(pitch_hz);
            self.osc2.set_frequency(pitch_hz);

            // Randomize envelope decay based on Turing output
            let turing_value = self.turing.tick(); // Get fresh value for envelope variation
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
