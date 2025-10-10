//! Musical scale quantizer for generative synthesis.
//!
//! Maps continuous random voltages to discrete musical pitches within specified scales.
//! Uses precomputed lookup tables to eliminate expensive powf() calls.

use crate::config::GENERATIVE_OCTAVE_RANGE_MAX;
use micromath::F32Ext;

/// Precomputed semitone-to-frequency-ratio lookup table.
///
/// Maps semitone offsets to frequency ratios: ratio = 2^(semitone/12)
/// Eliminates expensive powf() call (100-200 cycles) with array lookup (2-3 cycles).
///
/// Array size derived from GENERATIVE_OCTAVE_RANGE_MAX:
/// - Range: ±(4.0 octaves × 12 semitones) = ±48 semitones
/// - Size: 48 + 1 + 48 = 97 elements (MAXIMUM RANGE)
const SEMITONE_RANGE: i32 = (GENERATIVE_OCTAVE_RANGE_MAX * 12.0) as i32; // 48
const SEMITONE_OFFSET: i32 = SEMITONE_RANGE; // Center offset for array indexing
const ARRAY_SIZE: usize = (SEMITONE_RANGE * 2 + 1) as usize; // 97

const SEMITONE_RATIOS: [f32; ARRAY_SIZE] = [
    // -48 to -37 semitones (4-3 octaves down)
    0.06250, 0.06622, 0.07015, 0.07433, 0.07875, 0.08343, 0.08839, 0.09364, 0.09921, 0.10511,
    0.11136, 0.11798,
    // -36 to -25 semitones (3-2 octaves down)
    0.12500, 0.13243, 0.14031, 0.14865, 0.15749, 0.16685, 0.17678, 0.18729, 0.19843, 0.21022,
    0.22272, 0.23597,
    // -24 to -13 semitones (2-1 octaves down)
    0.25000, 0.26487, 0.28062, 0.29730, 0.31498, 0.33371, 0.35355, 0.37458, 0.39685, 0.42045,
    0.44545, 0.47194,
    // -12 to -1 semitones (1 octave down)
    0.50000, 0.52973, 0.56123, 0.59460, 0.62996, 0.66742, 0.70711, 0.74915, 0.79370, 0.84090,
    0.89090, 0.94387,
    // 0 to 11 semitones (root to 1 octave up)
    1.00000, 1.05946, 1.12246, 1.18921, 1.25992, 1.33484, 1.41421, 1.49831, 1.58740, 1.68179,
    1.78180, 1.88775,
    // 12 to 23 semitones (1-2 octaves up)
    2.00000, 2.11893, 2.24492, 2.37841, 2.51984, 2.66968, 2.82843, 2.99661, 3.17480, 3.36359,
    3.56359, 3.77550,
    // 24 to 35 semitones (2-3 octaves up)
    4.00000, 4.23785, 4.48985, 4.75683, 5.03968, 5.33936, 5.65685, 5.99323, 6.34960, 6.72717,
    7.12719, 7.55099,
    // 36 to 48 semitones (3-4 octaves up)
    8.00000, 8.47570, 8.97970, 9.51366, 10.07937, 10.67872, 11.31371, 11.98646, 12.69921, 13.45434,
    14.25438, 15.10199, 16.00000,
];

/// 12 musical scales as semitone intervals from root.
/// Each array contains 12 semitone offsets covering ~2 octaves from the root note.
const SCALES: [[i8; 12]; 12] = [
    [0, 2, 4, 5, 7, 9, 11, 12, 14, 16, 17, 19],     // 0: Major (Ionian)
    [0, 2, 3, 5, 7, 8, 10, 12, 14, 15, 17, 19],     // 1: Natural Minor (Aeolian)
    [0, 2, 3, 5, 7, 8, 11, 12, 14, 15, 17, 19],     // 2: Harmonic Minor
    [0, 3, 5, 7, 10, 12, 15, 17, 19, 22, 24, 27],   // 3: Pentatonic Minor
    [0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24, 26],    // 4: Pentatonic Major
    [0, 3, 5, 6, 7, 10, 12, 15, 17, 18, 19, 22],    // 5: Blues Scale
    [0, 2, 3, 5, 7, 9, 10, 12, 14, 15, 17, 19],     // 6: Dorian
    [0, 1, 3, 5, 7, 8, 10, 12, 13, 15, 17, 19],     // 7: Phrygian
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],         // 8: Chromatic
    [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22],    // 9: Whole Tone
    [0, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17],      // 10: Diminished
    [0, 1, 5, 7, 8, 12, 13, 17, 19, 20, 24, 25],    // 11: Japanese Pentatonic
];

/// Scale quantizer for generative pitch generation.
///
/// Snaps continuous voltages to musical scales, ensuring notes are always in key.
pub struct Quantizer {
    /// Current scale index (0-11)
    scale_index: usize,
    /// Root note frequency in Hz
    root_freq: f32,
    /// Octave range for jumps (1-4 octaves)
    octave_range: f32,
}

impl Quantizer {
    /// Create new quantizer with root frequency and octave range.
    ///
    /// # Arguments
    /// * `root_freq` - Root note frequency in Hz (e.g., 60 Hz for bass, 440 Hz for lead)
    /// * `octave_range` - Octave range for note jumps (1.0 = stepwise, 4.0 = dramatic)
    pub fn new(root_freq: f32, octave_range: f32) -> Self {
        Self {
            scale_index: 1, // Default: Natural Minor
            root_freq,
            octave_range,
        }
    }

    /// Quantize input to nearest scale note.
    ///
    /// # Arguments
    /// * `input` - Normalized [-1.0, 1.0] from S&H
    ///
    /// # Returns
    /// Frequency in Hz snapped to current scale
    #[inline]
    pub fn quantize(&self, input: f32) -> f32 {
        // Map input to semitones (can be negative for notes below root)
        let semitones = input * self.octave_range * 12.0;
        let semitone_int = semitones.round() as i32;

        // Separate octave and scale degree to handle negative inputs correctly
        let octave = if semitone_int >= 0 {
            semitone_int / 12
        } else {
            (semitone_int - 11) / 12 // Floor division for negative numbers
        };
        let degree = semitone_int - (octave * 12);

        // Find closest scale note within one octave (0-11 semitones)
        let scale = &SCALES[self.scale_index];
        let mut closest = scale[0] as i32;
        let mut min_dist = (degree - closest).abs();
        for &note in scale.iter() {
            let note_i32 = note as i32;
            if note_i32 < 12 {
                // Only check first octave of scale
                let dist = (degree - note_i32).abs();
                if dist < min_dist {
                    min_dist = dist;
                    closest = note_i32;
                }
            }
        }

        // Calculate final semitone offset (handles negative octaves correctly)
        let final_semitone = octave * 12 + closest;

        // Lookup frequency ratio with bounds check
        let index = (final_semitone + SEMITONE_OFFSET).clamp(0, (ARRAY_SIZE - 1) as i32) as usize;
        self.root_freq * SEMITONE_RATIOS[index]
    }

    /// Set scale by normalized value (0.0-1.0 from pot).
    pub fn set_scale(&mut self, normalized: f32) {
        self.scale_index = ((normalized.clamp(0.0, 1.0) * 11.99).floor() as usize).min(11);
    }

    /// Set root frequency in Hz.
    pub fn set_root(&mut self, freq: f32) {
        self.root_freq = freq;
    }

    /// Set octave range (1-4 octaves).
    pub fn set_octave_range(&mut self, octaves: f32) {
        self.octave_range = octaves.clamp(
            crate::config::GENERATIVE_OCTAVE_RANGE_MIN,
            crate::config::GENERATIVE_OCTAVE_RANGE_MAX,
        );
    }
}
