# State Broadcasting & UI Layer Refactor

## Executive Summary

This document outlines the architectural addition of a **state broadcasting system** to enable real-time UI feedback (LEDs, future LCD visualizer) while maintaining audio-critical performance isolation on the ESP32-S3's dual-core architecture.

**Core Principle**: Audio rendering (Core 0) never waits on GPIO/UI operations (Core 1).

---

## Problem Statement

### Current Architecture
- **Single-core operation**: All tasks run on Core 0 (PRO core)
- **One-way communication**: Controls → Engine via MPSC
- **No visual feedback**: LED/button states not synchronized with voice state
- **Incomplete selection logic**: Voice selection not fully implemented

### Requirements
1. Button press selects voice → LED lights solid
2. Active but unselected voice → LED blinks (e.g., 50ms period)
3. Inactive voice → LED off
4. Future: LCD visualizer showing freq/vol/waveform data
5. Zero audio interference (no jitter, clicks, or dropouts)

---

## Architectural Solution

### Dual-Core Task Distribution

```
┌─────────────────────────────────────────────────────┐
│ CORE 0 (PRO) - Audio Priority                       │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────┐  MPSC   ┌────────┐  I2S DMA          │
│  │ Controls ├────────→│ Engine ├─────────→ Audio   │
│  └──────────┘         └───┬────┘           Out     │
│   (buttons/pots)          │                         │
│                           │ PubSub broadcast        │
└───────────────────────────┼─────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────┐
│ CORE 1 (APP) - UI/Visualization                     │
├─────────────────────────────────────────────────────┤
│                                                      │
│         ┌───────────────────────┐                   │
│         │ State Subscriber      │                   │
│         │ (Cloned to all tasks) │                   │
│         └───────────────────────┘                   │
│              ↓       ↓       ↓                       │
│          LED 0    LED 1    LED 2   (Future: LCD)    │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Why Two Channels?**
- **MPSC** (Control → Engine): Commands, many producers → one consumer
- **PubSub** (Engine → UI): State snapshots, one producer → many consumers

They serve orthogonal purposes and work in parallel.

---

## Technical Implementation

### Phase 1: Core Infrastructure

#### 1.1 State Structs (`src/state.rs`)

```rust
//! State broadcasting types for UI synchronization

use crate::config::VOICE_COUNT;

/// Snapshot of a single voice's complete state
#[derive(Clone, Copy, Debug)]
pub struct VoiceState {
    /// Voice index (0-2)
    pub id: u8,

    /// Whether voice is producing sound
    pub active: bool,

    /// Whether voice is selected for pot/encoder control
    pub selected: bool,

    /// Current frequency in Hz
    pub frequency: f32,

    /// Current volume (0.0-1.0)
    pub volume: f32,
}

/// Complete engine state snapshot for UI rendering
#[derive(Clone, Copy, Debug)]
pub struct EngineState {
    /// All voice states
    pub voices: [VoiceState; VOICE_COUNT],

    // Future additions (Phase 2):
    // pub peak_level: f32,
    // pub waveform_buffer: &'static [i16],
    // pub spectrum_bins: [f32; 16],
}
```

**Design Rationale**:
- `Copy` trait: Fast stack copying for broadcast (no heap allocation)
- Complete snapshots: UI tasks receive full context, not deltas
- Extensible: Add fields without breaking existing consumers

#### 1.2 PubSub Channel Setup (`src/lib.rs`)

```rust
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as ChannelMutex;

use crate::state::EngineState;

// Type aliases to avoid generics in task signatures (embassy constraint)
pub type StatePublisher = Publisher<'static, ChannelMutex, EngineState, 4, 4, 1>;
pub type StateSubscriber = Subscriber<'static, ChannelMutex, EngineState, 4, 4, 1>;
pub type StatePubSubChannel = PubSubChannel<ChannelMutex, EngineState, 4, 4, 1>;
//                                                                      ^  ^  ^
//                                                             capacity ┘  │  │
//                                                       max_subscribers ───┘  │
//                                                         max_publishers ──────┘
```

**Parameters Explained**:
- **Capacity 4**: Buffer up to 4 state updates (UI slower than engine = drop old states, keep latest)
- **Max subscribers 4**: 3 LED tasks + 1 LCD task (room for growth)
- **Max publishers 1**: Only Engine broadcasts state

**Embassy Task Constraint**: Tasks cannot be generic, so we use type aliases to expose concrete types.

#### 1.3 Engine Modifications (`src/engine.rs`)

**Add fields**:
```rust
pub struct Engine {
    // ... existing fields ...

    /// State broadcaster for UI tasks
    state_publisher: StatePublisher,
}
```

**Add state extraction method**:
```rust
impl Engine {
    fn get_current_state(&self) -> EngineState {
        EngineState {
            voices: core::array::from_fn(|i| VoiceState {
                id: i as u8,
                active: self.voices[i].active,
                selected: self.selected_voice == Some(i as u8),
                frequency: self.voices[i].osc.frequency,  // Direct field access
                volume: self.voices[i].volume.target(),   // Slew getter
            }),
        }
    }
}
```

**Broadcast on state changes** (called OUTSIDE audio hot path):
```rust
pub fn consume_messages(&mut self) -> bool {
    let mut state_changed = false;

    while let Ok(msg) = self.consumer.try_receive() {
        state_changed |= match msg {
            Message::SelectVoice(id) => {
                if id as usize >= VOICE_COUNT {
                    false  // Invalid - don't broadcast
                } else {
                    let changed = self.selected_voice != Some(id);
                    self.selected_voice = Some(id);
                    changed
                }
            }
            Message::ToggleVoice(idx) => {
                // ... existing logic ...
                true
            }
            Message::SetFrequency(freq) | Message::SetVolume(_) => {
                // ... existing logic ...
                true
            }
        };
    }

    state_changed
}
```

**Update `stream_audio()` to broadcast before hot path**:
```rust
pub async fn stream_audio(&mut self, audio_stream: &mut AudioStream) {
    // Process messages OUTSIDE hot path
    let state_changed = self.consume_messages();

    if state_changed {
        let state = self.get_current_state();
        self.state_publisher.publish_immediate(state);  // Non-blocking!
    }

    // NOW enter audio hot path (pure math, no atomics)
    audio_stream.push_with(|buffer| {
        for chunk in buffer.chunks_exact_mut(4) {
            let voices_sum: f32 = self.voices.iter_mut().map(|v| v.tick()).sum();
            // ... rest of rendering ...
        }
        buffer.len()
    }).await.ok();
}
```

**Critical**: Broadcasting happens in async context, NOT inside DMA callback. Zero impact on audio rendering.

#### 1.4 Expose State Fields (`src/voice.rs`, `src/oscillator.rs`, `src/slew.rs`)

Make internal state readable via public fields:

**`src/oscillator.rs`**:
```rust
pub struct Oscillator {
    pub frequency: f32,  // Direct read access for state broadcasting
    phase: f32,          // Private - internal DSP state
    phase_delta: f32,    // Private - computed from frequency
}
```

**`src/voice.rs`**:
```rust
pub struct Voice {
    pub osc: Oscillator,  // Direct access to oscillator state
    pub volume: Slew,     // Direct access to volume control
    pub active: bool,     // Already public
}
```

**`src/slew.rs`**:
```rust
impl Slew {
    pub fn target(&self) -> f32 {
        self.target  // Getter needed since target is private (smoothing state)
    }
}
```

**Rationale**: Public fields for simple reads, getter only where internal state needs encapsulation.

---

### Phase 2: Dual-Core Setup

#### 2.1 Enable Unstable Features (`Cargo.toml`)

Already present in `esp-hal` dependency:
```toml
esp-hal = { version = "=1.0.0-rc.0", features = [
  "esp32s3",
  "log-04",
  "unstable",  # ← Required for CpuControl
] }
```

#### 2.2 Multicore Main Function (`src/bin/main.rs`)

**Import multicore types**:
```rust
use esp_hal::system::{CpuControl, Stack};
use core::ptr::addr_of_mut;
```

**Declare static stack for Core 1** (must be 16-byte aligned, Xtensa ABI requirement):
```rust
// 16KB stack for UI tasks (provides headroom for future LCD/SPI operations)
static mut APP_CORE_STACK: Stack<16384> = Stack::new();
```

**Simplified main function** (hardware.rs does the heavy lifting):
```rust
#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let cpu_control = CpuControl::new(peripherals.CPU_CTRL);
    let dma_channel = peripherals.DMA_CH0;

    // Setup channels
    use synth::{StatePubSubChannel, SynthChannel};

    static STATE_CHANNEL: StatePubSubChannel = StatePubSubChannel::new();
    let state_publisher = STATE_CHANNEL.publisher().unwrap();
    let state_subscriber = STATE_CHANNEL.subscriber().unwrap();

    static CONTROL_CHANNEL: SynthChannel = Channel::new();
    let control_rx = CONTROL_CHANNEL.receiver();
    let control_tx = CONTROL_CHANNEL.sender();

    // DMA buffers
    let (_, _, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, DMA_BUFFER_SIZE);

    // Hardware module handles all multicore setup internally
    let (mut audio_stream, adc_bus, freq_pin, vol_pin) = synth::hardware::init_multicore_hardware(
        peripherals,
        cpu_control,
        state_subscriber,
        dma_channel,
        tx_buffer,
        tx_descriptors,
    );

    // Spawn control tasks on Core 0
    spawner.spawn(synth::controls::pot_task(
        control_tx,
        adc_bus,
        freq_pin,
        vol_pin
    )).unwrap();

    // Create engine with both channels
    let mut engine = Engine::new(control_rx, state_publisher);

    // Main audio loop (Core 0)
    loop {
        engine.stream_audio(&mut audio_stream).await;
    }
}
```

**Key Points**:
- Main function stays clean and focused on Core 0 logic
- Hardware module owns multicore complexity
- Core 1 automatically starts with UI tasks
- No peripheral splitting logic in main.rs

---

### Phase 3: Hardware Abstraction & Multicore Setup

#### 3.1 Hardware Module (`src/hardware.rs`)

Hardware module handles all peripheral splitting and multicore initialization internally:

```rust
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::system::{CpuControl, Stack};
use core::ptr::addr_of_mut;
use embassy_executor::Spawner;
use crate::StateSubscriber;

// Core 1 stack (16KB for LCD/SPI headroom)
static mut APP_CORE_STACK: Stack<16384> = Stack::new();

/// Initialize dual-core system with hardware split between cores.
///
/// Handles:
/// - Core 1 startup with UI executor
/// - LED GPIO initialization on Core 1
/// - Audio/control hardware on Core 0
///
/// # Arguments
/// * `peripherals` - Full peripheral set from esp_hal::init()
/// * `cpu_ctrl` - CPU control peripheral
/// * `state_subscriber` - Cloned subscriber for UI tasks
/// * `dma_channel` - DMA channel for audio
///
/// # Returns
/// Tuple of (audio_stream, adc_bus, freq_pin, vol_pin) for Core 0
pub fn init_multicore_hardware(
    peripherals: esp_hal::Peripherals,
    cpu_ctrl: CpuControl,
    state_subscriber: StateSubscriber,
    dma_channel: /* ... */,
) -> (AudioStream, /* ADC types */) {

    // Extract Core 1 peripherals before moving into closure
    let gpio10 = peripherals.GPIO10;
    let gpio11 = peripherals.GPIO11;
    let gpio12 = peripherals.GPIO12;

    // Start Core 1 with UI tasks
    let _core1_guard = cpu_ctrl.start_app_core(
        unsafe { &mut *addr_of_mut!(APP_CORE_STACK) },
        move || {
            // Setup LEDs on Core 1
            let (led0, led1, led2) = setup_leds_internal(gpio10, gpio11, gpio12);

            // Clone subscribers for each LED
            let sub0 = state_subscriber.clone();
            let sub1 = state_subscriber.clone();
            let sub2 = state_subscriber.clone();

            // Run Core 1 executor
            let executor = embassy_executor::Executor::new();
            executor.run(|spawner| {
                spawner.spawn(crate::ui::led_task(0, led0, sub0)).unwrap();
                spawner.spawn(crate::ui::led_task(1, led1, sub1)).unwrap();
                spawner.spawn(crate::ui::led_task(2, led2, sub2)).unwrap();
            });
        }
    ).unwrap();

    // Setup Core 0 hardware (audio, ADC, buttons)
    let (adc_bus, freq_pin, vol_pin) = setup_adc(
        peripherals.ADC1,
        peripherals.GPIO1,
        peripherals.GPIO2,
    );

    let audio_stream = setup_audio(
        peripherals.I2S0,
        dma_channel,
        peripherals.GPIO7,
        peripherals.GPIO8,
        peripherals.GPIO9,
        /* buffers */
    );

    (audio_stream, adc_bus, freq_pin, vol_pin)
}

/// Internal LED setup (called on Core 1)
fn setup_leds_internal(
    gpio10: esp_hal::peripherals::GPIO10<'static>,
    gpio11: esp_hal::peripherals::GPIO11<'static>,
    gpio12: esp_hal::peripherals::GPIO12<'static>,
) -> (Output<'static>, Output<'static>, Output<'static>) {
    let config = OutputConfig::default();
    (
        Output::new(gpio10, config),
        Output::new(gpio11, config),
        Output::new(gpio12, config),
    )
}
```

**Key Design**: Main function remains clean - hardware.rs owns all the complexity.

#### 3.2 LED Blink Task (`src/ui/led.rs`)

```rust
use crate::StateSubscriber;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Output;

/// LED control task for a single voice indicator.
///
/// Behavior:
/// - Selected + Active: Solid ON
/// - Not selected + Active: Blink (50ms on, 50ms off)
/// - Inactive: OFF
///
/// # Arguments
/// * `voice_id` - Which voice this LED represents (0-2)
/// * `led` - Type-erased GPIO output pin
/// * `state_sub` - Cloned subscriber for state updates
#[embassy_executor::task(pool_size = 3)]
pub async fn led_task(
    voice_id: u8,
    mut led: Output<'static>,
    mut state_sub: StateSubscriber,
) {
    loop {
        // Wait for next state update (blocking, no busy wait)
        let state = state_sub.next_message_pure().await;

        // Find our voice's state
        let voice = state.voices[voice_id as usize];

        match (voice.active, voice.selected) {
            (true, true) => {
                // Selected + Active: Solid ON
                led.set_high();
                // Don't sleep - immediately check for next state
            }

            (true, false) => {
                // Active but not selected: Blink
                led.set_high();

                // Check for state change every 50ms (blink half-period)
                match embassy_time::with_timeout(
                    Duration::from_millis(50),
                    state_sub.next_message_pure()
                ).await {
                    Ok(new_state) => {
                        // State changed mid-blink, handle it next iteration
                        state_sub.try_next_message_pure();  // Consume
                        continue;
                    }
                    Err(_) => {
                        // Timeout - complete blink cycle
                        led.set_low();
                        Timer::after(Duration::from_millis(50)).await;
                    }
                }
            }

            (false, _) => {
                // Inactive: OFF
                led.set_low();
                // Don't sleep - immediately check for next state
            }
        }
    }
}
```

**Design Rationale**:
- `pool_size = 3`: Spawn same task 3 times with different `voice_id`/`led` params
- **Reactive, not polled**: Waits for state updates, no busy loops
- **Blink interruption**: State changes mid-blink are handled immediately
- **Type-erased GPIO**: `Output<'static>` (no generic) satisfies embassy constraint

#### 3.3 UI Module Structure (`src/ui/mod.rs`)

```rust
//! User interface tasks: LEDs, future LCD visualizer

pub mod led;

pub use led::led_task;
```

Update `src/lib.rs`:
```rust
pub mod ui;
```

---

### Phase 4: Wiring It Together

#### 4.1 Complete Voice Selection Logic (`src/engine.rs`)

```rust
Message::SelectVoice(id) => {
    if let Some(voice) = self.voices.get(id as usize) {
        match self.selected_voice {
            Some(current) if current == id => {
                // Clicking same voice: deselect
                self.selected_voice = None;
            }
            _ => {
                // Selecting different voice (or first selection)
                self.selected_voice = Some(id);
            }
        }
        true  // State changed
    } else {
        false  // Invalid voice ID
    }
}
```

#### 4.2 Button Behavior Update (`src/controls/button.rs`)

Currently sends `ToggleVoice`. Update to send `SelectVoice`:

```rust
#[embassy_executor::task(pool_size = 3)]
pub async fn button_task(sender: CtrlSender, mut button: Input<'static>, voice_idx: u8) {
    loop {
        button.wait_for_low().await;

        // First press: Select voice
        // Second press (if already selected): Toggle active state
        // This logic is handled in Engine
        if let Err(e) = sender.try_send(Message::SelectVoice(voice_idx)) {
            warn!("Button message dropped: {:?}", e);
        }

        button.wait_for_high().await;
    }
}
```

**Alternative**: Keep separate ToggleVoice logic, but SelectVoice makes more semantic sense for button press.

#### 4.3 Integration Complete

All multicore and LED logic now resides in `hardware.rs`. Main function simply:
1. Creates channels (MPSC + PubSub)
2. Calls `init_multicore_hardware()` which handles Core 1 startup + LED tasks
3. Spawns Core 0 control tasks
4. Runs audio loop

**No peripheral ownership issues** - hardware.rs extracts what each core needs before moving into closures.

---

## Phase 5: Testing & Validation

### 5.1 Unit Test State Extraction

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_snapshot() {
        // Create mock engine state
        // Verify VoiceState matches Voice internals
        // Ensure Copy/Clone work
    }
}
```

### 5.2 Hardware Test Scenarios

1. **LED Solid**: Press button → LED on, pots control voice
2. **LED Blink**: Activate 2nd voice (no selection) → LED blinks
3. **Multi-voice**: 3 voices active, select each → only selected is solid
4. **Deselection**: Press selected button again → LED turns off
5. **Audio continuity**: Monitor audio output during LED operations → no clicks/jitter

### 5.3 Performance Validation

```rust
// In Engine::stream_audio(), add periodic logging:
static mut FRAME_COUNT: u32 = 0;
unsafe { FRAME_COUNT += 1; }

if unsafe { FRAME_COUNT % 44100 == 0 } {  // Every 1 second
    log::info!("Audio core running, {} frames", unsafe { FRAME_COUNT });
}
```

Verify logs show consistent 44.1kHz operation (no stalls).

---

## Future Extensions (Phase 2 from DESIGN.md)

### LCD Visualizer Task

```rust
#[embassy_executor::task]
pub async fn lcd_task(
    mut state_sub: StateSubscriber,
    mut lcd: Lcd,  // SPI/I2C LCD driver
) {
    loop {
        let state = state_sub.next_message_pure().await;

        // Draw UI
        lcd.clear();
        for (i, voice) in state.voices.iter().enumerate() {
            lcd.draw_voice_bar(i, voice.frequency, voice.volume, voice.active);
        }
        lcd.flush();
    }
}
```

**No engine changes needed** - just spawn another subscriber on Core 1!

### Waveform Oscilloscope

Extend `EngineState`:
```rust
pub struct EngineState {
    pub voices: [VoiceState; VOICE_COUNT],
    pub waveform: &'static [i16; 128],  // Ring buffer reference
}
```

Engine snapshots last 128 samples before broadcasting. Oscilloscope task renders to CRT via VGA.

---

## API Surface Changes

### New Public Types (`src/lib.rs`)
- `StatePublisher`
- `StateSubscriber`
- `StatePubSubChannel`

### New Modules
- `src/state.rs` - State snapshot types
- `src/ui/mod.rs` - UI tasks module
- `src/ui/led.rs` - LED control

### Modified Modules
- `src/engine.rs` - Add state broadcasting (outside hot path)
- `src/voice.rs` - Make `osc` and `volume` fields public
- `src/oscillator.rs` - Make `frequency` field public
- `src/slew.rs` - Add `target()` getter
- `src/hardware.rs` - Add `init_multicore_hardware()` for dual-core setup
- `src/bin/main.rs` - Simplified to call hardware abstraction

### Modified Messages (Optional)
- `src/controls/button.rs` - Send `SelectVoice` instead of `ToggleVoice`
- Or: Keep ToggleVoice, add separate selection button

---

## Constraints & Guarantees

### Embassy Task Constraints
✅ **No generic tasks**: All types are concrete (via type aliases + type erasure)
✅ **Send bounds**: All task params are `Send` (`Output`, `Subscriber` are Send-safe)
✅ **Static lifetimes**: Cross-core data uses `'static` lifetime

### Concurrency Safety
✅ **Lock-free**: Both MPSC and PubSub are lock-free atomics
✅ **Core isolation**: Core 0 never touches GPIO, Core 1 never touches I2S
✅ **Non-blocking**: `publish_immediate()` never blocks audio thread

### Performance Guarantees
✅ **Zero audio impact**: Broadcasting happens in async context, NOT in DMA callback (pure math in hot path)
✅ **Lock-free atomics**: PubSub uses atomic operations (~200ns worst case), no mutex blocking
✅ **Dropped states OK**: UI wants latest state, history irrelevant (capacity: 4 updates)
✅ **Blink precision**: ±5ms jitter acceptable for LED (human perception ~50ms)

---

## Migration Checklist

- [ ] Create `src/state.rs` with state structs (VoiceState, EngineState)
- [ ] Add PubSub type aliases to `src/lib.rs` (StatePublisher, StateSubscriber, StatePubSubChannel)
- [ ] Make fields public: `Oscillator.frequency`, `Voice.osc`, `Voice.volume`
- [ ] Add `Slew.target()` getter to `src/slew.rs`
- [ ] Modify Engine:
  - [ ] Add `state_publisher: StatePublisher` field
  - [ ] Add `get_current_state()` method
  - [ ] Update `consume_messages()` to return `bool` (state changed flag)
  - [ ] Update `stream_audio()` to broadcast before hot path
- [ ] Create `src/ui/mod.rs` and `src/ui/led.rs` with LED task
- [ ] Add `init_multicore_hardware()` to `src/hardware.rs` (handles Core 1 setup internally)
- [ ] Update `main.rs`: simplified to call hardware abstraction
- [ ] Complete SelectVoice logic in Engine (deselect on second press)
- [ ] Update button behavior to send SelectVoice (optional)
- [ ] Test LED patterns (solid/blink/off)
- [ ] Validate audio continuity (no clicks/jitter during UI operations)
- [ ] Document GPIO pin assignments

---

## Pin Assignments

### Current
- **GPIO1**: Frequency pot (ADC)
- **GPIO2**: Volume pot (ADC)
- **GPIO3**: Button 0 (Input, pull-up)
- **GPIO7**: I2S BCLK (I2S TX)
- **GPIO8**: I2S WS (I2S TX)
- **GPIO9**: I2S DOUT (I2S TX)

### New (This Refactor)
- **GPIO10**: Voice 0 LED (Output)
- **GPIO11**: Voice 1 LED (Output)
- **GPIO12**: Voice 2 LED (Output)

### Reserved (Phase 2)
- **GPIO13-15**: SPI for LCD (MOSI, SCLK, CS)
- **GPIO16-17**: I2C for future sensors (SDA, SCL)

---

## References

- [esp-hal CpuControl docs](https://docs.esp-rs.org/esp-hal/esp32s3/esp_hal/system/struct.CpuControl.html)
- [Embassy PubSub API](https://docs.embassy.dev/embassy-sync/git/default/pubsub/index.html)
- [ESP32-S3 Dual Core Architecture](https://www.espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf)
- ESP-HAL multicore example: `esp-hal/examples/src/bin/embassy_multicore.rs`

---

## Architectural Improvements (Latest Revision)

This specification has been updated with the following improvements:

1. **Hardware Abstraction**: Multicore setup moved entirely into `hardware.rs` via `init_multicore_hardware()`. Main function stays clean and focused on Core 0 logic.

2. **Pub Fields over Getters**: Direct field access (`osc.frequency`, `voice.osc`) instead of getter methods for better performance and simpler code. Getter only used where encapsulation needed (e.g., `Slew.target()`).

3. **Broadcast Timing**: State publishing happens BEFORE `push_with()` (async context), not inside DMA callback. Hot path is pure math with zero atomic operations.

4. **Stack Size**: 16KB Core 1 stack from the start, providing headroom for future LCD/SPI operations without requiring changes.

5. **State Change Detection**: `consume_messages()` returns bool flag, only broadcasts when state actually changes. Validates voice IDs before updating.

**Result**: Cleaner code, better performance, easier to maintain, ready for LCD visualization without architectural changes.

---

**End of Technical Specification**
