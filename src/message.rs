//! Message types for lock-free communication between control tasks and audio task.

/// Messages sent from control tasks (pots, encoders) to audio task.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    // === FILTER PARAMETERS ===
    /// Set filter cutoff frequency (Hz, mapped from pot)
    SetCutoff(f32),

    /// Set filter resonance (0.0-4.0, mapped from pot)
    SetResonance(f32),

    /// Set filter cutoff LFO rate in Hz (0.01-10.0 Hz exponential)
    SetFilterLfoRate(f32),

    /// Set filter cutoff LFO depth (0.0-1.0)
    SetFilterLfoDepth(f32),

    /// Set filter envelope depth (0.0-1.0, controls rhythm-synced filter pluck intensity)
    SetFilterEnvelopeDepth(f32),

    // === GENERATIVE PARAMETERS ===
    /// Set global tempo in BPM (20-220)
    SetTempo(f32),

    /// Set lead Euclidean hits (0 = mute, length fixed at 16)
    SetLeadEuclideanHits(usize),

    /// Set musical scale for quantizer (0-11 for 12 scales)
    SetScale(usize),

    /// Set portamento smoothing coefficient (0.95-0.998, shared for all voices)
    SetPortamento(f32),

    /// Set envelope attack time in milliseconds (lead voice only)
    SetAttack(f32),

    /// Set envelope decay time in milliseconds (lead voice only)
    SetDecay(f32),
}
