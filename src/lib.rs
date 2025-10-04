#![no_std]

use embassy_sync::channel::{Channel, Receiver};
use esp_hal::i2s::master::asynch::I2sWriteDmaTransferAsync;

use crate::{config::MESSAGE_QUEUE_SIZE, message::Message};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as ChannelMutex;

pub mod config;
pub mod controls;
pub mod engine;
pub mod hardware;
pub mod message;
pub mod oscillator;
pub mod smoother;
pub mod voice;

pub type AudioStream = I2sWriteDmaTransferAsync<'static, &'static mut [u8]>;
pub type SynthChannel = Channel<ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;
pub type ChannelReceiver = Receiver<'static, ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;

// Computed once instead of per-sample
pub const I16_MAX_F32: f32 = i16::MAX as f32;