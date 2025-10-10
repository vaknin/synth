//! Euclidean rhythm generator using Bjorklund's algorithm.
//!
//! Distributes k hits across n steps as evenly as possible, creating
//! mathematically musical polyrhythms (tresillo, son clave, bossa nova, etc.).

const MAX_STEPS: usize = 32;

/// Euclidean rhythm generator.
///
/// Generates gate patterns using Bjorklund's algorithm for even distribution.
/// Patterns are precomputed when parameters change (not per-step).
pub struct EuclideanRhythm {
    /// Precomputed pattern buffer (true = hit, false = rest)
    pattern: [bool; MAX_STEPS],
    /// Current pattern length (n steps)
    pub length: usize,
    /// Number of hits (k)
    pub hits: usize,
    /// Current step position [0, length)
    position: usize,
}

impl EuclideanRhythm {
    /// Create new Euclidean rhythm generator.
    ///
    /// # Arguments
    /// * `hits` - Number of hits (k) to distribute
    /// * `length` - Total pattern length in steps (n)
    pub fn new(hits: usize, length: usize) -> Self {
        let mut rhythm = Self {
            pattern: [false; MAX_STEPS],
            length: length.clamp(1, MAX_STEPS),
            hits: hits.clamp(0, length.clamp(1, MAX_STEPS)),
            position: 0,
        };
        rhythm.calculate_pattern();
        rhythm
    }

    /// Get next gate state and advance step.
    ///
    /// Call this once per clock tick (tempo-based, typically ~5-12 Hz for 16th notes).
    /// NOT called at audio rate or buffer rate - this is event-driven rhythm generation.
    ///
    /// # Returns
    /// true if this step is a hit (trigger gate/envelope), false for rest
    #[inline]
    pub fn tick(&mut self) -> bool {
        let gate = self.pattern[self.position];
        self.position = (self.position + 1) % self.length;
        gate
    }

    /// Reset position to start of pattern.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Set number of hits (k) and recalculate pattern.
    pub fn set_hits(&mut self, hits: usize) {
        let new_hits = hits.clamp(0, self.length);
        if new_hits != self.hits {
            self.hits = new_hits;
            self.calculate_pattern();
        }
    }

    /// Set pattern length (n) and recalculate pattern.
    pub fn set_length(&mut self, length: usize) {
        let new_length = length.clamp(1, MAX_STEPS);
        if new_length != self.length {
            self.length = new_length;
            self.hits = self.hits.min(new_length);
            self.calculate_pattern();
            self.position %= self.length;
        }
    }

    /// Calculate Euclidean pattern using bit-packed Bjorklund's algorithm.
    ///
    /// Distributes k hits across n steps as evenly as possible.
    /// Algorithm complexity: O(log n) iterations, O(n) total time.
    ///
    /// Memory-optimized: Uses bit-packing (320 bytes stack vs 4,608 bytes original).
    /// Since MAX_STEPS=32 fits in u32, each pattern is stored as bits instead of bool arrays.
    fn calculate_pattern(&mut self) {
        self.pattern = [false; MAX_STEPS];

        if self.hits == 0 {
            return;
        }
        if self.hits >= self.length {
            for i in 0..self.length {
                self.pattern[i] = true;
            }
            return;
        }

        // Bjorklund's algorithm: maintain two lists of groups (front/back)
        // and iteratively pair them
        //
        // OPTIMIZATION: Use bit-packing instead of 2D bool arrays.
        // Each group's pattern fits in u32 (MAX_STEPS=32), dramatically reducing stack usage.

        // Bit-packed storage: each u32 holds a pattern up to 32 bits
        let mut front: [u32; MAX_STEPS] = [0; MAX_STEPS];       // 128 bytes (vs 1,024)
        let mut front_lengths: [u8; MAX_STEPS] = [0; MAX_STEPS]; // 32 bytes (vs 128)
        let mut front_count = self.hits;

        let mut back: [u32; MAX_STEPS] = [0; MAX_STEPS];        // 128 bytes (vs 1,024)
        let mut back_lengths: [u8; MAX_STEPS] = [0; MAX_STEPS];  // 32 bytes (vs 128)
        let mut back_count = self.length - self.hits;

        // Initialize: front = k groups of [1], back = (n-k) groups of [0]
        for i in 0..front_count {
            front[i] = 1;           // Single '1' bit
            front_lengths[i] = 1;
        }
        for i in 0..back_count {
            back[i] = 0;            // Single '0' bit
            back_lengths[i] = 1;
        }

        // Bjorklund pairing loop
        while back_count > 1 {
            let pairs = front_count.min(back_count);

            // Pair front[i] with back[i] by concatenating bits in-place
            for i in 0..pairs {
                let front_len = front_lengths[i];
                let back_bits = back[i];
                let back_len = back_lengths[i];

                // Concatenate: front[i] + back[i]
                // Shift back bits left by front_len, then OR with front
                front[i] |= back_bits << front_len;
                front_lengths[i] = front_len + back_len;
            }

            // Move unpaired groups to back (in-place, no large array copies)
            let mut new_back_count = 0;

            // Move unpaired fronts
            for i in pairs..front_count {
                back[new_back_count] = front[i];
                back_lengths[new_back_count] = front_lengths[i];
                new_back_count += 1;
            }

            // Keep unpaired backs
            for i in pairs..back_count {
                back[new_back_count] = back[i];
                back_lengths[new_back_count] = back_lengths[i];
                new_back_count += 1;
            }

            front_count = pairs;
            back_count = new_back_count;
        }

        // Flatten bit-packed patterns into final boolean array
        let mut index = 0;

        // Extract bits from front groups
        for i in 0..front_count {
            let pattern_bits = front[i];
            let length = front_lengths[i];

            for bit_pos in 0..length {
                if index < self.length {
                    self.pattern[index] = (pattern_bits >> bit_pos) & 1 == 1;
                    index += 1;
                }
            }
        }

        // Extract bits from back groups
        for i in 0..back_count {
            let pattern_bits = back[i];
            let length = back_lengths[i];

            for bit_pos in 0..length {
                if index < self.length {
                    self.pattern[index] = (pattern_bits >> bit_pos) & 1 == 1;
                    index += 1;
                }
            }
        }
    }
}
