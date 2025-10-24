#![no_std]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as ChannelMutex;
use embassy_sync::channel::{Channel, Receiver};
use esp_hal::i2s::master::asynch::I2sWriteDmaTransferAsync;
use heapless::spsc::{Consumer, Producer, Queue};

use crate::{
    config::{MESSAGE_QUEUE_SIZE, WAVEFORM_QUEUE_CAPACITY},
    message::{Message, WaveformData},
};

#[macro_use]
mod macros;

pub mod config;
pub mod controls;
pub mod display;
pub mod engine;
pub mod filter;
pub mod hardware;
pub mod lfo;
pub mod message;
pub mod oscillator;
pub mod slew;

// Re-export paste for use in generated macros
#[doc(hidden)]
pub use paste;

pub type AudioStream = I2sWriteDmaTransferAsync<'static, &'static mut [u8]>;
pub type SynthChannel = Channel<ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;
pub type ChannelReceiver = Receiver<'static, ChannelMutex, Message, MESSAGE_QUEUE_SIZE>;

// Waveform SPSC queue types (Engine → Display communication)
pub type WaveformQueue = Queue<WaveformData, WAVEFORM_QUEUE_CAPACITY>;
pub type WaveformProducer = Producer<'static, WaveformData>;
pub type WaveformConsumer = Consumer<'static, WaveformData>;

// Computed once instead of per-sample
pub const I16_MAX_F32: f32 = i16::MAX as f32;
