//! Message types for lock-free communication between control tasks and audio task.

/// Messages sent from control tasks (buttons, pots, encoders) to audio task.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    /// Select which voice is controlled by pots/encoders
    SelectVoice(u8),

    /// Toggle voice on/off
    /// Active state changes, but volume remains unchanged
    ToggleVoice(u8),

    /// Set frequency of currently selected voice (Hz)
    /// Only applies if a voice is selected (Some(n))
    SetFrequency(f32),

    /// Set volume of currently selected voice (0.0 to 1.0)
    /// Only applies if a voice is selected (Some(n))
    SetVolume(f32),
}
