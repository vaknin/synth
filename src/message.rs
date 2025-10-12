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

    /// Set bass Euclidean hits (0 = mute, length fixed at 8)
    SetBassEuclideanHits(usize),

    /// Set lead Euclidean hits (0 = mute, length fixed at 16)
    SetLeadEuclideanHits(usize),

    /// Set musical scale for quantizer (0-11 for 12 scales)
    SetScale(usize),

    /// Set portamento smoothing coefficient (0.95-0.998, shared for all voices)
    SetPortamento(f32),

    /// Set filter envelope depth (0.0-1.0, controls rhythm-synced filter pluck intensity)
    SetFilterEnvelopeDepth(f32),

    /// Set envelope attack time in milliseconds (shared for all voices)
    SetAttack(f32),

    /// Set envelope decay time in milliseconds (shared for all voices)
    SetDecay(f32),
}
