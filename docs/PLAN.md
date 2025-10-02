# Implementation Plan: Core Synth Architecture

## Overview
This plan implements the foundational architecture for the ESP32-S3 synthesizer:
- **3 independent voices** (configurable) with oscillators, volume, and active state
- **Lock-free message passing** using `heapless::spsc::Queue` for control → audio communication
- **Engine** that manages voices and processes control messages
- **Simple sequential message processing** (no deduplication for MVP simplicity)
- **Centralized configuration** to eliminate magic numbers

## Architecture Decisions
- `selected_voice: Option<u8>` - properly models "no voice selected" state
- `Voice` has both `active: bool` and `volume: f32` - toggle on/off without losing volume settings
- Sequential message processing - simpler than deduplication, good enough for 32-message queue
- Voices sum and normalize (divide by VOICE_COUNT) to prevent clipping
- Using **2 pots temporarily** instead of encoders (hardware constraint)

---

## Phase 1: Configuration & Dependencies

### 1.1 Add `heapless` Dependency
**File**: `Cargo.toml`

**Action**: Add to `[dependencies]` section:
```toml
heapless = "0.8"
```

**Why**: Provides `heapless::spsc::Queue` for lock-free message passing between control and audio tasks. Essential for real-time audio safety (no blocking).

---

### 1.2 Create Configuration Module
**File**: `src/config.rs` (new file)

**Action**: Create with all constants:
```rust
//! Centralized configuration constants for the synth.
//! All magic numbers should live here to ensure consistency.

/// Number of simultaneous voices
pub const VOICE_COUNT: usize = 3;

/// Audio sample rate in Hz
pub const SAMPLE_RATE: u32 = 44_100;

/// Starting frequency for all voices on initialization (Hz)
pub const STARTING_FREQUENCY: f32 = 77.0;

/// Default volume level for voices (0.0 to 1.0)
pub const DEFAULT_VOLUME: f32 = 0.5;

/// DMA circular buffer size in bytes
/// Must be divisible by 4 (stereo i16 frame size)
pub const DMA_BUFFER_SIZE: usize = 1024;

/// Message queue capacity (number of messages)
pub const MESSAGE_QUEUE_SIZE: usize = 32;
```

**Why**:
- Single source of truth for configuration
- Easy tuning without hunting through code
- Prevents inconsistencies (e.g., VOICE_COUNT mismatch between modules)
- Makes porting to different hardware easier

---

## Phase 2: Message System

### 2.1 Create Message Enum
**File**: `src/message.rs` (new file)

**Action**: Define message types for control → audio communication:
```rust
//! Message types for lock-free communication between control tasks and audio task.

/// Messages sent from control tasks (buttons, pots, encoders) to audio task.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    /// Select which voice is controlled by pots/encoders (0, 1, or 2)
    SelectVoice(u8),

    /// Toggle voice on/off (0, 1, or 2)
    /// Active state changes, but volume remains unchanged
    ToggleVoice(u8),

    /// Set frequency of currently selected voice (Hz)
    /// Only applies if a voice is selected (Some(n))
    SetFrequency(f32),

    /// Set volume of currently selected voice (0.0 to 1.0)
    /// Only applies if a voice is selected (Some(n))
    SetVolume(f32),
}
```

**Why**:
- `Copy` trait allows zero-copy message passing through queue
- `Debug` for logging/diagnostics
- Voice index (u8) instead of reference keeps messages self-contained
- SetFrequency/SetVolume operate on "selected voice" paradigm (UI simplicity)

**Processing Strategy** (Option B - Simple Sequential):
- Audio callback drains entire queue with `while let Some(msg) = queue.dequeue()`
- Each message processed immediately via `engine.process_message(msg)`
- Redundant updates are allowed (simpler code, negligible cost at 32 messages max)

---

## Phase 3: Voice Module

### 3.1 Create Voice Struct
**File**: `src/voice.rs` (new file)

**Action**: Implement Voice as oscillator wrapper with volume and active state:
```rust
//! Voice module: instrument instance with oscillator, volume, and active state.

use crate::oscillator::Oscillator;

/// A single voice in the synth.
/// Wraps an oscillator with volume control and active state.
pub struct Voice {
    /// Wavetable oscillator for audio generation
    osc: Oscillator,

    /// User-set volume (0.0 = silent, 1.0 = full scale)
    volume: f32,

    /// Whether voice is active (on) or inactive (off)
    /// When inactive, tick() returns 0.0 regardless of volume
    active: bool,
}

impl Voice {
    /// Create a new voice at specified frequency.
    ///
    /// # Arguments
    /// * `frequency` - Initial frequency in Hz
    /// * `sample_rate` - Audio sample rate in Hz
    ///
    /// # Returns
    /// Voice with specified frequency, DEFAULT_VOLUME, inactive state
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        Self {
            osc: Oscillator::new(frequency, sample_rate),
            volume: crate::config::DEFAULT_VOLUME,
            active: false,
        }
    }

    /// Set voice frequency in Hz.
    pub fn set_frequency(&mut self, freq: f32) {
        self.osc.set_frequency(freq);
    }

    /// Set voice volume (0.0 to 1.0).
    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
    }

    /// Set voice active state.
    /// true = voice plays, false = voice silent (but retains frequency/volume)
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Generate next audio sample.
    ///
    /// # Returns
    /// Audio sample (-1.0 to 1.0) scaled by volume, or 0.0 if inactive
    pub fn tick(&mut self) -> f32 {
        if self.active {
            self.osc.tick() * self.volume
        } else {
            0.0
        }
    }
}
```

**Why**:
- **Separation of concerns**: Voice adds instrument-level logic (volume, active), Oscillator stays pure signal generator
- **Active field**: Allows toggling on/off without losing volume setting (user experience)
- **Clamp on volume**: Safety against invalid values from ADC noise or bugs
- **Zero output when inactive**: Skips oscillator computation (minor optimization, clearer semantics)

---

## Phase 4: Engine Module

### 4.1 Create Engine Struct
**File**: `src/engine.rs` (new file)

**Action**: Implement synth engine managing 3 voices:
```rust
//! Engine module: manages voices, processes messages, renders audio.

use crate::config::{VOICE_COUNT, STARTING_FREQUENCY, DEFAULT_VOLUME};
use crate::message::Message;
use crate::voice::Voice;

/// Main synth engine managing all voices.
pub struct Engine {
    /// Array of voices (size determined by VOICE_COUNT config)
    voices: [Voice; VOICE_COUNT],

    /// Currently selected voice for control (None = no selection)
    selected_voice: Option<u8>,

    /// Audio sample rate (stored for future use)
    sample_rate: f32,
}

impl Engine {
    /// Create new engine with initialized voices.
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    ///
    /// # Returns
    /// Engine with VOICE_COUNT voices at STARTING_FREQUENCY, inactive, no selection
    pub fn new(sample_rate: f32) -> Self {
        Self {
            voices: core::array::from_fn(|_| {
                Voice::new(STARTING_FREQUENCY, sample_rate)
            }),
            selected_voice: None,
            sample_rate,
        }
    }

    /// Process a single control message.
    ///
    /// # Arguments
    /// * `msg` - Message from control task (buttons, pots, encoders)
    pub fn process_message(&mut self, msg: Message) {
        match msg {
            Message::SelectVoice(idx) => {
                if (idx as usize) < VOICE_COUNT {
                    self.selected_voice = Some(idx);
                }
            }

            Message::ToggleVoice(idx) => {
                if let Some(voice) = self.voices.get_mut(idx as usize) {
                    voice.set_active(!voice.active);
                }
            }

            Message::SetFrequency(freq) => {
                if let Some(idx) = self.selected_voice {
                    if let Some(voice) = self.voices.get_mut(idx as usize) {
                        voice.set_frequency(freq);
                    }
                }
            }

            Message::SetVolume(vol) => {
                if let Some(idx) = self.selected_voice {
                    if let Some(voice) = self.voices.get_mut(idx as usize) {
                        voice.set_volume(vol);
                    }
                }
            }
        }
    }

    /// Generate next mixed audio sample from all voices.
    ///
    /// # Returns
    /// Sum of all active voices normalized by VOICE_COUNT to prevent clipping
    pub fn tick(&mut self) -> f32 {
        let sum: f32 = self.voices.iter_mut().map(|v| v.tick()).sum();
        sum / VOICE_COUNT as f32
    }
}
```

**Why**:
- **`Option<u8>` for selected_voice**: Explicitly models "no selection" state (safer than default 0)
- **Bounds checking**: Validates voice indices before accessing array (prevents panics from corrupt messages)
- **SetFrequency/SetVolume check selected_voice**: No-op if nothing selected (graceful)
- **Normalization by VOICE_COUNT**: Prevents clipping when all 3 voices active at full volume (3.0 → 1.0)
- **`core::array::from_fn`**: Clean voice initialization without manual indexing

---

## Phase 5: Integration

### 5.1 Update Library Exports
**File**: `src/lib.rs`

**Action**: Add new module declarations:
```rust
#![no_std]

pub mod audio_util;
pub mod config;
pub mod engine;
pub mod message;
pub mod oscillator;
pub mod voice;
```

**Why**: Make modules accessible to binary crate (`src/bin/main.rs`)

---

### 5.2 Update Main Binary
**File**: `src/bin/main.rs`

**Changes**:

#### 5.2.1 Update Imports
```rust
use heapless::spsc::Queue;
use synth::{
    audio_util::f32_to_i16_le,
    config::*,
    engine::Engine,
    message::Message,
};
```

#### 5.2.2 Replace Hardcoded Constants
```rust
// OLD:
const SAMPLE_RATE: u32 = 44_100;
const FREQUENCY: f32 = 30.0;

// NEW:
// Use config::SAMPLE_RATE instead (imported via use config::*)
```

#### 5.2.3 Create Message Queue and Engine
```rust
// In main() after peripherals init:

// Create SPSC queue for control → audio messages
static mut MESSAGE_QUEUE: Queue<Message, MESSAGE_QUEUE_SIZE> = Queue::new();
let (mut producer, mut consumer) = unsafe { MESSAGE_QUEUE.split() };

// Create synth engine
let mut engine = Engine::new(SAMPLE_RATE as f32);
```

#### 5.2.4 Add Temporary ADC Setup for 2 Pots
```rust
// TODO: Replace with actual ADC pins (avoiding GPIO 7/8/9 used by I2S)
// Example placeholder:
// let adc1 = peripherals.ADC1;
// let pot_freq_pin = peripherals.GPIO1.into_analog(); // placeholder
// let pot_vol_pin = peripherals.GPIO2.into_analog();  // placeholder
// let mut adc_config = AdcConfig::new();
// let mut adc = Adc::new(adc1, adc_config);

// For now, we'll just process messages from queue (no ADC reads yet)
```

#### 5.2.5 Update Audio Callback Loop
```rust
loop {
    let _result = transaction.push_with(|buffer| {
        // Verify buffer meets minimum stereo frame size
        if buffer.len() < 4 {
            return 0;
        }

        // --- MESSAGE PROCESSING (NEW) ---
        // Drain all pending messages from queue and process sequentially
        while let Some(msg) = consumer.dequeue() {
            engine.process_message(msg);
        }

        // --- AUDIO GENERATION (UPDATED) ---
        let remainder = buffer.len() % 4;
        let mut chunks = buffer.chunks_exact_mut(4);

        for chunk in &mut chunks {
            // Generate mixed sample from all voices
            let sample = engine.tick();

            // Convert to i16 little-endian bytes
            let bytes = f32_to_i16_le(sample);

            // Write stereo (duplicate mono)
            chunk[0] = bytes[0];
            chunk[1] = bytes[1];
            chunk[2] = bytes[0];
            chunk[3] = bytes[1];
        }

        buffer.len() - remainder
    }).await;
}
```

**Why**:
- **Message draining before audio**: Ensures parameter changes apply immediately to next buffer
- **Sequential processing**: Simple, correct, fast enough (32 messages << 1024 samples)
- **Engine owns state**: Audio callback is now thin - just message processing + rendering
- **ADC placeholder**: Acknowledges hardware constraint, easy to add later

---

## Testing Strategy

### Manual Testing Checklist:
1. **Build & Flash**: Verify code compiles and runs without panics
2. **Audio Output**: Confirm silent output (all voices inactive by default)
3. **Message Injection** (via future control task):
   - `SelectVoice(0)` → select voice 0
   - `SetFrequency(220.0)` → set to 220Hz
   - `SetVolume(0.5)` → set to 50% volume
   - `ToggleVoice(0)` → voice starts playing
   - Verify 220Hz tone at reasonable volume
4. **Multi-voice Test**:
   - Activate all 3 voices at different frequencies (220, 440, 660 Hz)
   - Verify chord without clipping (normalization works)
5. **Edge Cases**:
   - Send SetFrequency/SetVolume with no selected voice → no crash
   - Send invalid voice index (e.g., 5) → no crash

---

## Future Work (Post-MVP)
- Add ADC reading for 2 pots (frequency + volume control)
- Add button handling for voice selection/toggle (GPIO + debouncing)
- Add encoder support (replace pots with PCNT peripheral)
- Implement envelopes (attack/release for click-free toggling)
- Add filters (low-pass, high-pass)
- Add effects (drive, delay, reverb)

---

## Summary

This implementation establishes the **foundational architecture**:
- ✅ Lock-free message passing (real-time safe)
- ✅ Modular design (Oscillator → Voice → Engine)
- ✅ Configurable via constants (easy tuning)
- ✅ Simple and correct (Option B sequential processing)
- ✅ Ready for control input (queue producer available for future tasks)

**Result**: A working 3-voice synth that responds to messages and outputs normalized audio, ready for UI integration.
