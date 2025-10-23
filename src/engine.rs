//! Engine module: manages generative voices, processes messages, renders audio.

use crate::clock::Clock;
use crate::config::{BASS_GAIN, LEAD_GAIN, MASTER_GAIN};
use crate::filter::{soft_clip, MoogLadderFilter};
use crate::lfo::LFO;
use crate::message::Message;
use crate::slew::Slew;
use crate::voice::Voice;
use crate::{AudioStream, ChannelReceiver, I16_MAX_F32};

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

    /// Soft-start fade-in using Slew (protects speakers/headphones)
    /// Slow fade prevents startup transients and gives hardware time to stabilize
    soft_start_gain: Slew,

    /// Bass Moog Ladder Filter (4-pole lowpass with resonance)
    bass_filter: MoogLadderFilter,

    /// Lead Moog Ladder Filter (independent spectral control)
    lead_filter: MoogLadderFilter,

    /// Bass filter cutoff LFO (evolving drone)
    bass_lfo: LFO,

    /// Base bass filter cutoff from pot (before LFO modulation)
    bass_base_cutoff: f32,

    /// Bass LFO modulation range in Hz (depth pot → Hz range)
    bass_lfo_cutoff_range: f32,

    /// Base lead filter cutoff from pot (static, no LFO)
    lead_base_cutoff: f32,
}

impl Engine {
    /// Create new engine with generative voices.
    pub fn new(receiver: ChannelReceiver) -> Self {
        // Create voices (portamento already set in constructors)
        let mut bass_voice = Voice::bass();
        let mut lead_voice = Voice::lead();

        // Set Phrygian scale (index 7): dark, hypnotic, perfect for techno
        let phrygian_normalized = 7.0 / 11.0;
        bass_voice.quantizer.set_scale(phrygian_normalized);
        lead_voice.quantizer.set_scale(phrygian_normalized);

        // Initialize per-voice filters
        let bass_filter = MoogLadderFilter::new(
            crate::config::FILTER_CUTOFF_MIN,
            crate::config::FILTER_RESONANCE_MIN,
        );
        let lead_filter = MoogLadderFilter::new(
            crate::config::FILTER_CUTOFF_MIN,
            crate::config::FILTER_RESONANCE_MIN,
        );

        // === MANUAL FILTER TUNING (until pots are wired) ===
        // Uncomment to override pot values for testing spectral separation:
        // bass_filter.set_cutoff(300.0);  // Dark, warm drone (low-mids)
        // lead_filter.set_cutoff(2000.0); // Bright, cutting melodies (upper mids)
        // bass_filter.set_resonance(2.5); // Thick, resonant bass character
        // lead_filter.set_resonance(1.0); // Clean, articulate lead

        Self {
            clock: Clock::new(),
            bass_voice,
            lead_voice,
            consumer: receiver,
            soft_start_gain: {
                let mut gain = Slew::with_coeff(0.0, 0.9995);
                gain.set_target(1.0);
                gain
            },
            bass_filter,
            lead_filter,
            bass_lfo: LFO::new(
                crate::config::LFO_FREQUENCY_DEFAULT,
                crate::config::LFO_DEPTH_DEFAULT,
            ),
            bass_base_cutoff: crate::config::FILTER_CUTOFF_MIN,
            bass_lfo_cutoff_range: 0.0,
            lead_base_cutoff: crate::config::FILTER_CUTOFF_MIN,
        }
    }

    /// BUFFER RATE: Drain control message queue and update parameter targets.
    #[inline]
    fn consume_messages(&mut self) {
        while let Ok(msg) = self.consumer.try_receive() {
            match msg {
                // === BASS FILTER ===
                Message::SetBassCutoff(freq) => {
                    self.bass_base_cutoff = freq;
                }

                Message::SetBassResonance(q) => {
                    self.bass_filter.set_resonance(q);
                }

                Message::SetBassLfoRate(rate) => {
                    self.bass_lfo.set_frequency(rate);
                }

                Message::SetBassLfoDepth(depth) => {
                    self.bass_lfo.depth = 1.0;
                    self.bass_lfo_cutoff_range = self.bass_base_cutoff * 1.5 * depth;
                }

                // === LEAD FILTER ===
                Message::SetLeadCutoff(freq) => {
                    self.lead_base_cutoff = freq;
                    self.lead_filter.set_cutoff(freq);
                }

                Message::SetLeadResonance(q) => {
                    self.lead_filter.set_resonance(q);
                }

                // === GENERATIVE PARAMETERS ===
                Message::SetTempo(bpm) => {
                    self.clock.set_bpm(bpm);
                }

                Message::SetLeadEuclideanHits(hits) => {
                    if let Some(euclidean) = &mut self.lead_voice.euclidean {
                        euclidean.set_hits(hits);
                    }
                }

                Message::SetScale(scale_idx) => {
                    let normalized = scale_idx as f32 / 11.0;
                    self.bass_voice.quantizer.set_scale(normalized);
                    self.lead_voice.quantizer.set_scale(normalized);
                }

                Message::SetPortamento(coeff) => {
                    self.bass_voice.osc.set_portamento_coeff(coeff);
                    self.lead_voice.osc.set_portamento_coeff(coeff);
                }

                Message::SetDecay(decay_ms) => {
                    if let Some(envelope) = &mut self.lead_voice.envelope {
                        envelope.set_decay_ms(decay_ms);
                    }
                }
            }
        }
    }

    /// BUFFER RATE: Update cached parameters once per buffer.
    #[inline]
    fn update_buffer_parameters(&mut self) {
        // Clock-driven generative steps
        if self.clock.tick() {
            self.bass_voice.step();
            self.lead_voice.step();
        }

        // Bass filter: LFO modulation for evolving drone (only when depth > 0)
        if self.bass_lfo_cutoff_range > 0.0 {
            let lfo_modulation = self.bass_lfo.tick() * self.bass_lfo_cutoff_range;
            let bass_modulated_cutoff = self.bass_base_cutoff + lfo_modulation;
            self.bass_filter
                .set_cutoff(bass_modulated_cutoff.max(20.0).min(20000.0));
        }

        // Update filter coefficients (dirty flag prevents redundant tan() calls)
        self.bass_filter.update_coefficients();
        self.lead_filter.update_coefficients();
    }

    /// AUDIO RATE: Render one audio sample - SUPER HOT PATH!
    #[inline]
    fn render_sample(&mut self) -> f32 {
        // Render both voices (oscillator + envelope)
        let bass = self.bass_voice.render();
        let lead = self.lead_voice.render();

        // Per-voice filtering (spectral separation before mixing)
        let bass_filtered = self.bass_filter.process(bass);
        let lead_filtered = self.lead_filter.process(lead);

        // Mix filtered voices with gain staging
        let mix = bass_filtered * BASS_GAIN + lead_filtered * LEAD_GAIN;

        // Clean mixing - let master gain handle scaling, hard clip only at DAC
        mix
    }

    /// Render audio samples into the DMA buffer.
    ///
    /// Processing rates:
    /// - BUFFER RATE: Update cached parameters, consume messages
    /// - AUDIO RATE: Generate samples with audio-rate smoothing
    #[inline]
    async fn render(&mut self, audio_stream: &mut AudioStream) {
        audio_stream
            .push_with(|buffer| {
                if buffer.len() < 4 {
                    return 0;
                }

                // === BUFFER RATE: Update parameters once per buffer ===
                self.update_buffer_parameters();

                // Cache soft-start gain (fades in gradually, then no-op via settled flag)
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
