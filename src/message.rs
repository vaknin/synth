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
/// Contains 1024 audio samples captured via decimation (every 50th sample at 44.1kHz).
/// Sent from Engine (Core 0) to Display (Core 1) via Embassy Signal (latest-value semantics).
///
/// Display maintains its own read pointer for smooth time-based scrolling independent of
/// audio engine write position. Signal provides complete buffer snapshot on every update.
///
/// Transfer mechanism: Embassy Signal (lock-free, atomic-only, overwrites if not consumed)
#[derive(Clone, Copy)]
pub struct WaveformData {
    /// Circular buffer of audio samples
    pub samples: [i16; WAVEFORM_BUFFER_SIZE],
}
