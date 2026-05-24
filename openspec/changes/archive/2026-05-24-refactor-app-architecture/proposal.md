## Why

`src/app.rs` has accumulated 502 lines of mixed responsibilities — entity definition, four unnamed async blocks in the constructor, Bluetooth domain logic, and utility functions — making it hard to navigate, test, or extend. Bluetooth helper functions (`run_discovery`, `execute_device_action`, `quick_device_status`, `devices_changed`) live in `app.rs` instead of the `bluetooth/` module where they belong. The Tokio `EnterGuard` nesting is fragile and has already caused panics, and Bluetooth channel types are inconsistent (Tokio mpsc for discovery, futures mpsc for commands) solely to work around the nesting issue. This is the right time to clean house before the codebase grows further.

## What Changes

- Extract the four async init blocks from `BludioApp::new()` into named private methods on `BludioApp` (e.g., `spawn_audio_state_loop`, `spawn_bluetooth_init`, `spawn_bluetooth_command_handler`)
- Move `run_discovery` from `app.rs` into `bluetooth/discovery.rs` (it orchestrates discovery — `start_scan` is already there)
- Move `execute_device_action` from `app.rs` into `bluetooth/device.rs` or a new `bluetooth/actions.rs`
- Move `quick_device_status` from `app.rs` into `bluetooth/device.rs` (it calls `properties::fetch_properties` + `properties::build_device`)
- Move `devices_changed` from `app.rs` into `bluetooth/device.rs` as a utility
- Remove the dead `initialized` field from `BludioApp` (never read; `BluetoothPage` derives its own from `state.adapter.is_some()`)
- Remove the unused `_text_secondary` binding in `BludioApp::render()`
- Consolidate Tokio context management into a single Tokio worker that owns the `EnterGuard`, and route all Bluetooth interop through non-Tokio channels (`futures::channel::mpsc`), eliminating the scattered `TOKIO.enter()` calls and the nesting panic risk
- Unify Bluetooth channel types to `futures::channel::mpsc` consistently (discovery events and command channels)
- Investigate whether PulseAudio / audio channels can also adopt the same unified pattern, or whether they must remain on `tokio::sync::mpsc` for technical reasons (PA thread is a `std::thread`, not a Tokio task)

## Capabilities

### New Capabilities

None — this is a pure internal refactor with no user-facing behavioral changes.

### Modified Capabilities

None — no spec-level requirement changes. All existing behavior is preserved.

## Impact

- **`src/app.rs`**: Reduced from ~500 lines to ~150 lines. Entity definition, `Render`, and named init methods only.
- **`src/bluetooth/discovery.rs`**: Gains `run_discovery` (orchestration layer above `start_scan`).
- **`src/bluetooth/device.rs`** or **`src/bluetooth/actions.rs`**: Gains `execute_device_action`, `quick_device_status`, `devices_changed`.
- **`src/bluetooth/mod.rs`**: Possible new `pub(crate) mod actions;` if a new file is created.
- **`src/main.rs`**: Possible minor changes if the Tokio worker pattern moves there.
- **Channel plumbing**: `tokio::sync::mpsc` in Bluetooth discovery path replaced with `futures::channel::mpsc`. Audio channels evaluated for the same treatment.
