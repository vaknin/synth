//! Engine module: manages voices, processes messages, renders audio.

use crate::config::{VOICE_COUNT, STARTING_FREQUENCY};
use crate::message::Message;
use crate::voice::Voice;

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
    /// Sum of all active voices normalized by VOICE_COUNT to prevent clipping
    pub fn tick(&mut self) -> f32 {
        let sum: f32 = self.voices.iter_mut().map(|v| v.tick()).sum();
        sum / VOICE_COUNT as f32
    }
}
