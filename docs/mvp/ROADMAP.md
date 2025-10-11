# Turing Machine + Euclidean Rhythm Roadmap

**Vision**: Pocket-sized Aphex Twin machine. Evolving melodies (Turing Machine) + polyrhythmic patterns (Euclidean rhythms). You perform filter sweeps while algorithms create the music.

**Current State**:
- Phase 1: ✅ COMPLETE (hardware testing in progress)
- Phase 2: ✅ MOSTLY COMPLETE (swing timing deferred)
- Phase 3: ⏸️ NOT STARTED (delay effects)
- 4 oscillators (2 per voice), Moog filter, LFOs working
- 8 pots wired and functional
- Generative system operational: Euclidean rhythms + Turing Machine + Quantizer

**Target Sound**: Fat techno bass + glitchy melodic lead with polyrhythmic complexity

---

## Core Concept

**Euclidean Rhythms**: Mathematical pattern distribution. E(k,n) = k hits across n steps, evenly spaced.
- E(3,8) = [x . . x . . x .] = Tresillo (reggaeton)
- E(5,12) = Shifted techno bassline
- Layer different patterns = instant polyrhythm

**Turing Machine**: 16-bit shift register with mutation probability.
- Low mutation (5%) = slowly evolving melody
- High mutation (50%) = rapid variation
- Quantized to scales = always musical

**Your Performance**: Control macro parameters (filter, resonance, mutation rate) while algorithms handle the notes.

---

## Signal Chain

```
RHYTHM GENERATION (per voice):
Euclidean Pattern → Gate Events

MELODY GENERATION (per voice):
Turing Machine → Scale Quantizer → Pitch

SYNTHESIS:
Oscillators → Envelope → Moog Filter → Delay → Output
```

---

## Implementation Phases

### Phase 1: Core Rhythm Engine (Weekend 1)
**Goal**: Implement Euclidean rhythm generator with two independent voices

**Tasks**:
- [x] Create `src/euclidean.rs` - Bjorklund algorithm (COMPLETE, verified with Python)
- [x] Create `src/turing.rs` - Shift register with mutation (COMPLETE, verified with Python)
- [x] Create `src/quantizer.rs` - Scale quantization with 12 scales + optimized lookup tables
- [x] Add tempo control (80-180 BPM) - `src/clock.rs` with smooth slew transitions
- [x] Add saturation/distortion - `soft_clip()` function in `src/engine.rs`
- [~] Test basic polyrhythms (E(3,8) + E(5,13)) - **IN PROGRESS (hardware testing)**

**Pot Allocation (8 pots wired)**:
- Pot 0: Filter Cutoff
- Pot 1: Filter Resonance
- Pot 2: Bass Pattern Length (4-16 steps)
- Pot 3: Bass Hit Density (1-16 hits)
- Pot 4: Lead Pattern Length (3-17 steps)
- Pot 5: Lead Hit Density (1-17 hits)
- Pot 6: Tempo (80-180 BPM)
- Pot 7: Mutation Rate (0-50%, affects both voices)

---

### Phase 2: Turing Machine Integration (Weekend 2)
**Goal**: Add evolving melodies via Turing Machine

**Tasks**:
- [x] Implement mutation logic (bit flipping with probability) - `src/turing.rs`
- [x] Add scale quantization to Turing output - `src/quantizer.rs` (12 scales)
- [x] Create smooth glide between notes - Oscillator pitch smoothing
- [x] Integrate into Voice abstraction - `src/voice.rs` (Euclidean + Turing + Quantizer + Dual Osc + Envelope)
- [ ] Add randomized swing (10-25% variation for human groove)
- [x] Wire 8 pots (Pot 0-7) - See Phase 1 pot allocation

**Status**: Phase 2 essentially complete (swing timing feature deferred)

**Note**: Voice architecture now encapsulates all generative components per-voice, cleaner than original roadmap design.

---

### Phase 3: Delay & Polish (Weekend 3)
**Goal**: Add delay for pseudo-polyphony and refine performance

**Tasks**:
- [ ] Implement PSRAM delay buffer
- [ ] Add tempo-synced delay (dotted eighth, quarter)
- [ ] Performance optimizations
- [ ] Create preset starting points

**Delay Controls** (if pots available):
- Delay Time (tempo-synced divisions)
- Delay Feedback (hardcode at 40% if no pot)

---

## Control Mapping

### Current Configuration (8 Pots - Wired)

```
PERFORMANCE (2 pots):
0. Filter Cutoff (main performance control, 20 Hz - 20 kHz)
1. Filter Resonance (0.0 - 0.95, Moog scream at high values)

RHYTHM GENERATION (4 pots):
2. Bass Pattern Length (4-16 steps, Euclidean parameter n)
3. Bass Hit Density (1-16 hits, Euclidean parameter k)
4. Lead Pattern Length (3-17 steps, offset for polyrhythm)
5. Lead Hit Density (1-17 hits, creates pattern complexity)

GLOBAL CONTROLS (2 pots):
6. Tempo (80-180 BPM, affects all voices)
7. Mutation Rate (0-50%, applies to both Turing Machines)
```

### Future Expansion (4+ Pots Available)

```
POTENTIAL ADDITIONS:
- Per-voice mutation rates (separate bass/lead control)
- Turing Machine loop length (8-32 bits per voice)
- Saturation drive control (currently soft-clipping at fixed level)
- Scale selection (currently hardcoded to Natural Minor)
- LFO rate/depth for filter modulation
- Delay controls (if Phase 3 implemented)
```

### Hardcoded Parameters

```
AUDIO:
- Master volume: 0.75 (safe level)
- Saturation: Soft-clipping enabled (rational function approximation)
- Sample rate: 44.1 kHz

GENERATIVE:
- Scale: Natural Minor (Aeolian, index 1 in quantizer)
- Root notes: 65 Hz (C2 bass), 260 Hz (C4 lead)
- Octave range: 2.0 (bass), 3.0 (lead)
- Glide: Built into oscillator pitch smoothing
- Swing: Not implemented yet

SYNTHESIS:
- Oscillators per voice: 2 (detuned +5¢ for thickness)
- Waveforms: Saw (bass), Square (lead)
- Envelope: AD (attack-decay) per voice
- Delay: OFF (Phase 3 feature)
```

---

## Musical Examples

**Deep Techno**:
- Bass: E(3,8) at low mutation (5%)
- Lead: OFF or E(5,13) at medium mutation
- Filter: Slow sweeps with high resonance

**Aphex Twin Glitch**:
- Bass: E(5,12) at 5% mutation
- Lead: E(7,13) at 30% mutation
- Result: 12:13 polyrhythm with evolving melodies

**Ambient Exploration**:
- Bass: E(2,16) at 15% mutation (sparse, evolving)
- Lead: E(3,11) at 40% mutation
- Filter: Wide open, heavy delay

---

## Success Metrics

**Phase 1**: Polyrhythms feel musical, not random
**Phase 2**: Melodies evolve organically over 5+ minutes
**Phase 3**: Can perform for 30+ minutes without boredom

---

## Why This Works

- **Euclidean** = Instant groove (not random, mathematically musical)
- **Turing Machine** = Evolving melodies (not static loops)
- **Moog Filter** = Fat analog character (already working)
- **12 Pots** = Perfect amount of control without menu diving
- **No screens** = Immediate, tactile, performative

This is your pocket-sized Analord machine. Let's build it. 🎛️