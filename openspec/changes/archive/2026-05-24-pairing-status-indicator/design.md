## Context

The current pairing flow (`execute_device_action` with `PairAndTrust`) runs entirely on the Tokio runtime as a single blocking sequence: `pair()` → `set_trusted(true)` → `connect()`. The GPUI side awaits the entire operation via a `oneshot` channel, then does a single status refresh. There is no mechanism to communicate intermediate progress back to the UI.

The pairing chain involves three observable BlueZ D-Bus operations:
1. **Connect** (`Device1.Connect`) — establishes the ACL link
2. **Pair** (`Device1.Pair`) — performs authentication via the agent (passkey/confirmation), then marks the device as paired
3. **Set Trusted** (`Device1.Trusted := true`) — allows auto-reconnection

Each operation can succeed or fail independently. BlueZ's `Pair` method internally connects if needed, but calling `Connect` explicitly first gives us a clean "connecting" step that can be observed separately.

## Goals / Non-Goals

**Goals:**
- Show per-device status text ("connecting…", "pairing…", "trusting…") during the Pair & Trust operation
- Show a color-coded dot alongside the status text
- Log detailed failure reasons to the console (`eprintln!`)
- Show a brief "failed to pair" status in the UI on failure
- Clear the status indicator automatically once the device reaches a final state
- Each device independently tracks its own pairing status

**Non-Goals:**
- Animated/spinning dots (static colored dot is sufficient for now)
- Status indication for Connect-only or Disconnect actions (these are near-instant)
- Canceling an in-progress pairing operation
- A global pairing progress bar or toast notification
- Tracking intermediate agent steps (passkey entry, confirmation) — BlueZ's `Pair` call is opaque; we only see the overall pair outcome

## Decisions

### Decision 1: Add `PairingStatus` enum to `BluetoothDevice` model

**Rationale**: The device row already owns a copy of the device state (`paired`, `connected`, `display_name`). Adding `pairing_status: Option<PairingStatus>` directly to `BluetoothDevice` keeps the data with its subject and requires no new channels or synchronization primitives.

**Alternatives considered**:
- *Separate `HashMap<Address, PairingStatus>` in `BluetoothState`*: More indirection, harder to keep in sync during upserts/replaces.
- *Status updates via a separate `mpsc` channel*: Adds complexity; the existing command channel pattern is sufficient.

### Decision 2: Execute pairing in discrete steps from the command handler

**Rationale**: The command handler (`spawn_bluetooth_command_handler`) already runs in `cx.spawn_in(window, ...)`, which gives access to `AsyncWindowContext` for `update_in` calls. Instead of calling a monolithic `execute_device_action` and awaiting, we break the Pair & Trust flow into individual steps, calling `this.update_in(cx, ...)` between each step to push status updates to GPUI.

**Flow**:
```
1. update_in: set pairing_status = Connecting
2. tokio_task: device.connect().await
3. update_in: set pairing_status = Pairing
4. tokio_task: device.pair().await
5. update_in: set pairing_status = Trusting
6. tokio_task: device.set_trusted(true).await; device.connect().await
7. update_in: clear pairing_status + refresh device state
```

**Alternatives considered**:
- *Keep monolithic `execute_device_action` and use a callback channel*: More indirection; the command handler approach keeps all timing control in one place.
- *Spawn a separate task per status step*: Over-engineered; the steps are sequential by nature.

### Decision 3: Use `Option<PairingStatus>` — `None` means "no active operation"

**Rationale**: Clear semantics — `None` is the normal idle state where the row shows its usual connected/paired/discovered status. When `Some(...)`, the row overrides the status line with the pairing indicator.

### Decision 4: Status dot uses Unicode circle character (●) with Hsla color

**Rationale**: This matches the existing "● connected" indicator already used in `device_row.rs`. No new icon assets or svg imports needed. The color varies by status:
- Connecting: blue (accent) — `hsla(210/360, 0.7, 0.55, 1.0)`
- Pairing: yellow/amber — `hsla(45/360, 0.8, 0.55, 1.0)`
- Trusting: green — `hsla(140/360, 0.6, 0.5, 1.0)`
- Failed: red — `hsla(0/360, 0.7, 0.55, 1.0)`

### Decision 5: Failure clears on next device list refresh

**Rationale**: The monitor loop already refreshes device state on D-Bus signals and a 10s fallback. When the device list is refreshed and a device has `pairing_status: Some(Failed(...))`, we can clear it automatically. This avoids needing a separate timeout mechanism.

**Implementation**: In `BluetoothState::upsert_device` and `replace_devices`, existing failed status is cleared (the operation is no longer in progress). The `BluetoothDevice` model gets a helper `clear_pairing_status(&mut self)`.

## Risks / Trade-offs

- **Risk**: If the GPUI event loop is busy (e.g., during a complex render), intermediate status updates may be batched and the user might not see every step (e.g., "connecting" might flash by too fast to see). → **Mitigation**: This is acceptable — the purpose is to show *something* is happening, not to guarantee every sub-step is visible.
- **Risk**: If the Tokio runtime panics or the D-Bus connection drops mid-operation, the status might get stuck in "pairing…" until the next refresh cycle. → **Mitigation**: The 10-second fallback refresh will eventually clear it. This matches existing behavior for stale state.
- **Trade-off**: Breaking `execute_device_action` open in the handler makes the handler function longer. → **Acceptable**: The pairing flow is the only multi-step action; Connect/Disconnect/Forget remain one-liners.
