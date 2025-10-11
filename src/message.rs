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

    // === GENERATIVE PARAMETERS ===
    /// Set global tempo in BPM
    SetTempo(f32),

    /// Set bass Euclidean hits (0 = mute, length fixed)
    SetBassEuclideanHits(usize),

    /// Set lead Euclidean pattern length in steps
    SetLeadEuclideanLength(usize),

    /// Set lead Euclidean hits (0 = mute)
    SetLeadEuclideanHits(usize),

    /// Set Turing Machine mutation rate (shared for both voices)
    SetMutationRate(f32),

    /// Set portamento smoothing coefficient (0.95-0.998, shared for all voices)
    SetPortamento(f32),
}
