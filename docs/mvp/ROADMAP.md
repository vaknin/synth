# Turing Machine + Euclidean Rhythm Roadmap

**Vision**: Pocket-sized Aphex Twin machine. Evolving melodies (Turing Machine) + polyrhythmic patterns (Euclidean rhythms). You perform filter sweeps while algorithms create the music.

**Current State**:
- Phases 1-3 complete (see ARCHITECTURE.md)
- 4 oscillators, Moog filter working
- 6 pots wired, 6 more available (12 total)
- 10+ pots arriving from AliExpress

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
- [ ] Create `src/quantizer.rs` - Scale quantization (reuse from current plan)
- [ ] Add tempo control (80-180 BPM for live energy control)
- [ ] Add saturation/distortion (soft clipping for PHAT bass)
- [ ] Test basic polyrhythms (E(3,8) + E(5,13))

**Pot Allocation (First 6)**:
- Pot 0-1: Filter Cutoff/Resonance (keep existing)
- Pot 2: Bass Pattern Length
- Pot 3: Bass Hit Density
- Pot 4: Lead Pattern Length
- Pot 5: Lead Hit Density

---

### Phase 2: Turing Machine Integration (Weekend 2)
**Goal**: Add evolving melodies via Turing Machine

**Tasks**:
- [ ] Implement mutation logic (bit flipping with probability)
- [ ] Add scale quantization to Turing output
- [ ] Create smooth glide between notes
- [ ] Add randomized swing (10-25% variation for human groove)
- [ ] Wire remaining 6 pots

**Additional Pots (6-11)**:
- Pot 6-7: Bass/Lead Mutation Rate
- Pot 8-9: Bass/Lead Loop Length
- Pot 10: Tempo (80-180 BPM)
- Pot 11: Saturation Drive

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

## Control Mapping (12 Pots)

```
RHYTHM (4 pots):
0. Bass Pattern Length (4-16 steps)
1. Bass Hit Density (2-15 hits)
2. Lead Pattern Length (3-17 steps - offset for polyrhythm)
3. Lead Hit Density (2-16 hits)

MELODY (4 pots):
4. Bass Mutation Rate (0-20%)
5. Bass Loop Length (8-32 bits)
6. Lead Mutation Rate (0-50%)
7. Lead Loop Length (8-32 bits)

PERFORMANCE (4 pots):
8. Filter Cutoff (main performance control)
9. Filter Resonance (Moog scream)
10. Tempo (80-180 BPM for live energy)
11. Saturation Drive (0-100% for PHAT bass)

HARDCODED:
- Master volume: 0.75 (safe level)
- Scale: Minor pentatonic
- Root notes: C2 (bass), C4 (lead)
- Glide: 150ms (bass), 50ms (lead)
- Swing: Randomized 10-25% per beat
- Delay: OFF (Phase 3 if desired)
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