## 1. Project Setup & Dependencies

- [x] 1.1 Add `bluer` (with `zbus`), `tokio`, and `futures` dependencies to `Cargo.toml`
- [x] 1.2 Create `src/bluetooth/` directory with `mod.rs`, `device.rs`, `agent.rs` stubs (later extended with `discovery.rs`, `properties.rs`; `adapter.rs` removed as empty placeholder)
- [x] 1.3 Verify project compiles with new crates and module structure (`cargo check`)

## 2. Bluetooth Backend — Data Model & Session

- [x] 2.1 Define `BluetoothDevice` struct in `src/bluetooth/device.rs` with fields: path, address, name, alias, paired, connected, trusted
- [x] 2.2 Define `BluetoothState` struct in `src/bluetooth/mod.rs` with: session, adapter handle, device list (`Vec<BluetoothDevice>`), discovery active flag, error message string
- [x] 2.3 Implement `BluetoothState::new()` — connect to BlueZ session, get default adapter, set powered+pairable, return state (or error)

## 3. Bluetooth Backend — Device Operations

- [x] 3.1 Implement `list_devices()` on `BluetoothState` — call `adapter.device_addresses()`, build `BluetoothDevice` instances with current properties
- [x] 3.2 Implement `connect_device(&self, address)` — get device by address, call `device.connect()`, handle errors
- [x] 3.3 Implement `disconnect_device(&self, address)` — call `device.disconnect()`, handle errors
- [x] 3.4 Implement `forget_device(&self, address)` — call `adapter.remove_device(address)`, handle errors

## 4. Bluetooth Backend — Discovery

- [x] 4.1 Implement `start_discovery()` — call `adapter.discover_devices()`, return an event stream for new/removed devices
- [x] 4.2 Implement discovery event handling — on `AdapterEvent::DeviceAdded`, fetch device properties and add to state; emit updates that UI can consume
- [x] 4.3 Implement `stop_discovery()` — stop the discovery stream, set discovering flag to false
- [x] 4.4 Implement 30-second scan timeout as a safety measure

## 5. Bluetooth Backend — Pairing Agent

- [x] 5.1 Implement `BluetoothAgent` struct in `src/bluetooth/agent.rs` with `KeyboardDisplay` capability
- [x] 5.2 Implement auto-accept handlers: `request_passkey` (return 123456), `request_confirmation` (accept), `display_passkey` (log). Combined these provide `KeyboardDisplay` capability covering modern pairing models. `request_pin_code` and `request_authorization` omitted — not needed for current device support.
- [x] 5.3 Implement `register_agent()` — register agent path with `AgentManager1`, set as default agent
- [x] 5.4 Implement `pair_and_trust(&self, address)` — ensure agent registered, call `device.pair()`, then `device.set_trusted(true)`

## 6. UI — Device List Component

- [x] 6.1 Create `src/device_list.rs` with `DeviceListView` render function that accepts `&BluetoothState`
- [x] 6.2 Implement the device list — each row shows device name (or MAC), connection status badge, and action buttons
- [x] 6.3 Implement conditional button rendering per row: Connect (if paired & !connected), Disconnect (if connected), Forget (if paired), Pair & Trust (if !paired)
- [x] 6.4 Wire button clicks to dispatch actions via async spawns that call the Bluetooth backend and update state
- [x] 6.5 Add Scan/Stop button at the top of the list area, with visual toggle state reflecting discovery status

## 7. App Integration

- [x] 7.1 Update `BludioApp` in `src/main.rs` to hold `BluetoothState` instead of just `FocusHandle`
- [x] 7.2 Initialize Bluetooth state in `cx.spawn()` during app creation; set error state if BlueZ unavailable
- [x] 7.3 Update `Render` impl to compose the device list component along with the existing app chrome
- [x] 7.4 Wire up PropertiesChanged signal listener to update device state reactively (Connected/Paired changes from external events). Implemented via per-device D-Bus signal monitoring (`bluetooth/monitor.rs`) with a 10-second full-list fallback refresh.

## 8. Error Handling & Cleanup

- [x] 8.1 Display error banner when BlueZ is unavailable or operations fail
- [x] 8.2 Ensure all D-Bus calls have `.ok()` or error logging — no silent failures
- [x] 8.3 Run `cargo clippy` — fix all warnings, ensure no `unsafe` blocks
- [x] 8.4 Run `cargo build` — confirm zero warnings and successful compilation
