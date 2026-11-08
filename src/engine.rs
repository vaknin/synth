//! Engine module: manages oscillators, processes messages, renders audio.

use crate::config::{
    MASTER_GAIN, WAVEFORM_BUFFER_SIZE, WAVEFORM_DECIMATION_FACTOR, WAVEFORM_SEND_INTERVAL,
};
use crate::filter::MoogLadderFilter;
use crate::lfo::LFO;
use crate::message::Message;
use crate::oscillator::{Oscillator, Waveform};
use crate::slew::Slew;
use crate::{AudioStream, ChannelReceiver, WaveformData, WaveformSignal, I16_MAX_F32};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Sender;

/// Waveform capture state for oscilloscope visualization.
///
/// Captures every Nth audio sample (decimation) for display on OLED.
/// Sends to display via Signal (overwrites previous - latest-value semantics).
struct WaveformCapture {
    /// Sample buffer (1024 i16 samples)
    buffer: [i16; WAVEFORM_BUFFER_SIZE],

    /// Decimation counter (capture every WAVEFORM_DECIMATION_FACTOR samples)
    decimation_counter: u32,

    /// Current write position in buffer (0-1023)
    write_index: usize,

    /// Frame counter for periodic sends (every N decimated samples)
    send_counter: u32,
}

impl WaveformCapture {
    fn new() -> Self {
        Self {
            buffer: [0; WAVEFORM_BUFFER_SIZE],
            decimation_counter: 0,
            write_index: 0,
            send_counter: 0,
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

    /// Watch sender for sending waveform data to display (Core 0 → Core 1)
    waveform_sender: Sender<'static, CriticalSectionRawMutex, WaveformData, 1>,
}

impl Engine {
    /// Create new engine with single drone oscillator and message receiver.
    ///
    /// # Arguments
    /// * `receiver` - Embassy channel receiver for control messages
    /// * `waveform_signal` - Watch for sending waveform data to display
    ///
    /// # Returns
    /// Engine with single sine oscillator at 220Hz (A3), always on at full level.
    pub fn new(receiver: ChannelReceiver, waveform_signal: &'static WaveformSignal) -> Self {
        // Single sine oscillator configured as drone (220Hz A3, full level)
        let mut oscillator = Oscillator::new(Waveform::Sine);
        oscillator.set_frequency(70.0); // A3 note (220Hz)
        oscillator.set_level(1.0); // Always on, full volume

        log::info!("Engine::new() - 220Hz sine oscillator initialized");

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
            waveform_sender: waveform_signal.sender(),
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
    /// - Single oscillator tick() → filter
    ///
    /// # Returns
    /// Raw mono sample (before master gain and soft-start)
    #[inline]
    fn render_sample(&mut self) -> f32 {
        // Single oscillator tick (220Hz A3 drone at full level) → Moog filter
        let sample = self.oscillator.tick();
        self.filter.process(sample)
    }

    /// Render audio samples into the DMA buffer.
    ///
    /// Processing rates:
    /// - BUFFER RATE (~172Hz): Update cached parameters, consume messages
    /// - AUDIO RATE (44.1kHz): Generate samples with audio-rate smoothing
    #[inline]
    async fn render(&mut self, audio_stream: &mut AudioStream) {
        let bytes_written = audio_stream
            .push_with(|buffer| {
                if buffer.len() < 4 {
                    return 0; // Buffer not ready yet (normal for circular DMA)
                }

                // === BUFFER RATE: Update parameters once per buffer ===
                self.update_buffer_parameters();

                // Cache soft-start gain (settles in ~10ms, then no-op via settled flag)
                let fade_gain = self.soft_start_gain.value();

                static mut SAMPLE_COUNTER: u32 = 0;
                static mut FIRST_NONZERO_SAMPLE: Option<i16> = None;
                static mut FIRST_WRITE_LOGGED: bool = false;
                static mut LATEST_SAMPLE: i16 = 0;

                // === AUDIO RATE: Process each stereo frame ===
                for chunk in buffer.chunks_exact_mut(4) {
                    // Render one sample (oscillator mix → normalize → filter)
                    let sample = self.render_sample();

                    // Apply soft-start fade, master gain, convert to i16
                    let output = (sample * fade_gain * MASTER_GAIN * I16_MAX_F32) as i16;
                    let bytes = output.to_le_bytes();

                    // Capture first non-zero sample and track latest
                    unsafe {
                        if FIRST_NONZERO_SAMPLE.is_none() && output != 0 {
                            FIRST_NONZERO_SAMPLE = Some(output);
                            log::info!("First non-zero sample: {}", output);
                        }
                        LATEST_SAMPLE = output;
                        SAMPLE_COUNTER += 1;
                    }

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
                        self.waveform_capture.send_counter += 1;

                        // Send periodically: adaptive rate (2× display refresh for smooth updates)
                        if self.waveform_capture.send_counter >= WAVEFORM_SEND_INTERVAL {
                            self.waveform_capture.send_counter = 0;

                            let data = WaveformData {
                                samples: self.waveform_capture.buffer,
                            };
                            self.waveform_sender.send(data);
                        }

                        // Wrap around at buffer size (circular buffer)
                        if self.waveform_capture.write_index >= WAVEFORM_BUFFER_SIZE {
                            self.waveform_capture.write_index = 0;
                        }
                    }
                    self.waveform_capture.decimation_counter += 1;
                }

                let bytes_written = buffer.len() - (buffer.len() % 4);

                // Log first successful write and periodically
                unsafe {
                    if !FIRST_WRITE_LOGGED && bytes_written > 0 {
                        log::info!("I2S: FIRST write successful! {} bytes", bytes_written);
                        FIRST_WRITE_LOGGED = true;
                    }

                    // Log every ~2 seconds
                    if SAMPLE_COUNTER % (44100 * 2) == 0 && SAMPLE_COUNTER > 0 {
                        log::info!(
                            "I2S: {} bytes/buffer, latest_sample={}, first_nonzero={:?}, total={}",
                            bytes_written,
                            LATEST_SAMPLE,
                            FIRST_NONZERO_SAMPLE,
                            SAMPLE_COUNTER
                        );
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
