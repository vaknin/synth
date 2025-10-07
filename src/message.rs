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

/// Filter type identifier for multimode filter control.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    /// Lowpass filter output
    Lowpass = 0,
    /// Highpass filter output
    Highpass = 1,
    /// Bandpass filter output
    Bandpass = 2,
}

/// Messages sent from control tasks (buttons, pots, encoders) to audio task.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    // === Phase 1: Oscillator Controls ===

    /// Set master pitch for all oscillators (30Hz-1024Hz, exponential mapping)
    SetMasterPitch(f32),

    /// Set level for specific oscillator (0.0-1.0)
    SetOscLevel(OscillatorId, f32),

    /// Set detune for specific oscillator (-50¢ to +50¢)
    /// Note: Only Osc2, Osc3, and Sub support detuning
    SetOscDetune(OscillatorId, f32),

    // === Phase 2: Filter Controls ===

    /// Set filter cutoff frequency (normalized 0.0-1.0)
    SetCutoff(f32),

    /// Set filter resonance/Q (normalized 0.0-1.0)
    SetResonance(f32),

    /// Set output level for specific filter type (0.0-1.0)
    SetFilterLevel(FilterType, f32),
}
