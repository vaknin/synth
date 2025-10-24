//! Message types for lock-free communication between control tasks and audio task.

use crate::config::WAVEFORM_BUFFER_SIZE;

/// Messages sent from control tasks (pots) to audio task.
/// Simplified for drone synth with 2 pots only (cutoff, resonance).
#[derive(Debug, Clone, Copy)]
pub enum Message {
    /// Set filter cutoff frequency (Hz, mapped from pot)
    SetCutoff(f32),

    /// Set filter resonance (0.0-10.0, mapped from pot)
    SetResonance(f32),
}

/// Waveform data for oscilloscope-style display visualization.
///
/// Contains 256 audio samples captured via decimation (every 11th sample at 44.1kHz).
/// Each buffer represents ~5.8ms of audio, sent from Engine (Core 0) to Display (Core 1).
///
/// Overlapping window strategy: Buffers are sent every 128 samples (50% overlap) to provide
/// smooth scrolling with phase continuity. The `write_index` field indicates where the next
/// sample will be written, allowing the display to extract the most recent 128 samples.
///
/// Transfer mechanism: heapless::spsc::Queue (lock-free, atomic-only, ~15 cycles per operation)
#[derive(Clone, Copy)]
pub struct WaveformData {
    /// Circular buffer of audio samples (newest samples before write_index)
    pub samples: [i16; WAVEFORM_BUFFER_SIZE],

    /// Write index (0-255): position where next sample will be written.
    /// Most recent 128 samples are at indices [(write_index - 128)..write_index] (with wraparound)
    pub write_index: usize,
}
