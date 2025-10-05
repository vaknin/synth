//! Engine module: manages voices, processes messages, renders audio.

use core::array::from_fn;

use crate::config::{MASTER_GAIN, SAMPLE_RATE, DEFAULT_FREQUENCY, VOICE_COUNT};
use crate::message::Message;
use crate::voice::Voice;
use crate::{AudioStream, ChannelReceiver, I16_MAX_F32};

/// Main synth engine managing all voices.
pub struct Engine {
    /// Array of voices (size determined by VOICE_COUNT config)
    voices: [Voice; VOICE_COUNT],

    /// Currently selected voice for control (None = no selection)
    selected_voice: Option<u8>,

    /// Message receiver from control tasks
    consumer: ChannelReceiver,

    /// Number of currently active voices
    active_count: u32,

    /// Cached reciprocal of active voice count (for fast normalization)
    active_count_reciprocal: f32,
}

impl Engine {
    /// Create new engine with initialized voices and message receiver.
    ///
    /// # Arguments
    /// * `receiver` - Embassy channel receiver for control messages
    ///
    /// # Returns
    /// Engine with VOICE_COUNT voices at STARTING_FREQUENCY, inactive, no selection
    pub fn new(
        receiver: ChannelReceiver,
    ) -> Self {
        Self {
            voices: from_fn(|_| Voice::new(DEFAULT_FREQUENCY, SAMPLE_RATE as f32)),
            selected_voice: None,
            consumer: receiver,
            active_count: 0,
            active_count_reciprocal: 1.0,
        }
    }

    /// Process a single control message.
    ///
    /// # Arguments
    /// * `msg` - Message from control task (buttons, pots, encoders)
    fn consume_messages(&mut self) {

        while let Ok(msg) = self.consumer.try_receive() {
        
        match msg {
            Message::SelectVoice(id) => {
                self.selected_voice = Some(id)
                // match self.selected_voice {
                //     // a voice is already selected
                //     Some(selected) => {
                //         if let Some(voice) = self.voices.get_mut(id as usize) {
                //             // it's this voice - deselect this
                            
                //             // it's another voice - deselect other, select this
                //         }
                        
                //     },
                //     // if no voice is selected - select it
                //     None => self.selected_voice = Some(id)
                // }
            }

            Message::ToggleVoice(idx) => {
                if let Some(voice) = self.voices.get_mut(idx as usize) {
                    let was_active = voice.active;
                    voice.set_active(!was_active);

                    // Update active count and cache reciprocal
                    if was_active {
                        self.active_count = self.active_count.saturating_sub(1);
                    } else {
                        self.active_count += 1;
                    }

                    // Cache reciprocal for fast multiplication (avoid division in tick)
                    self.active_count_reciprocal = if self.active_count > 0 {
                        1.0 / self.active_count as f32
                    } else {
                        1.0 // Doesn't matter, sum will be 0.0
                    };
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
    }

    async fn render(&mut self, audio_stream:&mut AudioStream) {
        audio_stream.push_with(|buffer| {


            if buffer.len() < 4 {
                return 0;
            }

            // Generate audio for each stereo frame
            for chunk in buffer.chunks_exact_mut(4) {

                let voices_sum: f32 = self.voices.iter_mut().map(|v| v.tick()).sum();
                let sample = (voices_sum * self.active_count_reciprocal * MASTER_GAIN * I16_MAX_F32) as i16;
                let bytes = sample.to_le_bytes();

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
