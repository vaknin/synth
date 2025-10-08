//! Engine module: manages oscillators, processes messages, renders audio.

use crate::config::{DEFAULT_FREQUENCY, MASTER_GAIN};
use crate::filter::MoogLadderFilter;
use crate::message::Message;
use crate::oscillator::{Oscillator, Waveform};
use crate::slew::Slew;
use crate::{AudioStream, ChannelReceiver, I16_MAX_F32};

/// Main synth engine managing direct oscillators (monophonic drone architecture).
pub struct Engine {
    /// Array of 4 oscillators [Osc1(Sine), Osc2(Saw), Osc3(Square), Sub(Sine)]
    oscillators: [Oscillator; 4],

    /// Cached reciprocal of level sum for fast normalization (multiply instead of divide)
    /// Smoothed at audio rate to prevent zipper noise when oscillator levels change
    level_sum_reciprocal: Slew,

    /// Message receiver from control tasks
    consumer: ChannelReceiver,

    /// Soft-start fade-in using Slew (0.0→1.0 over ~50ms, protects speakers/headphones)
    /// Initialized with current=0.0, target=1.0, uses existing SMOOTHING_COEFF
    soft_start_gain: Slew,

    /// Moog Ladder Filter (4-pole lowpass with resonance)
    filter: MoogLadderFilter,
}

impl Engine {
    /// Create new engine with 4 initialized oscillators and message receiver.
    ///
    /// # Arguments
    /// * `receiver` - Embassy channel receiver for control messages
    ///
    /// # Returns
    /// Engine with oscillators at DEFAULT_FREQUENCY, all silent (level=0.0).
    /// Pot messages arrive within ~20ms and Slew smooths to actual positions.
    pub fn new(receiver: ChannelReceiver) -> Self {
        Self {
            // Array of 4 oscillators: Sine, Saw, Square, Sub (-1 oct)
            // All start silent (level=0.0), pot messages arrive in ~20ms
            oscillators: [
                Oscillator::new(Waveform::Sine),
                Oscillator::new(Waveform::Saw),
                Oscillator::new(Waveform::Square),
                {
                    let mut sub = Oscillator::new(Waveform::Sine);
                    sub.set_frequency(DEFAULT_FREQUENCY / 2.0); // Sub at -1 octave
                    sub
                },
            ],
            // Normalization factor: starts at 1.0 (neutral), smoothed at audio rate to prevent zipper
            level_sum_reciprocal: Slew::new(1.0),
            consumer: receiver,
            // Soft-start gain: 0.0 → 1.0 over ~10ms (prevents speaker pop)
            soft_start_gain: {
                let mut gain = Slew::with_coeff(0.0, 0.99);
                gain.set_target(1.0);
                gain
            },
            filter: MoogLadderFilter::new(
                crate::config::FILTER_CUTOFF_DEFAULT,
                crate::config::FILTER_RESONANCE_DEFAULT,
            ),
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

    /// BUFFER RATE (~172 Hz): Drain control message queue and update parameter targets.
    ///
    /// Called once per buffer. Processes all pending messages from pot polling (50Hz).
    /// May process 0, 1, or multiple messages per buffer depending on pot activity.
    ///
    /// Messages set target values for parameters that smooth at their respective rates:
    /// - Oscillator freq/level → Audio-rate Slew (smoothed in render_sample())
    /// - Filter cutoff/resonance → Direct (marks dirty for coefficient recalc)
    /// - Normalization factor → Audio-rate Slew (smoothed in render_sample())
    #[inline]
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
                        let old_level = osc.level.target();

                        // Update oscillator level
                        osc.set_level(level);

                        // Incrementally update cached level sum reciprocal
                        // level_sum_new = level_sum_old - old_level + new_level
                        // reciprocal = 1.0 / level_sum_new
                        let old_sum = 1.0 / self.level_sum_reciprocal.target();
                        let new_sum = old_sum - old_level + level;

                        let new_reciprocal = if new_sum > 0.0 {
                            1.0 / new_sum
                        } else {
                            1.0 // Avoid division by zero (all silent)
                        };

                        // Smooth at audio rate to prevent zipper noise
                        self.level_sum_reciprocal.set_target(new_reciprocal);
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
            }
        }
    }

    /// BUFFER RATE (~172 Hz): Update cached parameters once per buffer.
    ///
    /// Called before processing each DMA buffer to recalculate coefficients
    /// that don't need audio-rate smoothing. Future: LFO/envelope modulation.
    #[inline]
    fn update_buffer_parameters(&mut self) {
        // Oscillator parameters: Currently no-op (freq/level smoothed at audio rate)
        // Future: LFO/envelope modulation will update Slew targets here
        for osc in &mut self.oscillators {
            osc.update_parameters();
        }

        // Filter coefficients: Only recalc if parameters changed (dirty flag optimization)
        self.filter.update_coefficients();
    }

    /// AUDIO RATE (44,100 Hz): Render one audio sample - SUPER HOT PATH!
    ///
    /// Called for every sample in the buffer. Performs all audio-rate smoothing:
    /// - Oscillator tick() smooths frequency and level via Slew
    /// - Normalization uses audio-rate Slew to prevent zipper noise
    /// - Filter processes with cached coefficients (updated at buffer rate)
    ///
    /// # Returns
    /// Filtered mono sample (before master gain and soft-start)
    #[inline]
    fn render_sample(&mut self) -> f32 {
        // Sum all oscillators (tick() smooths both freq & level at audio rate)
        let mix: f32 = self.oscillators.iter_mut().map(|osc| osc.tick()).sum();

        // Normalize by sum of oscillator levels (smoothed at audio rate to prevent zipper)
        let normalized = mix * self.level_sum_reciprocal.value();

        // Process through Moog Ladder Filter (lowpass only, cached coefficients)
        let filtered = self.filter.process(normalized);

        // Debug: Detect NaN/Inf and warn (indicates filter instability)
        // #[cfg(debug_assertions)]
        // if !filtered.is_finite() {
        //     log::error!("Filter output is NaN/Inf! cutoff={}, resonance={}",
        //                self.filter.cutoff, self.filter.resonance);
        //     return 0.0;
        // }

        filtered
    }

    /// Render audio samples into the DMA buffer.
    ///
    /// Processing rates:
    /// - BUFFER RATE (~172Hz): Update cached parameters, consume messages
    /// - AUDIO RATE (44.1kHz): Generate samples with audio-rate smoothing
    #[inline]
    async fn render(&mut self, audio_stream: &mut AudioStream) {
        // DIAGNOSTIC: Measure cycles spent in audio generation
        #[cfg(feature = "profile")]
        let start = embassy_time::Instant::now();

        #[cfg_attr(not(feature = "profile"), allow(unused_variables))]
        let bytes_written = audio_stream
            .push_with(|buffer| {
                if buffer.len() < 4 {
                    return 0;
                }

                // === BUFFER RATE: Update parameters once per buffer ===
                self.update_buffer_parameters();

                // Cache soft-start gain (settles in ~10ms, then no-op via settled flag)
                let fade_gain = self.soft_start_gain.value();

                // === AUDIO RATE: Process each stereo frame ===
                for chunk in buffer.chunks_exact_mut(4) {
                    // Render one sample (oscillator mix → normalize → filter)
                    let sample = self.render_sample();

                    // Apply soft-start fade, master gain, convert to i16
                    let output = (sample * fade_gain * MASTER_GAIN * I16_MAX_F32) as i16;
                    let bytes = output.to_le_bytes();

                    // Write stereo (duplicate mono to both channels)
                    // Direct assignment is faster than copy_from_slice for 4 bytes
                    chunk[0] = bytes[0];
                    chunk[1] = bytes[1];
                    chunk[2] = bytes[0];
                    chunk[3] = bytes[1];
                }

                buffer.len() - (buffer.len() % 4)
            })
            .await
            .unwrap_or(0);

        // DIAGNOSTIC: Log performance metrics
        #[cfg(feature = "profile")]
        {
            let elapsed = embassy_time::Instant::now() - start;
            let samples = bytes_written / 4;
            log::info!(
                "Rendered {} samples in {:?} ({} us/sample)",
                samples,
                elapsed,
                elapsed.as_micros() / samples.max(1) as u64
            );
        }
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
    #[inline]
    pub async fn stream_audio(&mut self, audio_stream: &mut AudioStream) {
        // Process all pending control messages
        self.consume_messages();

        self.render(audio_stream).await;
    }
}
