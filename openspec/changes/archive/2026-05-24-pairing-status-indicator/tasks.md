## 1. Data model — Add `PairingStatus` enum

- [x] 1.1 Add `PairingStatus` enum to `src/bluetooth/device.rs` with variants: `Connecting`, `Pairing`, `Trusting`, `Failed(String)`, plus `Display` impl for human-readable text
- [x] 1.2 Add `pairing_status: Option<PairingStatus>` field to `BluetoothDevice` struct
- [x] 1.3 Update `quick_device_status` to accept and preserve an existing `Option<PairingStatus>` (so status isn't lost during single-device refreshes)

## 2. Bluetooth state — Propagate pairing status through state operations

- [x] 2.1 Update `build_device` in `src/bluetooth/properties.rs` to initialize `pairing_status: None` on new devices
- [x] 2.2 Update `BluetoothState::upsert_device` to preserve existing `pairing_status` when updating a device that has one
- [x] 2.3 Update `BluetoothState::replace_devices` to clear stale `pairing_status` on all devices (full refresh means any active operation is done)
- [x] 2.4 Update `devices_changed` comparison to ignore `pairing_status` (it's a display-only field, should not block UI updates)

## 3. Command handler — Execute Pair & Trust in discrete steps with status updates

- [x] 3.1 In `spawn_bluetooth_command_handler` in `src/app.rs`, replace the monolithic `PairAndTrust` handling with step-by-step execution:
  - Set `pairing_status = Connecting` via `update_in`, notify
  - Run `device.connect()` on Tokio, log error on failure
  - Set `pairing_status = Pairing` via `update_in`, notify
  - Run `device.pair()` on Tokio, handle failure with status + log
  - Set `pairing_status = Trusting` via `update_in`, notify
  - Run `device.set_trusted(true)` + `device.connect()` on Tokio
  - Clear `pairing_status` and run final device refresh on success
- [x] 3.2 Extract a small helper for pushing pairing status updates to `BluetoothState` (set status on device by address, then `cx.notify()`)
- [x] 3.3 On any step failure, set `pairing_status = Failed(reason)`, log the detailed error to `eprintln!`, and skip remaining steps

## 4. Device row UI — Render pairing status indicator

- [x] 4.1 In `BluetoothDeviceRow::render` in `src/ui/bluetooth/device_row.rs`, check `self.pairing_status` before the normal status line
- [x] 4.2 When `pairing_status` is `Some(...)`, render a colored dot + status text in place of the normal "connected"/"paired"/"discovered" status
- [x] 4.3 Define dot colors: blue for Connecting, amber for Pairing, green for Trusting, red for Failed
- [x] 4.4 When `pairing_status` is `None`, render the existing status line unchanged

## 5. Data flow — Update constructors and sync logic

- [x] 5.1 Update `BluetoothDeviceRow::new` to accept `Option<PairingStatus>`
- [x] 5.2 Update `BluetoothDeviceRow::update_from_device` to sync `pairing_status`
- [x] 5.3 Update `BluetoothPage::sync_state` in `bluetooth_page.rs` to pass `pairing_status` when creating/updating rows
- [x] 5.4 Verify the monitor loop (D-Bus signal handler and 10s fallback) naturally clears stale status via `replace_devices` or `upsert_device`

## 6. Polish and verify

- [x] 6.1 Run `cargo build` and fix any compilation errors
- [x] 6.2 Run `cargo clippy` and fix any warnings
- [x] 6.3 Run `cargo fmt` to ensure consistent formatting
- [x] 6.4 Manual verification: confirm the status indicator shows during pairing (if Bluetooth hardware available)
