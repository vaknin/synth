//! Arpeggiator module: clock-driven melody generation with hardcoded chord patterns.

use crate::config::{ARP_GATE_LENGTH, ARP_RATE_DEFAULT, ARP_SCALE_CHANGE_INTERVAL, BUFFER_RATE};
use crate::message::ArpMode;
use crate::noise::WhiteNoise;

// === Chord Patterns: Classic Arpeggio Building Blocks ===
//
// Short 4-5 note patterns that cycle like traditional arpeggios (C→E→G→C).
// Randomly switch between chord types every N notes for generative variety.

/// Minor triad: [root, m3, 5th, octave]
/// Classic sad/moody sound. Foundation of minor key music.
const CHORD_MINOR: [i8; 4] = [0, 3, 7, 12];

/// Major triad: [root, M3, 5th, octave]
/// Bright, happy, resolved. Most common chord in Western music.
const CHORD_MAJOR: [i8; 4] = [0, 4, 7, 12];

/// Minor 7th: [root, m3, 5th, m7, octave]
/// Jazzy, sophisticated minor sound. Adds color without dissonance.
const CHORD_MINOR_7: [i8; 5] = [0, 3, 7, 10, 12];

/// Dominant 7th: [root, M3, 5th, m7, octave]
/// Tension and resolution. Blues/jazz staple.
const CHORD_DOM_7: [i8; 5] = [0, 4, 7, 10, 12];

/// Sus4: [root, 4th, 5th, octave]
/// Ambiguous, floating, neither major nor minor. Very generative.
const CHORD_SUS4: [i8; 4] = [0, 5, 7, 12];

/// Augmented triad: [root, M3, #5, octave]
/// Symmetrical, dreamy, unresolved. Whole-tone flavor.
const CHORD_AUG: [i8; 4] = [0, 4, 8, 12];

/// Diminished 7th: [root, m3, dim5, dim7, octave]
/// Tense, dramatic, symmetrical. Horror movie vibes.
const CHORD_DIM7: [i8; 5] = [0, 3, 6, 9, 12];

/// All available chords for random selection.
const ALL_CHORDS: &[&[i8]] = &[
    &CHORD_MINOR,
    &CHORD_MAJOR,
    &CHORD_MINOR_7,
    &CHORD_DOM_7,
    &CHORD_SUS4,
    &CHORD_AUG,
    &CHORD_DIM7,
];

/// Semitone-to-frequency ratio lookup table.
/// Indexed by semitone offset (0-24), returns frequency multiplier.
/// Formula: ratio = 2^(semitones/12)
/// Extended to 24 semitones (2 octaves) to support ±12 semitone transpose.
/// Optimization: Precomputed to avoid powf() in hot path.
const SEMITONE_RATIOS: [f32; 25] = [
    1.000000, // 0 semitones
    1.059463, // 1 semitone
    1.122462, // 2 semitones
    1.189207, // 3 semitones
    1.259921, // 4 semitones
    1.334840, // 5 semitones
    1.414214, // 6 semitones
    1.498307, // 7 semitones
    1.587401, // 8 semitones
    1.681793, // 9 semitones
    1.781797, // 10 semitones
    1.887749, // 11 semitones
    2.000000, // 12 semitones (1 octave)
    2.118926, // 13 semitones
    2.244924, // 14 semitones
    2.378414, // 15 semitones
    2.519842, // 16 semitones
    2.669680, // 17 semitones
    2.828427, // 18 semitones
    2.996614, // 19 semitones
    3.174802, // 20 semitones
    3.363586, // 21 semitones
    3.563595, // 22 semitones
    3.775497, // 23 semitones
    4.000000, // 24 semitones (2 octaves)
];

/// Output from arpeggiator tick: frequencies for lead and bass oscillators.
pub struct ArpNote {
    /// Lead oscillator frequency (Osc2 Saw) - plays current arp note
    pub lead_freq: f32,

    /// Gate state (true = note on, false = note off/silence for articulation)
    pub gate_active: bool,
}

/// Arpeggiator state: clock, pattern tracking, root frequency.
pub struct Arpeggiator {
    /// Root frequency in Hz (arp lead melody pitch)
    root_freq: f32,

    /// Arp clock rate in Hz (1-20 Hz, controlled by Pot 4)
    rate_hz: f32,

    /// Phase accumulator for clock (0.0-1.0, wraps to trigger notes)
    phase: f32,

    /// Phase increment per buffer tick (rate_hz / BUFFER_RATE)
    phase_inc: f32,

    /// Current position in active scale pattern
    note_index: usize,

    /// Arpeggiator playback mode (Up/Down/UpDown/Random)
    mode: ArpMode,

    /// Root transpose in semitones (±12 semitones)
    transpose: i8,

    /// Direction for UpDown mode (true = ascending, false = descending)
    direction_up: bool,

    /// LFSR random number generator for Random mode and chord selection
    rng: WhiteNoise,

    /// Currently active chord index (into ALL_CHORDS)
    active_chord_index: usize,

    /// Note counter for chord change interval
    notes_played: usize,
}

impl Arpeggiator {
    /// Create new arpeggiator with default rate and Up mode.
    pub fn new() -> Self {
        let rate_hz = ARP_RATE_DEFAULT;
        Self {
            root_freq: 220.0, // A3 - will be overwritten by arp pitch pot
            rate_hz,
            phase: 0.0,
            phase_inc: rate_hz / BUFFER_RATE,
            note_index: 0,
            mode: ArpMode::Up,
            transpose: 0,
            direction_up: true,
            rng: WhiteNoise::new(),
            active_chord_index: 0, // Start with minor triad
            notes_played: 0,
        }
    }

    /// Set arpeggiator playback mode.
    pub fn set_mode(&mut self, mode: ArpMode) {
        self.mode = mode;
        self.note_index = 0; // Reset position when mode changes
        self.direction_up = true; // Reset direction for UpDown mode
    }

    /// Set root transpose in semitones (±12 semitones).
    pub fn set_transpose(&mut self, semitones: i8) {
        self.transpose = semitones.clamp(-12, 12);
    }

    /// Set arpeggiator clock rate (1-20 Hz).
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.rate_hz = rate_hz;
        self.phase_inc = rate_hz / BUFFER_RATE;
    }

    /// Set root frequency (arp lead melody pitch).
    /// Bass frequency is independent, fixed at BASS_FIXED_FREQ.
    pub fn set_root_freq(&mut self, freq: f32) {
        self.root_freq = freq;
    }

    /// Tick arpeggiator clock (called at buffer rate ~172 Hz).
    /// Returns current note frequencies for lead and bass.
    pub fn tick(&mut self) -> ArpNote {
        // Advance phase accumulator
        self.phase += self.phase_inc;

        // Wrap phase and advance to next note
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.advance_note();
        }

        // Get current semitone offset from active chord
        let active_chord = ALL_CHORDS[self.active_chord_index];
        let chord_semitones = active_chord[self.note_index];

        // Apply transpose (±12 semitones)
        let total_semitones = chord_semitones + self.transpose;

        // Convert semitones to frequency ratio (lookup table with bounds checking)
        let ratio = if total_semitones >= 0 {
            // Positive: use lookup table (0-24 range)
            let index = (total_semitones as usize).min(24);
            SEMITONE_RATIOS[index]
        } else {
            // Negative: use reciprocal (1 / ratio)
            let index = ((-total_semitones) as usize).min(24);
            1.0 / SEMITONE_RATIOS[index]
        };

        // Calculate lead frequency (root × ratio)
        let lead_freq = self.root_freq * ratio;

        // Gate: active for first 80% of note cycle, silent for last 20%
        let gate_active = self.phase < ARP_GATE_LENGTH;

        ArpNote {
            lead_freq,
            gate_active,
        }
    }

    /// Advance to next note based on current mode.
    /// Also handles chord switching: after completing a pattern cycle + reaching interval.
    #[inline]
    fn advance_note(&mut self) {
        let active_chord = ALL_CHORDS[self.active_chord_index];
        let chord_len = active_chord.len();

        match self.mode {
            ArpMode::Up => {
                self.note_index = (self.note_index + 1) % chord_len;
            }
            ArpMode::Down => {
                if self.note_index == 0 {
                    self.note_index = chord_len - 1;
                } else {
                    self.note_index -= 1;
                }
            }
            ArpMode::UpDown => {
                if self.direction_up {
                    self.note_index += 1;
                    if self.note_index >= chord_len - 1 {
                        self.direction_up = false;
                    }
                } else {
                    self.note_index -= 1;
                    if self.note_index == 0 {
                        self.direction_up = true;
                    }
                }
            }
            ArpMode::Random => {
                self.note_index = self.rng.next_usize(chord_len);
            }
        }

        // Count notes played
        self.notes_played += 1;

        // Chord switching: on pattern wrap + interval reached
        if self.note_index == 0 && self.notes_played >= ARP_SCALE_CHANGE_INTERVAL {
            // Pick a random chord (ensure different from current)
            let new_chord_index = loop {
                let candidate = self.rng.next_usize(ALL_CHORDS.len());
                if candidate != self.active_chord_index {
                    break candidate;
                }
            };
            self.active_chord_index = new_chord_index;
            self.notes_played = 0; // Reset counter
        }
    }
}
