//! Message types for lock-free communication between control tasks and audio task.

/// Oscillator identifier for control routing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OscillatorId {
    /// Oscillator 1 (Sine wave)
    Osc1 = 0,
    /// Oscillator 2 (Sawtooth wave)
    Osc2 = 1,
    /// Oscillator 3 (Square wave)
    Osc3 = 2,
    /// Sub-oscillator (Sine at -1 octave)
    Sub = 3,
}

/// Arpeggiator playback mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArpMode {
    /// Play chord notes ascending (C → E → G → C)
    Up = 0,
    /// Play chord notes descending (C → G → E → C)
    Down = 1,
    /// Play ascending then descending (C → E → G → E → C)
    UpDown = 2,
    /// Random note selection from chord
    Random = 3,
}

/// Messages sent from control tasks (buttons, pots, encoders) to audio task.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    /// Set master pitch for all oscillators (30Hz-1024Hz, exponential mapping)
    SetMasterPitch(f32),

    /// Set level for specific oscillator (0.0-1.0)
    SetOscLevel(OscillatorId, f32),

    /// Set detune for specific oscillator (-50¢ to +50¢)
    /// Note: Only Osc2, Osc3, and Sub support detuning
    SetOscDetune(OscillatorId, f32),

    /// Set filter cutoff frequency (Hz, mapped from pot)
    SetCutoff(f32),

    /// Set filter resonance (0.0-10.0, mapped from pot)
    SetResonance(f32),

    /// Set filter cutoff LFO rate in Hz (0.01-10.0 Hz exponential)
    SetFilterLfoRate(f32),

    /// Set filter cutoff LFO depth (0.0-1.0)
    SetFilterLfoDepth(f32),

    /// Set arpeggiator clock rate (1-20 Hz)
    SetArpRate(f32),

    /// Set arpeggiator playback mode (Up/Down/UpDown/Random)
    SetArpMode(ArpMode),

    /// Set arpeggiator root transpose in semitones (±12 semitones)
    SetArpTranspose(i8),

    /// Set arpeggiator chord pattern (0-7, for future expansion)
    SetArpChord(usize),
}
