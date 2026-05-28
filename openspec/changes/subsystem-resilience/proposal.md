## Why

Bludio currently assumes both PulseAudio and BlueZ are available for the entire application lifetime. If either subsystem is restarted (e.g., `systemctl restart bluetooth`, PulseAudio crash, `pipewire` restart), the app enters a broken state — stale data, silent command failures, and no indication to the user that anything is wrong. This makes the app unreliable for daily use, especially on systems where PipeWire/PulseAudio or BlueZ may be restarted independently of Bludio.

## What Changes

- **Bluetooth disconnect detection**: Detect when the BlueZ D-Bus session becomes invalid (adapter removed, D-Bus name lost) and surface this as a UI error state with the existing error banner and page-level error display.
- **Bluetooth reconnection**: Automatically attempt to re-establish the BlueZ session and re-register the agent when BlueZ comes back online, using D-Bus `NameOwnerChanged` signals (reactive, no polling).
- **Audio disconnect detection**: Detect when the PulseAudio thread exits (channel closure) or the PA context enters a failed/terminated state, and surface this as a UI error state on both audio pages.
- **Audio reconnection**: Automatically restart the PA thread when PulseAudio/PipeWire becomes available again. Since PA has no D-Bus name to watch, use a short fixed-interval retry (every 3 seconds) only while disconnected.
- **Unified subsystem status model**: Introduce a `SubsystemStatus` enum (`Connecting`, `Connected`, `Disconnected(String)`, `Reconnecting`) shared across Bluetooth and Audio state. This replaces all boolean/option combinations (`connected`, `error`, `initialized`) with a single state machine that pages `match` on to decide what to render.

## Capabilities

### New Capabilities
- `subsystem-health`: Unified subsystem health monitoring — status enum, disconnect detection, reconnect coordination, and UI error/reconnecting states for both Bluetooth and Audio backends.

### Modified Capabilities
- `bluetooth-device-management`: Requirements change for session lifecycle — the Bluetooth state must survive BlueZ restarts and re-establish the adapter/agent without user intervention.
- `audio-device-management`: Requirements change for PA thread lifecycle — the audio backend must detect PA thread exit and reinitialize the connection, preserving the command channel.

## Impact

- **`src/bluetooth/mod.rs`**: `BluetoothState` gains reconnect logic; session/adapter become re-derivable.
- **`src/bluetooth/monitor.rs`**: Monitor must detect D-Bus name loss and signal disconnect.
- **`src/audio/pulse.rs`**: PA thread must detect context failure and signal disconnect; thread lifecycle becomes restartable.
- **`src/audio/mod.rs`**: `AudioState` gains `SubsystemStatus`; command channel must survive restarts.
- **`src/app.rs`**: `BludioApp` must handle subsystem disconnect/reconnect events, rewire channels, and propagate status to pages.
- **`src/ui/bluetooth/bluetooth_page.rs`**: Must render disconnected/reconnecting states (reuse existing error/loading patterns).
- **`src/ui/audio/audio_page.rs`**: Must render disconnected/reconnecting states.
- **No new dependencies**: All detection mechanisms use existing `bluer` (D-Bus signals) and `libpulse-binding` (context state) APIs.
