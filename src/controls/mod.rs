//! Control input handling: potentiometers and future encoders.
//!
//! This module uses Embassy channels for lock-free, multi-producer messaging.
//! Each control input (pot, encoder) is an independent async task
//! that sends messages to the audio engine via a shared channel.

pub mod pot;
pub mod task;

// Re-export commonly used items
pub use pot::{map_cutoff, map_master_pitch, map_osc_detune, map_osc_level, map_resonance, Potentiometer};
pub use task::pot_task;

use crate::config::MESSAGE_QUEUE_SIZE;
use crate::message::Message;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as ChannelMutex;
use embassy_sync::channel::Sender;

/// Type alias for control message sender (used across all control tasks)
pub type CtrlSender = Sender<'static, ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;
