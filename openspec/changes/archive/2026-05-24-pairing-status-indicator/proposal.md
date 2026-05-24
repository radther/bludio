## Why

Currently, when a user clicks "Pair & Trust" on a Bluetooth device, there is no visual feedback in the UI that anything is happening. The user sees no indication of progress, waits in the dark, and only learns the outcome once the operation completes (or silently fails). This makes the app feel unresponsive and leaves users uncertain whether their click registered.

## What Changes

- Add per-device pairing status state that tracks the current step in the pairing/connection chain
- Display a colored status dot with brief descriptive text (e.g., "connecting…", "pairing…", "trusting…") on the device row during pairing
- Break the Pair & Trust action into discrete observable steps so each phase can be reported to the UI
- On failure, show a brief failure status ("failed to pair") in the UI and log a detailed error message to the console
- Automatically clear the pairing status indicator once the device reaches its final state (paired/connected, or after a refresh cycle)

## Capabilities

### New Capabilities
- `pairing-status-indicator`: Per-device visual status indicator showing the current step in the Bluetooth pairing/connection chain, with brief descriptive text and color-coded dot during active operations, and clear failure state display.

### Modified Capabilities
- `bluetooth-device-management`: The pairing flow requirement changes — the system SHALL display intermediate status during the Pair & Trust action instead of remaining silent. The "Pairing fails" scenario is extended to include a visible failure indicator in the UI.

## Impact

- **`src/bluetooth/device.rs`**: Add `PairingStatus` enum to the device model
- **`src/bluetooth/mod.rs`**: Add status-tracking map or field to `BluetoothState`
- **`src/ui/bluetooth/device_row.rs`**: Render the pairing status dot + text when status is active; modify status line rendering logic
- **`src/app.rs`**: Modify the `DeviceAction` command handler to execute pairing in observable steps with status updates between each step
- **No new external dependencies**: Uses existing `bluer`, `gpui`, and `tokio` infrastructure
