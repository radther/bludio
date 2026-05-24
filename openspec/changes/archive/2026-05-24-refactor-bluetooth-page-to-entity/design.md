## Context

The current Bluetooth page (`src/ui/bluetooth_page.rs`) is implemented as a set of free functions (`bluetooth_page_view()`, `device_list_view()`, `action_btn()`, etc.) that receive `&BluetoothState` and `&mut Context<BludioApp>`. This pattern couples the page to `BludioApp`'s internal fields (`this.bt_state`) and forces `BludioApp`'s render method to thread Bluetooth-specific imports through the app module.

The `AudioPage` entity (introduced later) established the preferred pattern: a self-contained `Entity` that owns its children, receives state updates via a typed `sync_rows()` method, and communicates actions via an internal channel (`cmd_tx: UnboundedSender<AudioCommand>`). The app module only needs to create the page entity and call `sync_rows()` — never accesses the page's children directly.

The `TextFieldTestPage` page already follows the Entity pattern but needs a rename to `DevTestPage` to match conventions and reflect its broader purpose.

## Goals / Non-Goals

**Goals:**
- `BluetoothPage` is a self-contained `Entity` that manages its own render, child entities, and action logic
- `BludioApp::render()` delegates to `Entity<BluetoothPage>` without accessing `BluetoothState` directly for the Bluetooth content area
- Bluetooth device list rows are extracted into a sub-module with clear entity ownership
- `DevTestPage` follows the same page naming conventions as other pages
- Module structure mirrors `src/ui/audio/` (page + row sub-modules)
- Existing user-facing behavior is 100% preserved (scan button, action buttons, error/loading states, device list)

**Non-Goals:**
- Changing the Bluetooth lifecycle orchestration (init, monitor, discovery loop) — these stay in `BludioApp`
- Adding new Bluetooth features or changing device filtering behavior
- Refactoring `audio/device_row.rs` or any audio components
- Changing `main.rs`

## Decisions

### 1. BluetoothPage owns an action channel (matching AudioPage's `cmd_tx`)

The `AudioPage` pattern: the page receives an `UnboundedSender<AudioCommand>` channel. User interactions on rows send commands through this channel. A background thread (PA thread) processes them. This decouples the page from the backend entirely.

For Bluetooth, there is no separate background thread — actions are async Tokio tasks spawned on-demand. But the same channel pattern can bridge the gap: `BluetoothPage` gets an `UnboundedSender<BluetoothAction>` (or simply a `WeakEntity<BludioApp>` for spawning). However, the Bluetooth actions need the adapter, which is on `BluetoothState`.

**Decision**: Use an action channel pattern. `BluetoothPage` receives an `UnboundedSender<BluetoothPageCommand>` which `BludioApp` services. This keeps the page from needing to know about the Tokio runtime or adapter lifecycle.

**Alternative considered**: Pass `WeakEntity<BludioApp>` to the page and let it spawn directly. Rejected — this inverts the ownership chain (pages shouldn't know about the app entity) and prevents testing the page in isolation.

### 2. Bluetooth device rows are `Entity<BluetoothDeviceRow>` (matching AudioDeviceRow)

`AudioDeviceRow` is an Entity because it owns interactive components (`Entity<TextField>`, `Entity<Dropdown>`). Bluetooth device rows have no such children — they're purely informational with action buttons. However, making them entities provides:

- Consistent ownership pattern across all pages
- EventEmitter-based communication from row → page
- Easy future extension (adding editable name, etc.)

**Decision**: Use `Entity<BluetoothDeviceRow>` for consistency. The overhead is minimal and the pattern clarity is worth it.

### 3. `sync_state(&mut self, state: &BluetoothState, window, cx)` replaces direct access

Currently `bluetooth_page_view(&state, cx)` reads everything from `BluetoothState` during render. The Entity pattern requires state to be pushed into the entity before render. `AudioPage::sync_rows()` does exactly this.

**Decision**: `BluetoothPage::sync_state()` receives a reference to `BluetoothState` and checks what changed (discovering flag, error, device list). It updates its internal fields, creates/removes row entities as needed, and calls `cx.notify()`. Render only reads cached fields, never `BluetoothState`.

### 4. `actions.rs` folds into `BluetoothPage`'s internal methods

The `execute()` and `quick_device_status()` functions in `src/actions.rs` are specific to Bluetooth device operations. They don't need to be a top-level module. Moving them into `BluetoothPage` (or a private helper) keeps related code together.

**Decision**: Move `execute()` and `quick_device_status()` into `src/ui/bluetooth/bluetooth_page.rs` as private methods or free functions. Remove `src/actions.rs`.

### 5. Dev test page: simple rename, no structural change

`TextFieldTestPage` already follows the Entity pattern correctly. The rename to `DevTestPage` is purely cosmetic.

**Decision**: Rename file to `dev_test_page.rs`, struct to `DevTestPage`, keep everything else identical.

## Risks / Trade-offs

- **[Risk] Breaking existing behavior during refactor**: The Bluetooth page has several states (loading, error, empty, device list, scanning). Missing any state branch could cause regressions. → **Mitigation**: Follow the existing state branches exactly. Build and test after each implementation step.
- **[Risk] Action channel adds indirection**: Adding a channel between page and app for Bluetooth actions is a layer of abstraction that didn't exist before. → **Mitigation**: The channel is minimal — just a few variants (ToggleScan, DeviceAction { addr, action }). `BludioApp` services it in a small `cx.spawn` block. Same pattern already works for audio.
- **[Risk] Entity-per-row may feel heavy**: Each `BluetoothDeviceRow` is an Entity with state, even though it currently has no interactive sub-components. → **Trade-off**: The overhead is negligible compared to the D-Bus calls these rows trigger. Consistency across all page types outweighs the minor allocation cost.
