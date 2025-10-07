//! Engine module: manages oscillators, processes messages, renders audio.

use crate::config::{DEFAULT_FREQUENCY, MASTER_GAIN};
use crate::filter::StateVariableFilter;
use crate::message::Message;
use crate::oscillator::{Oscillator, Waveform};
use crate::slew::Slew;
use crate::{AudioStream, ChannelReceiver, I16_MAX_F32};

/// Main synth engine managing direct oscillators (monophonic drone architecture).
pub struct Engine {
    /// Array of 4 oscillators [Osc1(Sine), Osc2(Saw), Osc3(Square), Sub(Sine)]
    oscillators: [Oscillator; 4],

    /// Cached reciprocal of level sum for fast normalization (multiply instead of divide)
    /// Updated whenever any oscillator level changes
    level_sum_reciprocal: f32,

    /// Message receiver from control tasks
    consumer: ChannelReceiver,

    /// Soft-start fade-in using Slew (0.0→1.0 over ~50ms, protects speakers/headphones)
    /// Initialized with current=0.0, target=1.0, uses existing SMOOTHING_COEFF
    soft_start_gain: Slew,

    /// State Variable Filter (LP/HP/BP outputs)
    filter: StateVariableFilter,
}

impl Engine {
    /// Create new engine with 4 initialized oscillators and message receiver.
    ///
    /// # Arguments
    /// * `receiver` - Embassy channel receiver for control messages
    ///
    /// # Returns
    /// Engine with oscillators at DEFAULT_FREQUENCY:
    /// - Osc1: Sine wave at DEFAULT_VOICE_VOLUME (active by default)
    /// - Osc2: Sawtooth wave at 0.0 (silent)
    /// - Osc3: Square wave at 0.0 (silent)
    /// - Sub: Sine wave at -1 octave at 0.0 (silent)
    pub fn new(receiver: ChannelReceiver) -> Self {
        use crate::config::DEFAULT_VOICE_VOLUME;

        let mut osc1 = Oscillator::new(Waveform::Sine);
        let mut osc2 = Oscillator::new(Waveform::Saw);
        let mut osc3 = Oscillator::new(Waveform::Square);
        let mut sub = Oscillator::new(Waveform::Sine);
        sub.set_frequency(DEFAULT_FREQUENCY / 2.0);  // Sub oscillator at -1 octave

        // Only Osc1 active by default
        osc1.set_level(DEFAULT_VOICE_VOLUME);
        osc2.set_level(0.0);
        osc3.set_level(0.0);
        sub.set_level(0.0);

        // Initial level sum is just Osc1
        let level_sum = DEFAULT_VOICE_VOLUME;
        let level_sum_reciprocal = 1.0 / level_sum;

        // Soft-start gain: starts at 0.0, ramps to 1.0 over ~10ms
        // Uses fast coefficient (0.99) for quick but smooth startup, preventing speaker pop
        let mut soft_start_gain = Slew::with_coeff(0.0, 0.99);
        soft_start_gain.set_target(1.0);

        // Initialize filter with config defaults
        let filter = StateVariableFilter::new(
            crate::config::FILTER_CUTOFF_DEFAULT,
            crate::config::FILTER_RESONANCE_DEFAULT,
        );

        Self {
            oscillators: [osc1, osc2, osc3, sub],
            level_sum_reciprocal,
            consumer: receiver,
            soft_start_gain,
            filter,
        }
    }

    /// Set master pitch for all oscillators (respects per-oscillator detune).
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz (sub-oscillator plays at half frequency)
    fn set_master_pitch(&mut self, frequency: f32) {
        // Update Osc1, Osc2, Osc3 with base frequency
        for osc in &mut self.oscillators[0..3] {
            osc.set_frequency(frequency);
        }

        // Sub-oscillator plays at -1 octave (half frequency)
        self.oscillators[3].set_frequency(frequency / 2.0);
    }

    /// Process all pending control messages from the queue.
    fn consume_messages(&mut self) {
        while let Ok(msg) = self.consumer.try_receive() {
            match msg {
                // === Phase 1: Oscillator Controls ===
                Message::SetMasterPitch(freq) => {
                    self.set_master_pitch(freq);
                }

                Message::SetOscLevel(osc_id, level) => {
                    if let Some(osc) = self.oscillators.get_mut(osc_id as usize) {
                        // Read old level for incremental sum update
                        let old_level = osc.level.target;

                        // Update oscillator level
                        osc.set_level(level);

                        // Incrementally update cached level sum reciprocal
                        // level_sum_new = level_sum_old - old_level + new_level
                        // reciprocal = 1.0 / level_sum_new
                        let old_sum = 1.0 / self.level_sum_reciprocal;
                        let new_sum = old_sum - old_level + level;

                        self.level_sum_reciprocal = if new_sum > 0.0 {
                            1.0 / new_sum
                        } else {
                            1.0  // Avoid division by zero (all silent)
                        };
                    }
                }

                Message::SetOscDetune(osc_id, cents) => {
                    if let Some(osc) = self.oscillators.get_mut(osc_id as usize) {
                        osc.set_detune_cents(cents);
                    }
                }

                // === Phase 2: Filter Controls ===
                Message::SetCutoff(freq) => {
                    self.filter.set_cutoff(freq);
                }

                Message::SetResonance(q) => {
                    self.filter.set_resonance(q);
                }

                Message::SetFilterLevel(_filter_type, _level) => {
                    // TODO: Implement filter blending in Phase 3
                }
            }
        }
    }

    /// Render audio samples into the DMA buffer.
    ///
    /// Sums all 4 oscillators (each applies its own level) and normalizes
    /// by cached level sum reciprocal for dynamic loudness preservation.
    async fn render(&mut self, audio_stream: &mut AudioStream) {
        audio_stream.push_with(|buffer| {
            if buffer.len() < 4 {
                return 0;
            }

            // Generate audio for each stereo frame
            for chunk in buffer.chunks_exact_mut(4) {
                // Sum all oscillators (level is applied internally by tick())
                let mix: f32 = self.oscillators.iter_mut()
                    .map(|osc| osc.tick())
                    .sum();

                // Normalize by cached level sum reciprocal (fast multiply, not divide)
                let normalized = mix * self.level_sum_reciprocal;

                // Process through State Variable Filter (LP/HP/BP outputs)
                let (lp, _hp, _bp) = self.filter.process(normalized);

                // Use lowpass output only (Phase 2: LP-only)
                let filtered = lp;

                // DIAGNOSTIC: Bypass soft-start fade-in
                // let fade_gain = self.soft_start_gain.value();
                let fade_gain = 1.0;

                // Apply soft-start, master gain, and convert to i16
                let sample = (filtered * fade_gain * MASTER_GAIN * I16_MAX_F32) as i16;
                let bytes = sample.to_le_bytes();

                // Write stereo (duplicate mono to both channels)
                // Direct assignment is faster than copy_from_slice for 4 bytes
                chunk[0] = bytes[0];
                chunk[1] = bytes[1];
                chunk[2] = bytes[0];
                chunk[3] = bytes[1];
            }

            buffer.len() - (buffer.len() % 4)
        }).await.ok();
    }

    /// Render audio into provided buffer.
    ///
    /// Processes all pending control messages, generates audio samples,
    /// converts to i16 stereo format, and writes to buffer.
    ///
    /// # Arguments
    /// * `buffer` - Output buffer for i16 LE stereo audio (must be multiple of 4 bytes)
    ///
    /// # Returns
    /// Number of bytes written to buffer (will be multiple of 4)
    /// #[inline] should we inline it? it's the literal hot-path
    pub async fn stream_audio(&mut self, audio_stream:&mut AudioStream) {

        // Process all pending control messages
        self.consume_messages();

        self.render(audio_stream).await;
        
    }
}
