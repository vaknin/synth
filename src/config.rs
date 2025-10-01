//! Centralized configuration constants for the synth.
//! All magic numbers should live here to ensure consistency.

/// Number of simultaneous voices
pub const VOICE_COUNT: usize = 3;

/// Audio sample rate in Hz
pub const SAMPLE_RATE: u32 = 44_100;

/// Starting frequency for all voices on initialization (Hz)
pub const STARTING_FREQUENCY: f32 = 77.0;

/// Default volume level for voices (0.0 to 1.0)
pub const DEFAULT_VOLUME: f32 = 0.5;

/// DMA circular buffer size in bytes
/// Must be divisible by 4 (stereo i16 frame size)
pub const DMA_BUFFER_SIZE: usize = 2040;

/// Message queue capacity (number of messages)
pub const MESSAGE_QUEUE_SIZE: usize = 32;
