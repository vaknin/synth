//! Chord palette for generative bass synthesis.
//!
//! Maps Turing Machine output to chord progressions with smooth harmonic transitions.
//! Uses mutation rate to control progression stability vs. exploration.

use micromath::F32Ext;

/// Maximum number of voices per chord (power chord to 7th chord).
pub const MAX_CHORD_VOICES: usize = 4;

/// Number of chords in the palette.
pub const CHORD_PALETTE_SIZE: usize = 16;

/// Chord definition as semitone intervals from root.
/// Up to 4 voices: [root_offset, voice2, voice3, voice4]
/// Unused voices are marked with i8::MIN.
pub type ChordIntervals = [i8; MAX_CHORD_VOICES];

/// 16-chord palette optimized for evolving bass lines.
///
/// Arranged for smooth harmonic motion when Turing mutation gradually shifts indices.
/// Progression covers common tonal movements: I → iv → V → variants.
///
/// Palette structure:
/// - Indices 0-3: Root chord variations (power, triad, sus, 7th)
/// - Indices 4-7: Subdominant movement (iv, IV)
/// - Indices 8-11: Dominant movement (V, v)
/// - Indices 12-15: Color chords (bVII, vi, III)
///
/// Example in C: [C, F, G, Bb, Am, Eb, ...]
const CHORD_PALETTE: [ChordIntervals; CHORD_PALETTE_SIZE] = [
    // --- ROOT (I) variations ---
    [0, 7, i8::MIN, i8::MIN],           // 0: Power chord (root + 5th)
    [0, 3, 7, i8::MIN],                 // 1: Minor triad (root + m3 + 5th)
    [0, 5, 7, i8::MIN],                 // 2: Sus4 (root + 4th + 5th)
    [0, 3, 7, 10],                      // 3: Minor 7th (root + m3 + 5th + m7)

    // --- SUBDOMINANT (iv/IV) ---
    [5, 12, i8::MIN, i8::MIN],          // 4: iv power chord (4th + octave above)
    [5, 8, 12, i8::MIN],                // 5: iv minor triad
    [5, 9, 12, i8::MIN],                // 6: IV major triad
    [5, 8, 12, 15],                     // 7: iv minor 7th

    // --- DOMINANT (V/v) ---
    [7, 14, i8::MIN, i8::MIN],          // 8: V power chord (5th + octave above)
    [7, 11, 14, i8::MIN],               // 9: V major triad
    [7, 10, 14, i8::MIN],               // 10: v minor triad
    [7, 11, 14, 17],                    // 11: V major 7th

    // --- COLOR CHORDS ---
    [-2, 5, i8::MIN, i8::MIN],          // 12: bVII power chord (flat 7th)
    [-2, 2, 5, i8::MIN],                // 13: bVII major triad
    [-3, 4, i8::MIN, i8::MIN],          // 14: vi power chord (relative minor)
    [-3, 0, 4, i8::MIN],                // 15: vi minor triad
];

/// Chord selector for generative bass synthesis.
///
/// Maps continuous Turing Machine output to discrete chord selections.
pub struct ChordSelector {
    /// Root note frequency in Hz
    root_freq: f32,
}

impl ChordSelector {
    /// Create new chord selector with root frequency.
    ///
    /// # Arguments
    /// * `root_freq` - Root note frequency in Hz (e.g., 65 Hz for C2 bass)
    pub fn new(root_freq: f32) -> Self {
        Self {
            root_freq,
        }
    }

    /// Select chord from palette and return frequencies for all voices.
    ///
    /// # Arguments
    /// * `input` - Normalized [-1.0, 1.0] from Turing Machine
    ///
    /// # Returns
    /// Array of up to 4 frequencies in Hz. Unused voices are 0.0.
    #[inline]
    pub fn select(&self, input: f32) -> [f32; MAX_CHORD_VOICES] {
        // Map input [-1.0, 1.0] to chord index [0, 15]
        let normalized = (input + 1.0) * 0.5; // Convert to [0.0, 1.0]
        let index = (normalized * (CHORD_PALETTE_SIZE - 1) as f32).round() as usize;
        let index = index.clamp(0, CHORD_PALETTE_SIZE - 1);

        // Get chord intervals
        let intervals = &CHORD_PALETTE[index];

        // Convert semitone intervals to frequencies
        let mut freqs = [0.0; MAX_CHORD_VOICES];
        for (i, &semitone) in intervals.iter().enumerate() {
            if semitone != i8::MIN {
                freqs[i] = self.semitone_to_freq(semitone);
            }
        }

        freqs
    }

    /// Convert semitone offset to frequency using precomputed ratios.
    ///
    /// Reuses the same SEMITONE_RATIOS lookup table from quantizer.rs for consistency.
    #[inline]
    fn semitone_to_freq(&self, semitone: i8) -> f32 {
        use crate::quantizer::semitone_to_ratio;
        self.root_freq * semitone_to_ratio(semitone as i32)
    }

    /// Set root frequency in Hz.
    pub fn set_root(&mut self, freq: f32) {
        self.root_freq = freq;
    }
}
