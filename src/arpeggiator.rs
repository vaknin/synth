//! Arpeggiator module: clock-driven melody generation with hardcoded chord patterns.

use crate::config::{ARP_GATE_LENGTH, ARP_RATE_DEFAULT, BUFFER_RATE};
use crate::message::ArpMode;
use crate::noise::WhiteNoise;

/// Chord pattern: C major [root, maj3rd, 5th, octave] in semitone offsets.
/// Used with "up" mode to cycle through notes: C → E → G → C...
/// Bass triggers when pattern wraps (note_index == 0), creating sparse rhythm.
const CHORD_MAJOR: [i8; 4] = [0, 4, 7, 12];

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

    /// Whether bass should play this note (true every 4th note)
    pub bass_active: bool,

    /// Gate state (true = note on, false = note off/silence for articulation)
    pub gate_active: bool,
}

/// Arpeggiator state: clock, pattern tracking, root frequency.
pub struct Arpeggiator {
    /// Root frequency in Hz (from master pitch)
    root_freq: f32,

    /// Bass frequency cached at -1 octave (public field per CLAUDE.md style)
    pub bass_freq: f32,

    /// Arp clock rate in Hz (1-20 Hz, controlled by Pot 4)
    rate_hz: f32,

    /// Phase accumulator for clock (0.0-1.0, wraps to trigger notes)
    phase: f32,

    /// Phase increment per buffer tick (rate_hz / BUFFER_RATE)
    phase_inc: f32,

    /// Current position in chord pattern (0-3 for C major)
    note_index: usize,

    /// Arpeggiator playback mode (Up/Down/UpDown/Random)
    mode: ArpMode,

    /// Root transpose in semitones (±12 semitones)
    transpose: i8,

    /// Direction for UpDown mode (true = ascending, false = descending)
    direction_up: bool,

    /// LFSR random number generator for Random mode
    rng: WhiteNoise,
}

impl Arpeggiator {
    /// Create new arpeggiator with default rate and Up mode.
    pub fn new() -> Self {
        let rate_hz = ARP_RATE_DEFAULT;
        let root_freq = 220.0; // A3 - will be overwritten by master pitch
        Self {
            root_freq,
            bass_freq: root_freq / 2.0, // -1 octave
            rate_hz,
            phase: 0.0,
            phase_inc: rate_hz / BUFFER_RATE,
            note_index: 0,
            mode: ArpMode::Up,
            transpose: 0,
            direction_up: true,
            rng: WhiteNoise::new(),
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

    /// Set root frequency (from master pitch).
    pub fn set_root_freq(&mut self, freq: f32) {
        self.root_freq = freq;
        self.bass_freq = freq / 2.0; // Cache bass at -1 octave
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

        // Get current semitone offset from chord pattern
        let chord_semitones = CHORD_MAJOR[self.note_index];

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

        // Bass active only on beat 1 (note_index == 0, every 4th note)
        let bass_active = self.note_index == 0;

        // Gate: active for first 80% of note cycle, silent for last 20%
        let gate_active = self.phase < ARP_GATE_LENGTH;

        ArpNote {
            lead_freq,
            bass_active,
            gate_active,
        }
    }

    /// Advance to next note based on current mode.
    #[inline]
    fn advance_note(&mut self) {
        match self.mode {
            ArpMode::Up => {
                self.note_index = (self.note_index + 1) % CHORD_MAJOR.len();
            }
            ArpMode::Down => {
                if self.note_index == 0 {
                    self.note_index = CHORD_MAJOR.len() - 1;
                } else {
                    self.note_index -= 1;
                }
            }
            ArpMode::UpDown => {
                if self.direction_up {
                    self.note_index += 1;
                    if self.note_index >= CHORD_MAJOR.len() - 1 {
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
                self.note_index = self.rng.next_usize(CHORD_MAJOR.len());
            }
        }
    }
}
