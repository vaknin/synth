//! Engine module: manages oscillators, processes messages, renders audio.

use crate::arpeggiator::Arpeggiator;
use crate::config::MASTER_GAIN;
use crate::envelope::ADEnvelope;
use crate::filter::MoogLadderFilter;
use crate::lfo::LFO;
use crate::message::Message;
use crate::oscillator::{Oscillator, Waveform};
use crate::slew::Slew;
use crate::{AudioStream, ChannelReceiver, I16_MAX_F32};

/// Main synth engine managing direct oscillators (monophonic architecture).
pub struct Engine {
    /// Array of 2 oscillators [Lead(Saw), Bass(Sine)]
    oscillators: [Oscillator; 2],

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

    /// Filter cutoff LFO
    filter_lfo: LFO,

    /// Base filter cutoff from pot (before LFO modulation)
    base_cutoff: f32,

    /// LFO modulation range in Hz (depth pot → Hz range)
    lfo_cutoff_range: f32,

    /// Arpeggiator (clock-driven melody generation)
    arpeggiator: Arpeggiator,

    /// Bass drift LFO (Lyra-8 style slow pitch modulation)
    bass_drift_lfo: LFO,

    /// Lead oscillator envelope (Attack-Decay for note articulation)
    lead_envelope: ADEnvelope,

    /// Previous arp gate state (for detecting note triggers)
    prev_gate_active: bool,

    /// Profiling: accumulated statistics (only compiled with 'profile' feature)
    #[cfg(feature = "profile")]
    profile_stats: ProfileStats,
}

#[cfg(feature = "profile")]
struct ProfileStats {
    buffer_count: u32,
    total_micros: u64,
    min_micros_per_sample: u64,
    max_micros_per_sample: u64,
}

impl Engine {
    /// Create new engine with 4 initialized oscillators and message receiver.
    ///
    /// # Arguments
    /// * `receiver` - Embassy channel receiver for control messages
    ///
    /// # Returns
    /// Engine with oscillators at FREQUENCY_MIN, all silent (level=0.0).
    /// Pot messages arrive within ~20ms and Slew smooths to actual positions.
    pub fn new(receiver: ChannelReceiver) -> Self {
        let mut oscillators = [
            Oscillator::new(Waveform::Saw),  // Lead (arp-controlled)
            Oscillator::new(Waveform::Sine), // Bass (fixed drone)
        ];

        // Bass: Set fixed frequency and level once, never change them
        oscillators[1].set_frequency(crate::config::BASS_FIXED_FREQ);
        oscillators[1].set_level(crate::config::BASS_LEVEL);

        Self {
            oscillators,
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
                crate::config::FILTER_CUTOFF_MIN,
                crate::config::FILTER_RESONANCE_MIN,
            ),
            filter_lfo: LFO::new(
                crate::config::LFO_FREQUENCY_DEFAULT,
                crate::config::LFO_DEPTH_DEFAULT,
            ),
            base_cutoff: crate::config::FILTER_CUTOFF_MIN,
            lfo_cutoff_range: 0.0,
            arpeggiator: Arpeggiator::new(),
            bass_drift_lfo: LFO::new(
                crate::config::BASS_DRIFT_LFO_RATE,
                1.0, // Full depth - we'll scale by BASS_DRIFT_CENTS in modulation
            ),
            lead_envelope: ADEnvelope::new(),
            prev_gate_active: false,
            #[cfg(feature = "profile")]
            profile_stats: ProfileStats {
                buffer_count: 0,
                total_micros: 0,
                min_micros_per_sample: u64::MAX,
                max_micros_per_sample: 0,
            },
        }
    }

    /// Set arpeggiator root pitch (lead melody frequency).
    /// Bass drone remains independent at fixed frequency.
    ///
    /// # Arguments
    /// * `frequency` - Root frequency in Hz (arpeggiator transposes from this)
    fn set_arp_pitch(&mut self, frequency: f32) {
        self.arpeggiator.set_root_freq(frequency);
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
                Message::SetArpPitch(freq) => {
                    self.set_arp_pitch(freq);
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

                Message::SetCutoff(freq) => {
                    self.base_cutoff = freq;
                }

                Message::SetResonance(q) => {
                    self.filter.set_resonance(q);
                }

                Message::SetFilterLfoRate(rate) => {
                    self.filter_lfo.set_frequency(rate);
                }

                Message::SetFilterLfoDepth(depth) => {
                    self.filter_lfo.depth = depth;
                    // Convert depth (0.0-1.0) to Hz range (±2 octaves = 4× multiplier)
                    self.lfo_cutoff_range = self.base_cutoff * 3.0 * depth;
                }

                Message::SetArpRate(rate) => {
                    self.arpeggiator.set_rate(rate);
                }

                Message::SetArpMode(mode) => {
                    self.arpeggiator.set_mode(mode);
                }

                Message::SetArpTranspose(semitones) => {
                    self.arpeggiator.set_transpose(semitones);
                }

                Message::SetArpChord(_chord_index) => {
                    // Future: Switch between hardcoded chord patterns
                    // For now, only C major is implemented
                }

                Message::SetEnvelopeAttack(attack_ms) => {
                    self.lead_envelope.set_attack_ms(attack_ms);
                }

                Message::SetEnvelopeDecay(decay_ms) => {
                    self.lead_envelope.set_decay_ms(decay_ms);
                }
            }
        }
    }

    /// BUFFER RATE (~172 Hz): Update cached parameters once per buffer.
    ///
    /// Called before processing each DMA buffer to recalculate coefficients
    /// that don't need audio-rate smoothing. LFO modulation applied here.
    #[inline]
    fn update_buffer_parameters(&mut self) {
        // Arpeggiator: tick at buffer rate, route to Lead oscillator
        let arp_note = self.arpeggiator.tick();

        // Detect note trigger (rising edge of gate: false → true)
        if arp_note.gate_active && !self.prev_gate_active {
            self.lead_envelope.trigger();
        }
        self.prev_gate_active = arp_note.gate_active;

        // Lead oscillator: frequency from arpeggiator, level always 1.0 (envelope handles amplitude)
        self.oscillators[0].set_frequency(arp_note.lead_freq);
        self.oscillators[0].set_level(1.0);

        // Bass oscillator: fixed 55 Hz drone with slow pitch drift (Lyra-8 style)
        // Frequency and level set once in new(), only detune changes here
        let drift_lfo_value = self.bass_drift_lfo.tick();
        let drift_cents = drift_lfo_value * crate::config::BASS_DRIFT_CENTS;
        self.oscillators[1].set_detune_cents(drift_cents);

        // Oscillator parameters: No-op (level/pitch smoothed at audio rate for responsiveness)
        for osc in &mut self.oscillators {
            osc.update_parameters();
        }

        // LFO modulation: tick at buffer rate, apply to filter cutoff
        let lfo_value = self.filter_lfo.tick();
        let modulated_cutoff = self.base_cutoff + (lfo_value * self.lfo_cutoff_range);
        self.filter.set_cutoff(modulated_cutoff);

        // Filter coefficients: OPTIMIZED - smoothed at buffer rate (only parameter that works well here)
        self.filter.update_coefficients();
    }

    /// AUDIO RATE (44,100 Hz): Render one audio sample - SUPER HOT PATH!
    ///
    /// OPTIMIZED: Filter coefficients cached at buffer rate, performance params at audio rate.
    /// - Oscillator tick() smooths pitch & level at audio rate (responsive control)
    /// - Normalization smoothed at audio rate (settled optimization keeps it cheap)
    /// - Filter processes with cached coefficients (ONLY param smoothed at buffer rate)
    ///
    /// # Returns
    /// Filtered mono sample (before master gain and soft-start)
    #[inline]
    fn render_sample(&mut self) -> f32 {
        // Tick lead envelope (audio rate)
        let envelope_level = self.lead_envelope.tick();

        // Render lead oscillator with envelope applied
        let lead_out = self.oscillators[0].tick() * envelope_level;

        // Render bass oscillator (no envelope, constant texture)
        let bass_out = self.oscillators[1].tick();

        // Mix oscillators
        let mix = lead_out + bass_out;

        // Normalize by sum of oscillator levels (smoothed at audio rate, cheap when settled)
        let normalized = mix * self.level_sum_reciprocal.value();

        // Process through Moog Ladder Filter (lowpass only, cached coefficients)
        let filtered = self.filter.process(normalized);

        filtered
    }

    /// Render audio samples into the DMA buffer.
    ///
    /// Processing rates:
    /// - BUFFER RATE (~172Hz): Update cached parameters, consume messages
    /// - AUDIO RATE (44.1kHz): Generate samples with audio-rate smoothing
    #[inline]
    async fn render(&mut self, audio_stream: &mut AudioStream) {
        #[cfg_attr(not(feature = "profile"), allow(unused_variables))]
        let bytes_written = audio_stream
            .push_with(|buffer| {
                // DIAGNOSTIC: Measure CPU time INSIDE push_with (excludes DMA waiting)
                #[cfg(feature = "profile")]
                let start = embassy_time::Instant::now();

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

                let bytes_written = buffer.len() - (buffer.len() % 4);

                // DIAGNOSTIC: Accumulate performance metrics (log only every ~1 second)
                #[cfg(feature = "profile")]
                {
                    let elapsed = embassy_time::Instant::now() - start;
                    let samples = bytes_written / 4;

                    if samples > 0 {
                        let micros_per_sample = elapsed.as_micros() / samples as u64;

                        // Accumulate stats
                        self.profile_stats.buffer_count += 1;
                        self.profile_stats.total_micros += elapsed.as_micros();
                        self.profile_stats.min_micros_per_sample =
                            self.profile_stats.min_micros_per_sample.min(micros_per_sample);
                        self.profile_stats.max_micros_per_sample =
                            self.profile_stats.max_micros_per_sample.max(micros_per_sample);

                        // Log every 172 buffers (~1 second at 44.1kHz with 256 samples/buffer)
                        if self.profile_stats.buffer_count >= 172 {
                            let avg_micros_per_sample =
                                self.profile_stats.total_micros / (self.profile_stats.buffer_count as u64 * samples as u64);

                            log::info!(
                                "CPU: avg={} us/sample, min={} us/sample, max={} us/sample ({} buffers)",
                                avg_micros_per_sample,
                                self.profile_stats.min_micros_per_sample,
                                self.profile_stats.max_micros_per_sample,
                                self.profile_stats.buffer_count
                            );

                            // Reset stats
                            self.profile_stats.buffer_count = 0;
                            self.profile_stats.total_micros = 0;
                            self.profile_stats.min_micros_per_sample = u64::MAX;
                            self.profile_stats.max_micros_per_sample = 0;
                        }
                    }
                }

                bytes_written
            })
            .await
            .unwrap_or(0);
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
