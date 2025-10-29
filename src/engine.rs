//! Engine module: manages oscillators, processes messages, renders audio.

use crate::config::{
    MASTER_GAIN, WAVEFORM_BUFFER_SIZE, WAVEFORM_DECIMATION_FACTOR, WAVEFORM_SAMPLES_PER_FRAME_FIXED,
};
use crate::filter::MoogLadderFilter;
use crate::lfo::LFO;
use crate::message::{Message, WaveformData};
use crate::oscillator::{Oscillator, Waveform};
use crate::slew::Slew;
use crate::{AudioStream, ChannelReceiver, WaveformProducer, I16_MAX_F32};

/// Waveform capture state for oscilloscope visualization.
///
/// Captures every Nth audio sample (decimation) for display on OLED.
/// Sends to display at exact refresh rate using fixed-point accumulator.
struct WaveformCapture {
    /// Sample buffer (1024 i16 samples)
    buffer: [i16; WAVEFORM_BUFFER_SIZE],

    /// Decimation counter (capture every WAVEFORM_DECIMATION_FACTOR samples)
    decimation_counter: u32,

    /// Current write position in buffer (0-1023)
    write_index: usize,

    /// Fixed-point accumulator (16.16) for rate-matched sending
    send_accumulator: u32,
}

impl WaveformCapture {
    fn new() -> Self {
        Self {
            buffer: [0; WAVEFORM_BUFFER_SIZE],
            decimation_counter: 0,
            write_index: 0,
            send_accumulator: 0,
        }
    }
}

/// Main synth engine - simplified drone synth with single oscillator.
pub struct Engine {
    /// Single sine oscillator at 220Hz (A3, always on)
    oscillator: Oscillator,

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

    /// Waveform capture state for oscilloscope display
    waveform_capture: WaveformCapture,

    /// SPSC producer for sending waveform data to display (Core 0 → Core 1)
    waveform_producer: WaveformProducer,

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
    /// Create new engine with single drone oscillator and message receiver.
    ///
    /// # Arguments
    /// * `receiver` - Embassy channel receiver for control messages
    /// * `waveform_producer` - SPSC producer for sending waveform data to display
    ///
    /// # Returns
    /// Engine with single sine oscillator at 110Hz (A2), always on at full level.
    pub fn new(receiver: ChannelReceiver, waveform_producer: WaveformProducer) -> Self {
        // Single sine oscillator configured as drone (220Hz A3, full level)
        let mut oscillator = Oscillator::new(Waveform::Sine);
        oscillator.set_frequency(220.0); // A3 note (220Hz)
        oscillator.set_level(1.0); // Always on, full volume

        Self {
            oscillator,
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
            lfo_cutoff_range: 0.0, // LFO disabled - no modulation applied
            waveform_capture: WaveformCapture::new(),
            waveform_producer,
            #[cfg(feature = "profile")]
            profile_stats: ProfileStats {
                buffer_count: 0,
                total_micros: 0,
                min_micros_per_sample: u64::MAX,
                max_micros_per_sample: 0,
            },
        }
    }

    /// BUFFER RATE (~172 Hz): Drain control message queue and update parameter targets.
    ///
    /// Called once per buffer. Processes all pending messages from pot polling (50Hz).
    /// May process 0, 1, or multiple messages per buffer depending on pot activity.
    ///
    /// Simplified for drone synth: only handles filter cutoff and resonance.
    #[inline]
    fn consume_messages(&mut self) {
        while let Ok(msg) = self.consumer.try_receive() {
            match msg {
                Message::SetCutoff(freq) => {
                    self.base_cutoff = freq;
                }

                Message::SetResonance(q) => {
                    self.filter.set_resonance(q);
                }
            }
        }
    }

    /// BUFFER RATE (~172 Hz): Update cached parameters once per buffer.
    ///
    /// Called before processing each DMA buffer to recalculate filter coefficients.
    /// LFO is disabled (lfo_cutoff_range = 0.0), so cutoff is applied directly.
    #[inline]
    fn update_buffer_parameters(&mut self) {
        // Oscillator parameters: No-op (level/pitch smoothed at audio rate for responsiveness)
        self.oscillator.update_parameters();

        // Set filter cutoff directly (no LFO modulation)
        self.filter.set_cutoff(self.base_cutoff);

        // Filter coefficients: OPTIMIZED - smoothed at buffer rate (only parameter that works well here)
        self.filter.update_coefficients();
    }

    /// AUDIO RATE (44,100 Hz): Render one audio sample - SUPER HOT PATH!
    ///
    /// Simplified for single drone oscillator:
    /// - Single oscillator tick() (no mixing, no normalization, no filtering)
    ///
    /// # Returns
    /// Raw mono sample (before master gain and soft-start)
    #[inline]
    fn render_sample(&mut self) -> f32 {
        // Single oscillator tick (220Hz A3 drone at full level, no filter)
        self.oscillator.tick()
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

                    // === WAVEFORM CAPTURE: Decimation for oscilloscope display ===
                    // Capture every Nth sample (N=DECIMATION_FACTOR, varies with TIME_WINDOW)
                    if self.waveform_capture.decimation_counter % WAVEFORM_DECIMATION_FACTOR == 0 {
                        // Write to circular buffer (1024 samples)
                        self.waveform_capture.buffer[self.waveform_capture.write_index] = output;
                        self.waveform_capture.write_index += 1;

                        // Accumulator-based send: match display refresh rate exactly (prevents queue overflow)
                        // Add 1.0 sample in 16.16 fixed-point format
                        self.waveform_capture.send_accumulator += 1 << 16;

                        // Send when we've accumulated enough samples for one display frame
                        // At 120 Hz: 7.35 samples/frame → sends exactly 120 times/second
                        if self.waveform_capture.send_accumulator >= WAVEFORM_SAMPLES_PER_FRAME_FIXED {
                            self.waveform_capture.send_accumulator -= WAVEFORM_SAMPLES_PER_FRAME_FIXED;

                            let data = WaveformData {
                                samples: self.waveform_capture.buffer,
                                write_index: self.waveform_capture.write_index,
                            };
                            // enqueue() returns Err if full - should never happen with rate-matched sending
                            match self.waveform_producer.enqueue(data) {
                                Ok(_) => {
                                    #[cfg(feature = "profile")]
                                    if self.profile_stats.buffer_count % 172 == 0 {
                                        log::info!("Waveform sent: write_index={}", self.waveform_capture.write_index);
                                    }
                                }
                                Err(_) => {
                                    log::warn!("Waveform queue full!");
                                }
                            }
                        }

                        // Wrap around at buffer size (circular buffer)
                        if self.waveform_capture.write_index >= WAVEFORM_BUFFER_SIZE {
                            self.waveform_capture.write_index = 0;
                        }
                    }
                    self.waveform_capture.decimation_counter += 1;
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
