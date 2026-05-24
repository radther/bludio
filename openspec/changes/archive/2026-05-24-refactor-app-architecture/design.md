## Context

`src/app.rs` is the application orchestrator — it owns `BludioApp`, creates page entities, and wires background subsystems (PulseAudio, Bluetooth). Over time it accumulated Bluetooth domain logic, four inline async blocks in the constructor, and fragile Tokio `EnterGuard` management. This refactor moves Bluetooth logic to the `bluetooth/` module, extracts constructor async blocks into named methods, cleans up dead code, and consolidates Tokio context management.

### Current channel landscape

| Subsystem | Channel type | Notes |
|---|---|---|
| Bluetooth discovery events | `tokio::sync::mpsc` | From `start_scan` (Tokio-spawned), received in GPUI |
| Bluetooth D-Bus monitor | `tokio::sync::mpsc` | From `run_monitor` (Tokio-spawned), received in GPUI |
| Bluetooth page commands | `futures::channel::mpsc` | Chosen to avoid `EnterGuard` nesting |
| Audio state updates | `tokio::sync::mpsc` | PA thread sends, GPUI receives |
| Audio commands | `tokio::sync::mpsc` | GPUI sends, PA thread receives via `try_recv()` |
| PA wakeup | `std::sync::mpsc` | One-shot: PA thread → GPUI |

The Bluetooth subsystem uses two different channel types for no reason other than `EnterGuard` avoidance. The Tokio mpsc discovery + monitor channels force `TOKIO.enter()` calls in GPUI tasks, which risk nesting panics when multiple tasks enter simultaneously.

### PulseAudio channel analysis

The PA thread (`std::thread`) uses `tokio::sync::mpsc` for both sending state and receiving commands. Critically, it only uses non-async operations:
- `UnboundedSender::send()` — works from any thread, no Tokio required
- `UnboundedReceiver::try_recv()` — non-blocking poll, no Tokio required

The GPUI-side PA state loop (`cx.spawn_in`) does `rx.recv().await`, which requires `TOKIO.enter()`. This is a **single, contained** EnterGuard — not the scattered, fragile pattern Bluetooth has. Moving PA to `futures::channel::mpsc` would add complexity (the PA thread needs `try_next()` from `TryStream`, which requires `Pin<&mut Self>` and extra trait imports) for negligible gain. **PA stays on `tokio::sync::mpsc`.**

## Goals / Non-Goals

**Goals:**
- Reduce `src/app.rs` from ~500 lines to ~150 lines (entity + render + init wiring)
- Extract the four inline async blocks from `new()` into named methods
- Move `run_discovery`, `execute_device_action`, `quick_device_status`, `devices_changed` into `bluetooth/`
- Remove the dead `initialized` field and `_text_secondary` binding
- Unify all Bluetooth channels to `futures::channel::mpsc`
- Eliminate `TOKIO.enter()` calls from all Bluetooth code
- Route `execute_device_action` D-Bus calls through `crate::tokio_task()` for proper Tokio context

**Non-Goals:**
- Changing PulseAudio channel types (stays on `tokio::sync::mpsc`)
- Modifying page entity architecture or ownership chain
- Changing any user-visible behavior
- Adding tests (out of scope for this refactor)
- Modifying icon placeholders (alpha cosmetic issue)
- Adding `Focusable` to BluetoothPage/AudioPage (needed for future interactive features, left for now)

## Decisions

### Decision 1: Extract async blocks into named `fn spawn_*` methods

**Chosen**: Four private methods on `BludioApp` that take `&WeakEntity<Self>`, `&mut Window`, `&mut Context<Self>` and call `cx.spawn_in(window, ...).detach()` internally.

```rust
impl BludioApp {
    fn new(window, cx) -> Self {
        let this = cx.weak_entity();
        // Create page entities + initial state first
        // Then wire background tasks:
        Self::spawn_audio_state_loop(&this, audio_state_rx, window, cx);
        Self::spawn_bluetooth_init(&this, window, cx);
        Self::spawn_bluetooth_command_handler(&this, bt_cmd_rx, window, cx);
        // ...
    }

    fn spawn_audio_state_loop(
        this: &WeakEntity<Self>,
        audio_state_rx: tmpsc::UnboundedReceiver<AudioState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) { ... }

    fn spawn_bluetooth_init(
        this: &WeakEntity<Self>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) { ... }

    fn spawn_bluetooth_command_handler(
        this: &WeakEntity<Self>,
        bt_cmd_rx: futures::channel::mpsc::UnboundedReceiver<BluetoothPageCommand>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) { ... }
}
```

**Alternatives considered:**
- Free functions outside `impl BludioApp`: Rejected — these are tightly coupled to BludioApp's fields (bt_state, pages, etc.) and access them via `this.update_in(cx, |this, ...| ...)`. Making them methods keeps the coupling explicit.
- Constructor does everything inline: Rejected (current state) — unreadable, untestable, violates single responsibility.

### Decision 2: Move Bluetooth functions to `bluetooth/` module

| Function | Destination | Rationale |
|---|---|---|
| `run_discovery` | `bluetooth/discovery.rs` | Orchestration above `start_scan`; takes `WeakEntity<BludioApp>` for UI updates |
| `execute_device_action` | `bluetooth/device.rs` | Pure Bluetooth domain logic (connect/disconnect/pair/forget) |
| `quick_device_status` | `bluetooth/device.rs` | Builds `BluetoothDevice` from adapter; already calls `properties::*` |
| `devices_changed` | `bluetooth/device.rs` | Device list comparison utility |

All moved functions retain their `pub(crate)` visibility. `run_discovery` takes a `WeakEntity<BludioApp>` parameter to push UI updates — this is the correct callback pattern, not a layering violation.

**Alternatives considered:**
- New `bluetooth/actions.rs` file for `execute_device_action`: Considered but rejected as overkill for a single function. `device.rs` is the right home (it already has `BluetoothDevice` and `sort_devices`).
- `quick_device_status` in `properties.rs`: Rejected — it builds a `BluetoothDevice`, not raw properties. `device.rs` is more appropriate.

### Decision 3: Unify Bluetooth channels to `futures::channel::mpsc` and eliminate `TOKIO.enter()`

**Chosen**: Switch all Bluetooth inter-task channels to `futures::channel::mpsc`. Route all D-Bus operations through `crate::tokio_task()` (which spawns on the global Tokio runtime). No GPUI-side Bluetooth task calls `TOKIO.enter()`.

Before:
```
start_scan ──tokio::mpsc──▶ run_discovery (TOKIO.enter() to recv)
run_monitor ─tokio::mpsc──▶ monitor loop (TOKIO.enter() to recv)
page rows ──futures::mpsc──▶ command handler (no Tokio, but calls execute_device_action directly)
```

After:
```
start_scan ──futures::mpsc──▶ run_discovery (no Tokio)
run_monitor ─futures::mpsc──▶ monitor loop (no Tokio)
page rows ──futures::mpsc──▶ command handler (no Tokio; dispatches D-Bus via tokio_task)
                                    │
                                    └──▶ tokio_task(execute_device_action) ──▶ TOKIO runtime
```

**Why `futures::channel::mpsc` over `tokio::sync::mpsc`:**
- `futures::channel::mpsc::UnboundedReceiver` implements `Stream` — `rx.next().await` works on any executor (including GPUI's) without Tokio context
- `futures::channel::mpsc::UnboundedSender::unbounded_send()` works from any thread, including Tokio-spawned tasks
- Already a dependency (used for BluetoothPageCommand)
- Eliminates the EnterGuard nesting risk entirely

**Changes in `start_scan`**: The internal `tokio::spawn` task continues to use Tokio (for `adapter.discover_devices()`), but creates a `futures::channel::mpsc::UnboundedSender` to send `DiscoveryEvent`s back to GPUI. The function signature changes from returning `tokio::sync::mpsc::UnboundedReceiver` to `futures::channel::mpsc::UnboundedReceiver`.

**Changes in `run_monitor`**: Same pattern — Tokio task internally, `futures::channel::mpsc::UnboundedSender` for output.

**Changes in `execute_device_action`**: Called from the command handler (GPUI executor, no Tokio context). Must be dispatched via `crate::tokio_task()` instead of being awaited directly:

```rust
BluetoothPageCommand::DeviceAction { addr, action } => {
    let a = adapter.clone();
    let _ = crate::tokio_task(async move {
        bluetooth::device::execute_device_action(&a, addr, action).await;
    }).await;
    // ... status update + full refresh (also via tokio_task)
}
```

**Alternatives considered:**
- Dedicated long-lived Tokio worker task with its own command channel: Rejected as overengineered. The existing `tokio_task()` infrastructure (spawn → oneshot) is sufficient and simpler.
- Keep mixed channels and just document the EnterGuard rules: Rejected — the current setup already caused panics and required defensive comments. Fix the root cause.

### Decision 4: Remove dead `initialized` field and `_text_secondary` binding

`BludioApp.initialized` is written once in the BT init task and never read. `BluetoothPage` derives its own `initialized` from `state.adapter.is_some()`. Remove it.

`_text_secondary` in `render()` is a let-binding prefixed with underscore to suppress the unused warning. Remove it.

### Decision 5: Audio stays on `tokio::sync::mpsc`

PA is a separate subsystem with its own thread, its own mainloop, and a single contained `TOKIO.enter()` on the GPUI side. The benefits of switching to `futures::channel::mpsc` are negligible. The PA thread's use of `try_recv()` (non-async) works perfectly with tokio mpsc. Leave it as-is.

## Risks / Trade-offs

- **[Risk] `futures::channel::mpsc` may have subtle behavioral differences from `tokio::sync::mpsc`** → Mitigation: Both are unbounded mpmc channels with identical semantics. `futures::channel::mpsc` is mature and widely used. The only difference is the async recv API (`Stream::next()` vs `.recv().await`), which is a drop-in replacement in async contexts.
- **[Risk] Moving `run_discovery` to `bluetooth/discovery.rs` introduces a dependency from `bluetooth/` back to `BludioApp`** → Mitigation: Already the case (`discovery.rs` has no knowledge of `BludioApp`; `start_scan` is pure). `run_discovery` takes a `WeakEntity<BludioApp>`, making the dependency explicit and unavoidable — Bluetooth needs to update the UI.
- **[Risk] `execute_device_action` currently works without `tokio_task()` — wrapping it may introduce latency** → Mitigation: The `tokio_task()` overhead is a single `oneshot` channel send+recv, sub-millisecond. D-Bus calls themselves are tens to hundreds of milliseconds. Negligible.
- **[Trade-off] Larger `bluetooth/device.rs`** → Acceptable. Currently ~30 lines (just struct + sort). Adding three functions (~60 lines) brings it to ~90 — still a focused module.

## Open Questions

- **Should `execute_device_action` handle errors?** Currently all D-Bus calls use `let _ = device.connect().await`. In a follow-up change, we could add error propagation and UI feedback. Out of scope for this refactor.
