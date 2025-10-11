//! Attack-Decay envelope generator for note articulation.
//!
//! Runs at audio rate (44.1 kHz) for smooth amplitude control.
//! Optimized for minimal CPU cost (~8 cycles/sample).

use crate::config::SAMPLE_RATE_F32;

/// Samples per millisecond (pre-computed to eliminate division).
const SAMPLES_PER_MS: f32 = SAMPLE_RATE_F32 / 1000.0;

/// Envelope stages for state machine.
#[derive(PartialEq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
}

/// Simple Attack-Decay envelope for note articulation.
///
/// Fixed fast attack (configurable in config.rs), fixed decay time.
pub struct ADEnvelope {
    /// Current envelope stage
    stage: EnvelopeStage,
    /// Current envelope level [0.0, 1.0]
    level: f32,
    /// Precalculated attack increment (rise rate per sample)
    attack_increment: f32,
    /// Decay increment (fall rate per sample)
    decay_increment: f32,
}

impl ADEnvelope {
    /// Create new envelope with fixed attack and decay times from config.
    pub fn new() -> Self {
        // Attack time from config (e.g., 5ms)
        let attack_ms = crate::config::ENVELOPE_ATTACK_MS;
        let attack_samples = attack_ms * SAMPLES_PER_MS;
        let attack_increment = 1.0 / attack_samples;

        // Decay time from config
        let decay_ms = crate::config::ENVELOPE_DEFAULT_DECAY_MS;
        let decay_samples = decay_ms * SAMPLES_PER_MS;
        let decay_increment = 1.0 / decay_samples;

        Self {
            stage: EnvelopeStage::Idle,
            level: 0.0,
            attack_increment,
            decay_increment,
        }
    }

    /// Trigger envelope (start attack phase).
    pub fn trigger(&mut self) {
        self.stage = EnvelopeStage::Attack;
        self.level = 0.0;
    }

    /// Set decay time dynamically (for envelope randomization).
    ///
    /// # Arguments
    /// * `decay_ms` - Decay time in milliseconds
    #[inline]
    pub fn set_decay_ms(&mut self, decay_ms: f32) {
        let decay_samples = decay_ms * SAMPLES_PER_MS;
        self.decay_increment = 1.0 / decay_samples;
    }

    /// Process one sample (AUDIO RATE 44.1 kHz).
    ///
    /// # Returns
    /// Current envelope level [0.0, 1.0]
    #[inline(always)]
    pub fn tick(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Attack => {
                self.level += self.attack_increment;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                self.level -= self.decay_increment;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
            EnvelopeStage::Idle => {
                self.level = 0.0;
            }
        }

        self.level
    }
}

impl Default for ADEnvelope {
    fn default() -> Self {
        Self::new()
    }
}
