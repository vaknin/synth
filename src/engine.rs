//! Engine module: manages voices, processes messages, renders audio.

use crate::config::{MESSAGE_QUEUE_SIZE, VOICE_COUNT, STARTING_FREQUENCY, MASTER_GAIN};
use crate::message::Message;
use crate::voice::Voice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Receiver;

/// Main synth engine managing all voices.
pub struct Engine {
    /// Array of voices (size determined by VOICE_COUNT config)
    voices: [Voice; VOICE_COUNT],

    /// Currently selected voice for control (None = no selection)
    selected_voice: Option<u8>,

    /// Audio sample rate (stored for future use in filters/effects)
    #[allow(dead_code)]
    sample_rate: f32,

    /// Message receiver from control tasks
    receiver: Receiver<'static, CriticalSectionRawMutex, Message, MESSAGE_QUEUE_SIZE>,
}

impl Engine {
    /// Create new engine with initialized voices and message receiver.
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `receiver` - Embassy channel receiver for control messages
    ///
    /// # Returns
    /// Engine with VOICE_COUNT voices at STARTING_FREQUENCY, inactive, no selection
    pub fn new(
        sample_rate: f32,
        receiver: Receiver<'static, CriticalSectionRawMutex, Message, MESSAGE_QUEUE_SIZE>,
    ) -> Self {
        Self {
            voices: core::array::from_fn(|_| Voice::new(STARTING_FREQUENCY, sample_rate)),
            selected_voice: None,
            sample_rate,
            receiver,
        }
    }

    /// Process a single control message.
    ///
    /// # Arguments
    /// * `msg` - Message from control task (buttons, pots, encoders)
    pub fn process_message(&mut self, msg: Message) {
        match msg {
            Message::SelectVoice(id) => {
                match self.selected_voice {
                    // a voice is already selected
                    Some(selected) => {
                        if let Some(voice) = self.voices.get_mut(id as usize) {
                            // it's this voice - deselect this
                            // if voice.
    
                            // it's another voice - deselect other, select this
                        }
                        
                    },
                    // if no voice is selected - select it
                    None => self.selected_voice = Some(id)
                }
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
    /// * `buffer` - Output buffer for i16 LE stereo audio (must be multiple of 4 bytes)
    ///
    /// # Returns
    /// Number of bytes written to buffer (will be multiple of 4)
    pub fn render(&mut self, buffer: &mut [u8]) -> usize {
        if buffer.len() < 4 {
            return 0;
        }

        // Process all pending control messages (non-blocking)
        while let Ok(msg) = self.receiver.try_receive() {
            self.process_message(msg);
        }

        // Generate audio for each stereo frame
        for chunk in buffer.chunks_exact_mut(4) {
            let sample_i16 = (self.tick() * (i16::MAX as f32)) as i16;
            let bytes = sample_i16.to_le_bytes();
            chunk.copy_from_slice(&[bytes[0], bytes[1], bytes[0], bytes[1]]);
        }

        buffer.len() - (buffer.len() % 4)
    }
}
