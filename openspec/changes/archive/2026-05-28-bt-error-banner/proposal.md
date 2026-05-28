## Why

Device actions (connect, disconnect, forget) silently swallow D-Bus errors — the user gets no feedback when an operation fails. The only indication is that nothing changed. Additionally, the error messages logged to stderr are terse (e.g., "Connect failed: org.bluez.Error.Failed") without explaining *why* — was the device out of range? Was the adapter busy? Did the device reject the connection? This makes debugging frustrating for both users and developers.

## What Changes

- **Error banner component**: A reusable bottom banner that slides up to display error messages, matching the tab bar's visual style. Uses GPUI's `with_animation` API for smooth entry/exit.
- **Device action error surfacing**: `execute_device_action` returns `Result<(), String>` with human-readable error messages. The Bluetooth page displays failures in the error banner.
- **Improved error messages**: Parse BlueZ D-Bus errors into user-friendly explanations (e.g., "Device is out of range" instead of "org.bluez.Error.NotAvailable").

## Capabilities

### New Capabilities
- `error-banner`: Reusable animated bottom banner component for displaying transient error messages. Covers banner lifecycle (show/hide), animation, styling, and dismissal.

### Modified Capabilities
- `bluetooth-device-management`: Device actions now surface errors to the UI via the error banner instead of silently logging to stderr.

## Impact

- **`src/bluetooth/device.rs`**: `execute_device_action` signature changes from `()` to `Result<(), String>`
- **`src/app.rs`**: Command handler must handle errors from device actions and push them to the banner
- **`src/ui/components/`**: New `error_banner.rs` module
- **`src/ui/bluetooth/bluetooth_page.rs`**: Integrates the error banner, manages its visibility state
- **`src/ui/theme.rs`**: May need banner-specific color tokens (or reuse tab bar colors)
