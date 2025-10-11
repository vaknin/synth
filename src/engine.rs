//! Engine module: manages generative voices, processes messages, renders audio.

use crate::clock::Clock;
use crate::config::MASTER_GAIN;
use crate::filter::MoogLadderFilter;
use crate::lfo::LFO;
use crate::message::Message;
use crate::slew::Slew;
use crate::voice::Voice;
use crate::{AudioStream, ChannelReceiver, I16_MAX_F32};
use micromath::F32Ext;

/// Fast soft-clipping saturation using rational function approximation.
///
/// Maps [-∞, ∞] → [-1, 1] smoothly, preserving signal character while preventing
/// digital clipping. Adds subtle analog warmth via smooth compression.
///
/// Uses micromath::F32Ext::abs() for consistent embedded performance.
///
/// Cost: ~5-8 cycles (vs 2-3 for hard clamp, but musically superior)
///
/// # Examples
/// ```
/// soft_clip(0.5)   ≈ 0.33  // Gentle compression below ±1
/// soft_clip(2.0)   ≈ 0.67  // Strong compression above ±1
/// soft_clip(10.0)  ≈ 0.91  // Asymptotic approach to ±1
/// ```
#[inline]
fn soft_clip(x: f32) -> f32 {
    x / (1.0 + F32Ext::abs(x))
}

/// Main synth engine with generative voices (Turing Machine + Euclidean rhythms).
pub struct Engine {
    /// Master tempo clock (drives all generative sequences)
    clock: Clock,

    /// Bass voice: low-frequency generative patterns
    bass_voice: Voice,

    /// Lead voice: high-frequency melodic sequences
    lead_voice: Voice,

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
}

impl Engine {
    /// Create new engine with generative voices.
    pub fn new(receiver: ChannelReceiver) -> Self {
        Self {
            clock: Clock::new(),
            bass_voice: Voice::bass(),
            lead_voice: Voice::lead(),
            consumer: receiver,
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
        }
    }

    /// BUFFER RATE (~172 Hz): Drain control message queue and update parameter targets.
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

                Message::SetFilterLfoRate(rate) => {
                    self.filter_lfo.set_frequency(rate);
                }

                Message::SetFilterLfoDepth(depth) => {
                    self.filter_lfo.depth = depth;
                    self.lfo_cutoff_range = self.base_cutoff * 3.0 * depth;
                }

                Message::SetTempo(bpm) => {
                    self.clock.set_bpm(bpm);
                }

                Message::SetBassEuclideanLength(length) => {
                    self.bass_voice.euclidean.set_length(length);
                }

                Message::SetBassEuclideanHits(hits) => {
                    self.bass_voice.euclidean.set_hits(hits);
                }

                Message::SetLeadEuclideanLength(length) => {
                    self.lead_voice.euclidean.set_length(length);
                }

                Message::SetLeadEuclideanHits(hits) => {
                    self.lead_voice.euclidean.set_hits(hits);
                }

                Message::SetMutationRate(rate) => {
                    self.bass_voice.turing.set_mutation_rate(rate);
                    self.lead_voice.turing.set_mutation_rate(rate);
                }

                _ => {} // Ignore old oscillator messages
            }
        }
    }

    /// BUFFER RATE (~172 Hz): Update cached parameters once per buffer.
    #[inline]
    fn update_buffer_parameters(&mut self) {
        // Clock-driven generative steps
        if self.clock.tick() {
            self.bass_voice.step();
            self.lead_voice.step();
        }

        // Voice parameter updates (oscillator pitch smoothing)
        self.bass_voice.osc1.update_parameters();
        self.bass_voice.osc2.update_parameters();
        self.lead_voice.osc1.update_parameters();
        self.lead_voice.osc2.update_parameters();

        // Filter LFO modulation
        let lfo_value = self.filter_lfo.tick();
        let modulated_cutoff = self.base_cutoff + (lfo_value * self.lfo_cutoff_range);
        self.filter.set_cutoff(modulated_cutoff);
        self.filter.update_coefficients();
    }

    /// AUDIO RATE (44,100 Hz): Render one audio sample - SUPER HOT PATH!
    #[inline]
    fn render_sample(&mut self) -> f32 {
        // Render both voices (oscillator + envelope)
        let bass = self.bass_voice.render();
        let lead = self.lead_voice.render();

        // Mix voices (bass-heavy for techno character)
        let mix = bass * 0.6 + lead * 0.8; //debug 0 instead of 0.4

        // Process through Moog Ladder Filter (can exceed ±1.0 at high resonance)
        let filtered = self.filter.process(mix);

        // Soft saturation: prevents i16 wraparound + adds analog warmth
        // Fast approximation: ~5 cycles vs 2 for hard clamp, but preserves filter character
        // TODO: Add drive control (pre-gain) when Pot 11 is wired (Phase 1 roadmap)
        soft_clip(filtered)
    }

    /// Render audio samples into the DMA buffer.
    ///
    /// Processing rates:
    /// - BUFFER RATE (~172Hz): Update cached parameters, consume messages
    /// - AUDIO RATE (44.1kHz): Generate samples with audio-rate smoothing
    #[inline]
    async fn render(&mut self, audio_stream: &mut AudioStream) {
        audio_stream
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
    }

    /// Render audio into provided buffer.
    ///
    /// Processes all pending control messages, generates audio samples,
    /// converts to i16 stereo format, and writes to buffer.
    ///
    /// # Arguments
    /// * `buffer` - Output buffer for i16 LE stereo audio (must be multiple of 4 bytes)
    #[inline]
    pub async fn stream_audio(&mut self, audio_stream: &mut AudioStream) {
        // Process all pending control messages
        self.consume_messages();

        self.render(audio_stream).await;
    }
}
