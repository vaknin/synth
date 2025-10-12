#![no_std]

use embassy_sync::channel::{Channel, Receiver};
use esp_hal::i2s::master::asynch::I2sWriteDmaTransferAsync;

use crate::{config::MESSAGE_QUEUE_SIZE, message::Message};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as ChannelMutex;

#[macro_use]
mod macros;

pub mod arpeggiator;
pub mod config;
pub mod controls;
pub mod engine;
pub mod filter;
pub mod hardware;
pub mod lfo;
pub mod message;
pub mod noise;
pub mod oscillator;
pub mod slew;

// Re-export paste for use in generated macros
#[doc(hidden)]
pub use paste;

pub type AudioStream = I2sWriteDmaTransferAsync<'static, &'static mut [u8]>;
pub type SynthChannel = Channel<ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;
pub type ChannelReceiver = Receiver<'static, ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;

// Computed once instead of per-sample
pub const I16_MAX_F32: f32 = i16::MAX as f32;