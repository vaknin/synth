//! Engine module: manages generative voices, processes messages, renders audio.

use crate::clock::Clock;
use crate::config::{BASS_GAIN, FILTER_ENVELOPE_DECAY_COEFF, LEAD_GAIN, MASTER_GAIN};
use crate::filter::{soft_clip, MoogLadderFilter};
use crate::lfo::LFO;
use crate::message::Message;
use crate::slew::Slew;
use crate::voice::Voice;
use crate::{AudioStream, ChannelReceiver, I16_MAX_F32};

// Debug storage for buffer-rate logging (updated at audio rate, read at buffer rate)
static mut LAST_BASS: f32 = 0.0;
static mut LAST_LEAD: f32 = 0.0;
static mut LAST_MIX: f32 = 0.0;
static mut LAST_FILTERED: f32 = 0.0;

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

    /// Moog Ladder Filter (4-pole lowpass with resonance)
    filter: MoogLadderFilter,

    /// Filter cutoff LFO
    filter_lfo: LFO,

    /// Base filter cutoff from pot (before modulation)
    base_cutoff: f32,

    /// LFO modulation range in Hz (depth pot → Hz range)
    lfo_cutoff_range: f32,

    /// Filter envelope depth in Hz (pot-controlled)
    filter_envelope_depth: f32,

    /// Current filter envelope offset in Hz (decays to 0)
    filter_envelope_offset: f32,
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

        Self {
            clock: Clock::new(),
            bass_voice,
            lead_voice,
            consumer: receiver,
            soft_start_gain: {
                let mut gain = Slew::with_coeff(0.0, 0.9995); // ~500ms fade-in
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
            filter_envelope_depth: 0.0,
            filter_envelope_offset: 0.0,
        }
    }

    /// BUFFER RATE: Drain control message queue and update parameter targets.
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
                    // LFO always outputs ±1.0, engine scales by desired range
                    self.filter_lfo.depth = 1.0;
                    self.lfo_cutoff_range = self.base_cutoff * 1.5 * depth;
                }

                Message::SetTempo(bpm) => {
                    self.clock.set_bpm(bpm);
                }

                Message::SetLeadEuclideanHits(hits) => {
                    if let Some(euclidean) = &mut self.lead_voice.euclidean {
                        euclidean.set_hits(hits);
                    }
                }

                Message::SetScale(scale_idx) => {
                    // Apply same scale to both voices (normalized 0.0-1.0)
                    let normalized = scale_idx as f32 / 11.0;
                    self.bass_voice.quantizer.set_scale(normalized);
                    self.lead_voice.quantizer.set_scale(normalized);
                }

                Message::SetPortamento(coeff) => {
                    self.bass_voice.osc1.set_portamento_coeff(coeff);
                    self.bass_voice.osc2.set_portamento_coeff(coeff);
                    self.lead_voice.osc1.set_portamento_coeff(coeff);
                    self.lead_voice.osc2.set_portamento_coeff(coeff);
                }

                Message::SetFilterEnvelopeDepth(depth) => {
                    // depth is 0.0-1.0, scale to Hz using config constant
                    self.filter_envelope_depth = depth * crate::config::FILTER_ENVELOPE_DEPTH_MAX;
                }

                Message::SetAttack(attack_ms) => {
                    // Only lead has envelope
                    if let Some(envelope) = &mut self.lead_voice.envelope {
                        envelope.set_attack_ms(attack_ms);
                    }
                }

                Message::SetDecay(decay_ms) => {
                    // Only lead has envelope
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
        // Debug diagnostics (buffer rate only - ~172 Hz, negligible overhead)
        static mut FRAME_COUNT: u32 = 0;
        unsafe {
            FRAME_COUNT += 1;
            // Log every ~0.5 seconds (86 buffers)
            if FRAME_COUNT % 86 == 0 {
                log::info!(
                    "Filter: cut={:.0}Hz res={:.2} | Bass={:+.3} Lead={:+.3} Mix={:+.3} Filt={:+.3}",
                    self.filter.cutoff,
                    self.filter.resonance,
                    LAST_BASS,
                    LAST_LEAD,
                    LAST_MIX,
                    LAST_FILTERED
                );
            }
        }

        // Clock-driven generative steps
        if self.clock.tick() {
            // Check if lead voice fires a gate AND is audible (trigger filter envelope)
            let lead_gate = if let Some(euclidean) = &self.lead_voice.euclidean {
                euclidean.pattern[euclidean.position]
            } else {
                false
            };

            if lead_gate && self.lead_voice.is_audible() {
                // Trigger filter envelope jump on lead hits (only if lead is playing)
                self.filter_envelope_offset = self.filter_envelope_depth;
            }

            // Step voices (bass ticks Turing slowly, lead uses Euclidean)
            self.bass_voice.step();
            self.lead_voice.step();
        }

        // Decay filter envelope exponentially at buffer rate
        self.filter_envelope_offset *= FILTER_ENVELOPE_DECAY_COEFF;

        // Voice parameter updates (oscillator pitch smoothing)
        self.bass_voice.osc1.update_parameters();
        self.bass_voice.osc2.update_parameters();
        self.lead_voice.osc1.update_parameters();
        self.lead_voice.osc2.update_parameters();

        // Tick LFO and get modulation value (bipolar: -depth to +depth)
        let lfo_modulation = self.filter_lfo.tick() * self.lfo_cutoff_range;

        // Apply combined filter modulation: base + envelope + LFO
        let modulated_cutoff = self.base_cutoff + self.filter_envelope_offset + lfo_modulation;

        // Clamp to safe range (20 Hz - 20 kHz)
        self.filter
            .set_cutoff(modulated_cutoff.max(20.0).min(20000.0));
        self.filter.update_coefficients();
    }

    /// AUDIO RATE: Render one audio sample - SUPER HOT PATH!
    #[inline]
    fn render_sample(&mut self) -> f32 {
        // Render both voices (oscillator + envelope)
        let bass = self.bass_voice.render();
        let lead = self.lead_voice.render();

        // Mix voices with gain staging (bass attenuated, lead prominent)
        let mix = bass * BASS_GAIN + lead * LEAD_GAIN;

        // Debug: Log levels periodically (buffer rate in update_buffer_parameters)
        // Store last mix/filter values for diagnostics
        unsafe {
            LAST_BASS = bass;
            LAST_LEAD = lead;
            LAST_MIX = mix;
        }

        // Process through Moog Ladder Filter (can exceed ±1.0 at high resonance)
        let filtered = self.filter.process(mix);

        unsafe {
            LAST_FILTERED = filtered;
        }

        // Soft-clip prevents digital clipping while preserving musical character
        soft_clip(filtered)
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
