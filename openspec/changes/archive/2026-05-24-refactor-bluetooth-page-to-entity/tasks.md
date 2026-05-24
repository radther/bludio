## 1. Rename dev test page

- [x] 1.1 Rename `src/ui/text_field_test.rs` → `src/ui/dev_test_page.rs`, rename `TextFieldTestPage` → `DevTestPage`
- [x] 1.2 Update `src/app.rs`: change import, field name (`text_field_test_page` → `dev_test_page`), and active page enum variant name
- [x] 1.3 Update `src/ui/mod.rs`: change `pub mod text_field_test` → `pub mod dev_test_page`

## 2. Create Bluetooth sub-module structure

- [x] 2.1 Create `src/ui/bluetooth/mod.rs` declaring `pub(crate) mod bluetooth_page;` and `pub(crate) mod device_row;`
- [x] 2.2 Add `pub mod bluetooth;` to `src/ui/mod.rs`

## 3. Create BluetoothDeviceRow entity

- [x] 3.1 Create `src/ui/bluetooth/device_row.rs` with `BluetoothDeviceRow` entity holding device data (address, name, paired, connected, adapter handle, action channel sender)
- [x] 3.2 Define `BluetoothPageCommand` enum (shared type for page↔app communication) with variants `ToggleScan` and `DeviceAction { addr, action }`
- [x] 3.3 Implement `Render` for `BluetoothDeviceRow`: device info (name, address, status badge) + conditional action buttons (Connect / Disconnect / Forget / Pair & Trust)
- [x] 3.4 Action buttons send commands via `UnboundedSender<BluetoothPageCommand>`; the row does NOT spawn async tasks directly

## 4. Create BluetoothPage entity

- [x] 4.1 Create `src/ui/bluetooth/bluetooth_page.rs` with `BluetoothPage` entity holding `Vec<Entity<BluetoothDeviceRow>>`, cached `discovering` flag, cached `error` string, cached `initialized` flag, and action channel sender
- [x] 4.2 Implement `BluetoothPage::new(cmd_tx, cx)` constructor
- [x] 4.3 Implement `sync_state(&mut self, state: &BluetoothState, window, cx)` — creates/updates/removes rows based on device list; caches discovering, error, initialized flags
- [x] 4.4 Implement `Render` for `BluetoothPage`: header with scan button, error banner, loading indicator, device list (same visual states as current `bluetooth_page_view()`)
- [x] 4.5 Move `execute()` and `quick_device_status()` from `src/actions.rs` into `app.rs` as private helpers (action execution needs the adapter which the page doesn't own)
- [x] 4.6 Rows send `BluetoothPageCommand` directly through cloned channel sender — no EventEmitter/subscribe_in needed (rows have no interactive sub-components)

## 5. Update BludioApp to use new Bluetooth entities

- [x] 5.1 Add `bluetooth_page: Entity<BluetoothPage>` field to `BludioApp`
- [x] 5.2 Create `UnboundedSender<BluetoothPageCommand>` and spawn handler in `BludioApp::new()` that listens for commands and executes actions via `tokio_task + adapter`
- [x] 5.3 Update render: swap `ui::bluetooth_page::bluetooth_page_view(&self.bt_state, cx)` with `self.bluetooth_page.clone().into_any_element()`
- [x] 5.4 Remove direct `BluetoothState` reads from render (the `match active_page { Page::BluetoothDevices => ... }` arm now just clones the entity)
- [x] 5.5 Call `self.bluetooth_page.update(cx, |page, cx| page.sync_state(&self.bt_state, window, cx))` at the same points where audio pages call `sync_rows()` (init, discovery updates, monitor updates)

## 6. Remove old code and update module declarations

- [x] 6.1 Delete `src/actions.rs` (contents folded into app.rs)
- [x] 6.2 Remove `mod actions;` from `src/main.rs`
- [x] 6.3 Remove `pub mod bluetooth_page;` from `src/ui/mod.rs` (replaced by `pub mod bluetooth;`)
- [x] 6.4 Delete old `src/ui/bluetooth_page.rs`

## 7. Build and verify

- [x] 7.1 Run `cargo build` — ensure no compilation errors
- [x] 7.2 Run `cargo clippy` — ensure no new warnings
- [x] 7.3 Visually verify `src/app.rs` render method is clean and minimal for Bluetooth page (one line: `self.bluetooth_page.clone()`) — confirmed at line 367
