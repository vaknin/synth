//! Control input handling: potentiometers and future encoders.
//!
//! This module uses Embassy channels for lock-free, multi-producer messaging.
//! Each control input (pot, encoder) is an independent async task
//! that sends messages to the audio engine via a shared channel.

pub mod pot;
pub mod task;

// Re-export commonly used items
pub use pot::{
    map_attack, map_bass_euclidean_hits, map_cutoff, map_decay, map_filter_envelope_depth,
    map_lead_euclidean_hits, map_lfo_depth, map_lfo_rate, map_master_pitch, map_osc_detune,
    map_osc_level, map_portamento, map_resonance, map_scale, map_tempo, Potentiometer,
};
pub use task::pot_task;

use crate::config::MESSAGE_QUEUE_SIZE;
use crate::message::Message;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as ChannelMutex;
use embassy_sync::channel::Sender;

/// Type alias for control message sender (used across all control tasks)
pub type CtrlSender = Sender<'static, ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;
