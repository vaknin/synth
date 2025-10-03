//! Engine module: manages voices, processes messages, renders audio.

use core::array::from_fn;

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

    /// Number of currently active voices
    active_count: u32,

    /// Cached reciprocal of active voice count (for fast normalization)
    active_count_reciprocal: f32,
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
            voices: from_fn(|_| Voice::new(STARTING_FREQUENCY, sample_rate)),
            selected_voice: None,
            sample_rate,
            receiver,
            active_count: 0,
            active_count_reciprocal: 1.0,
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
                            
                            // it's another voice - deselect other, select this
                        }
                        
                    },
                    // if no voice is selected - select it
                    None => self.selected_voice = Some(id)
                }
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

    /// Generate next mixed audio sample from all voices.
    ///
    /// # Returns
    /// Sum of all active voices, normalized by active count, with master gain applied
    pub fn tick(&mut self) -> f32 {
        let sum: f32 = self.voices.iter_mut().map(|v| v.tick()).sum();

        // active_count_reciprocal is pre-computed when voices toggle
        sum * self.active_count_reciprocal * MASTER_GAIN
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
        // if clicks or issues, check this section because of 'while' drains everything
        while let Ok(msg) = self.receiver.try_receive() {
            self.process_message(msg);
        }

        // Cache constant outside loop (computed once instead of per-sample)
        const I16_MAX_F32: f32 = i16::MAX as f32;

        // Generate audio for each stereo frame
        for chunk in buffer.chunks_exact_mut(4) {
            let sample_i16 = (self.tick() * I16_MAX_F32) as i16;
            let bytes = sample_i16.to_le_bytes();
            // Direct assignment is faster than copy_from_slice for 4 bytes
            chunk[0] = bytes[0];
            chunk[1] = bytes[1];
            chunk[2] = bytes[0];
            chunk[3] = bytes[1];
        }

        buffer.len() - (buffer.len() % 4)
    }
}
