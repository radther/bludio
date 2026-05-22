## Context

Bludio is a blank GPUI application (Linux only) currently rendering a centered "bludio" label. It uses `gpui-unofficial` v1.2.7 with `gpui-platform-gpui-unofficial` for X11/Wayland. The app shell spec is complete. This change adds the first real feature: Bluetooth device management via BlueZ D-Bus.

The reference implementation (`bluetooth-bluez-reference.md`) documents how BlueZ exposes adapters, devices, discovery, and pairing over the system D-Bus. We follow the recommended approach of using the `bluer` crate (async, high-level BlueZ wrapper built on `zbus`).

## Goals / Non-Goals

**Goals:**
- Integrate with BlueZ over D-Bus to list, scan, connect, disconnect, pair+trust, and forget Bluetooth devices
- Provide a minimal, non-designed scrolling list UI to exercise all Bluetooth operations
- Split code into well-organized modules with clear separation between Bluetooth backend and GPUI presentation
- Follow Zed/GPUI patterns: `Entity` for state, `cx.spawn()` for async, D-Bus signals for reactivity

**Non-Goals:**
- Visual polish, theming, or final UI design (will come later)
- Audio subsystem integration (future change)
- GATT service/characteristic browsing
- Bluetooth LE advertisement or beacon support beyond scanning device presence
- Cross-platform support (Linux + BlueZ only)
- Persistent configuration or device caching across sessions

## Decisions

### 1. Bluetooth crate: `bluer` over raw `zbus`

**Choice**: `bluer` (with `zbus` as transitive dependency)
**Rationale**: `bluer` provides idiomatic async wrappers for Session, Adapter, Device, and discovery events — eliminating boilerplate D-Bus proxy definitions. It is the most complete Rust BlueZ library. Raw `zbus` `#[proxy]` macros are more verbose and error-prone for complex BlueZ workflows.
**Alternatives considered**: `zbus` alone (too low-level), `bluez-async` (less maintained, narrower scope).

### 2. Module structure

```
src/
├── main.rs              # App shell: Tokio bridge, BludioApp, Render, discovery orchestration
├── device_list.rs       # GPUI UI component: scrolling list + action buttons + Action enum
├── actions.rs           # Device action execution (connect/disconnect/forget/pair+trust)
├── bluetooth/           # Bluetooth backend module
│   ├── mod.rs           # BluetoothState, list_devices, resolve_display_name
│   ├── device.rs        # BluetoothDevice data model + sort_devices
│   ├── agent.rs         # Minimal pairing agent (auto-accept, KeyboardDisplay)
│   ├── discovery.rs     # Scan loop, fetch_device_info, refresh_device_list
│   ├── monitor.rs       # Per-device D-Bus signal listener for instant updates
│   └── properties.rs    # Shared property fetching with configurable timeouts
```

**Rationale**: Clear boundary between D-Bus logic (`bluetooth/`), UI (`device_list.rs`),
and cross-cutting action orchestration (`actions.rs`). `properties.rs` eliminates
code duplication across four device-info-fetching call sites. `discovery.rs` isolates
the long-running scan loop + post-scan refresh.

### 3. State management: `BluetoothState` on `BludioApp` entity

**Choice**: `BluetoothState` is a direct field on `BludioApp` (the GPUI entity).
`replace_devices()`, `upsert_device()`, and `remove_device()` methods on
`BluetoothState` enforce the invariant that the device list is always sorted
after any mutation.
**Rationale**: gpui-unofficial 1.2.7 does not have `Model<T>` — entities own their
state directly. The sort invariant is enforced by the type rather than relying on
call sites to remember `sort_devices()`.

### 4. Scrolling list: div children with `overflow_y_scroll`

**Choice**: Regular GPUI div children wrapped in `overflow_y_scroll`.
**Rationale**: `uniform_list` requires per-item rendering via a closure that
receives `(&mut Window, &mut App)` — no `Context<T>` available for `cx.listener()`
callbacks on interactive buttons. This is a gpui-unofficial API limitation.
Regular children with overflow scroll work correctly for the ~20 device scale
this testing tool targets.

### 5. Pairing agent: minimal auto-accept

**Choice**: Implement `org.bluez.Agent1` with `KeyboardDisplay` capability, auto-accepting all passkey confirmations and PIN requests.
**Rationale**: The goal is testing device interactions, not building a secure pairing UI. Auto-accept reduces complexity. A proper user-facing agent dialog can be added later.
**Risk**: Auto-accepting pairing requests has security implications. This is acceptable for a developer tool; will be replaced before production use.

### 6. Async execution: `tokio` via `cx.spawn()` with a dedicated runtime bridge

**Choice**: All D-Bus calls run on a dedicated Tokio runtime, bridged to GPUI's
async context through `tokio_task()` (spawn-on-Tokio → oneshot → await-on-GPUI).
**Rationale**: `bluer` requires a Tokio 1.x reactor, but gpui-unofficial uses its
own executor. A global `LazyLock<tokio::runtime::Runtime>` is created at startup
for all BlueZ work. GPUI's `cx.spawn()` still orchestrates the flow — it calls
`tokio_task()` to dispatch work, then updates the model.

## Risks / Trade-offs

- **[D-Bus/system bus availability]** → At startup, attempt to create a `bluer::Session`. If it fails (BlueZ not running or permission denied), show an error state in the UI rather than crashing.
- **[`bluer` API stability]** → Pin a specific version in `Cargo.toml`. The crate is actively maintained and used in production (e.g., `overskride`).
- **[Scanning drains adapter resources]** → Implement a scan timeout (30s default) and a manual stop button. Re-use the same discovery session rather than starting/stopping repeatedly.
- **[Pairing agent lifecycle]** → Register the agent once at startup. If BlueZ restarts or the agent is unregistered, the next pairing attempt will fail with a descriptive error shown in the UI.
- **[Linux-only]** → The app only works where BlueZ is available. Document this clearly. No `#[cfg]` shenanigans needed since the app shell is already Linux-only.
