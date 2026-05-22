## Why

Bludio needs to interact with the Linux Bluetooth subsystem to manage paired devices, discover new ones, and control connections. This is the first real feature beyond the app shell, establishing the D-Bus/BlueZ integration layer and a functional (not designed) UI for testing Bluetooth interactions.

## What Changes

- Add BlueZ D-Bus integration using the `bluer` crate for async Bluetooth operations
- Build a scrolling list view of paired Bluetooth devices with name, address, and connection status
- Implement connect/disconnect/forget actions per device via button interactions
- Implement a scan workflow that discovers nearby devices and updates the UI in real time
- Implement a pair+trust workflow (single button triggers Pair then Trust)
- Register a minimal BlueZ pairing agent to handle passkey/confirmation dialogs
- Split the codebase into well-organized modules: Bluetooth backend layer, UI components, and app state
- No visual design work — functional layout only, focused on Bluetooth correctness

## Capabilities

### New Capabilities
- `bluetooth-device-management`: Enumerate paired Bluetooth devices via BlueZ D-Bus, scan for nearby devices with live UI updates, and perform device actions (connect, disconnect, forget, pair+trust) through a scrolling UI list.

### Modified Capabilities
<!-- None — app-shell requirements are unchanged -->

## Impact

- **Dependencies**: Add `bluer` (async BlueZ wrapper), `zbus` (transitive), `tokio` (async runtime), `futures` (channel utilities)
- **Source structure**: `src/bluetooth/` directory with submodules (device, agent, discovery, properties), `src/device_list.rs` (UI component), `src/actions.rs` (action orchestration), `src/main.rs` (app shell, Tokio bridge, discovery orchestration)
- **GPUI integration**: Uses div children with `overflow_y_scroll` for the list (not `uniform_list` — interactive buttons require `Context<T>` which the `uniform_list` render closure doesn't provide in gpui-unofficial). Async work routes through a dedicated `tokio::runtime::Runtime` via `tokio_task()` bridge because gpui-unofficial's executor is not Tokio-based
- **System**: Requires BlueZ running on D-Bus system bus; compatible with Linux only
