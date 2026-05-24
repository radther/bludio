## Why

The Bluetooth page and dev test page were built before the Entity-based page pattern was established. `AudioPage` now demonstrates the correct architecture: a self-contained `Entity` that owns its children, handles state internally, and communicates via `EventEmitter`. The Bluetooth page still uses raw functions (`bluetooth_page_view()`, `device_list_view()`) that couple `BludioApp` to page internals (actions, color params, event dispatching). Refactoring both pages to match `AudioPage`'s pattern makes the codebase consistent and reduces `BludioApp`'s surface area.

## What Changes

- **Refactor `src/ui/bluetooth_page.rs` into a self-contained `BluetoothPage` entity** that manages its own state, handles scan/action dispatching internally, and only needs `BluetoothState` passed in (matching `AudioPage::sync_rows`).
- **Extract Bluetooth device list rows** into a sub-module (`src/ui/bluetooth/`) with dedicated entity+components (matching `AudioDeviceRow`).
- **Rename `src/ui/text_field_test.rs` → `src/ui/dev_test_page.rs`** and rename the struct to `DevTestPage` to match page naming conventions and signal the page is a general developer test area.
- **Clean up `src/app.rs`**: replace `bluetooth_page_view()` call and raw `BluetoothState` child access with `Entity<BluetoothPage>` (same pattern as audio pages). Remove the `actions::execute` import from `app.rs` — action dispatch moves into the `BluetoothPage` entity.
- **Remove `src/actions.rs`**: the contents fold into `BluetoothPage`'s internal methods.
- **Clean up `src/ui/mod.rs`**: update module declarations for the renames and new bluetooth sub-module.

## Capabilities

### New Capabilities

None — this is a pure architectural refactor. All user-facing behaviors remain identical.

### Modified Capabilities

None — no spec-level requirement changes. The same Bluetooth device management behaviors are implemented with a different internal architecture.

## Impact

- **`src/ui/bluetooth_page.rs`**: major rewrite into `BluetoothPage` entity + extract list into `src/ui/bluetooth/` sub-module
- **`src/ui/text_field_test.rs`**: rename to `src/ui/dev_test_page.rs` (struct `TextFieldTestPage` → `DevTestPage`)
- **`src/app.rs`**: simplified render — uses `Entity<BluetoothPage>` and `Entity<DevTestPage>` instead of raw functions; `BluetoothState` no longer accessed directly in render
- **`src/ui/mod.rs`**: updated module declarations
- **`src/actions.rs`**: removed; action logic moves into `BluetoothPage`
- **`src/main.rs`**: no changes (already thin)
