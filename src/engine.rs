//! Engine module: manages voices, processes messages, renders audio.

use crate::audio_util::f32_to_i16_le;
use crate::config::{VOICE_COUNT, STARTING_FREQUENCY, MASTER_GAIN};
use crate::message::Message;
use crate::voice::Voice;
use heapless::spsc::Consumer;

/// Main synth engine managing all voices.
pub struct Engine {
    /// Array of voices (size determined by VOICE_COUNT config)
    voices: [Voice; VOICE_COUNT],

    /// Currently selected voice for control (None = no selection)
    selected_voice: Option<u8>,

    /// Audio sample rate (stored for future use in filters/effects)
    #[allow(dead_code)]
    sample_rate: f32,
}

impl Engine {
    /// Create new engine with initialized voices.
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    ///
    /// # Returns
    /// Engine with VOICE_COUNT voices at STARTING_FREQUENCY, inactive, no selection
    pub fn new(sample_rate: f32) -> Self {
        Self {
            voices: core::array::from_fn(|_| Voice::new(STARTING_FREQUENCY, sample_rate)),
            selected_voice: None,
            sample_rate,
        }
    }

    /// Process a single control message.
    ///
    /// # Arguments
    /// * `msg` - Message from control task (buttons, pots, encoders)
    pub fn process_message(&mut self, msg: Message) {
        match msg {
            Message::SelectVoice(idx) => {
                self.selected_voice = Some(idx);
            }

            Message::ToggleVoice(idx) => {
                if let Some(voice) = self.voices.get_mut(idx as usize) {
                    voice.set_active(!voice.active);
                }
            }

            Message::SetFrequency(freq) => {
                if let Some(idx) = self.selected_voice {
                    if let Some(voice) = self.voices.get_mut(idx as usize) {
                        voice.set_frequency(freq);
                    }
                }
            }

            Message::SetVolume(vol) => {
                if let Some(idx) = self.selected_voice {
                    if let Some(voice) = self.voices.get_mut(idx as usize) {
                        voice.set_volume(vol);
                    }
                }
            }
        }
    }

    /// Generate next mixed audio sample from all voices.
    ///
    /// # Returns
    /// Sum of all active voices with master gain applied
    pub fn tick(&mut self) -> f32 {
        let sum: f32 = self.voices.iter_mut().map(|v| v.tick()).sum();
        (sum / VOICE_COUNT as f32) * MASTER_GAIN
    }

    /// Render audio into provided buffer.
    ///
    /// Processes all pending control messages, generates audio samples,
    /// converts to i16 stereo format, and writes to buffer.
    ///
    /// # Arguments
    /// * `consumer` - Consumer end of message queue for processing control messages
    /// * `buffer` - Output buffer for i16 LE stereo audio (must be multiple of 4 bytes)
    ///
    /// # Returns
    /// Number of bytes written to buffer (will be multiple of 4)
    pub fn render(&mut self, consumer: &mut Consumer<'static, Message>, buffer: &mut [u8]) -> usize {
        if buffer.len() < 4 {
            return 0;
        }

        // Process all pending control messages
        while let Some(msg) = consumer.dequeue() {
            self.process_message(msg);
        }

        // Generate audio for each stereo frame
        for chunk in buffer.chunks_exact_mut(4) {
            let bytes = f32_to_i16_le(self.tick());
            chunk.copy_from_slice(&[bytes[0], bytes[1], bytes[0], bytes[1]]);
        }

        buffer.len() - (buffer.len() % 4)
    }
}
