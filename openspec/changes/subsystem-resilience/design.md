## Context

Bludio depends on two external subsystems: **BlueZ** (Bluetooth via D-Bus) and **PulseAudio/PipeWire** (audio via `libpulse-binding`). Both can be independently restarted by the user or the system (e.g., `systemctl restart bluetooth`, PipeWire upgrade, PA crash). When this happens today:

- **Bluetooth**: The `bluer::Session` and `Adapter` handles become stale. D-Bus calls silently fail or error. The monitor loop may exit. The UI shows stale device data with no indication of failure.
- **Audio**: The PA mainloop thread exits. The `tokio::sync::mpsc` channel from PA→GPUI closes. The `AudioState` stops updating. The UI freezes on the last known state with no error indication.

Neither subsystem has reconnection logic. The app must be restarted to recover.

### Current architecture (relevant parts)

```
BludioApp
├── bt_state: BluetoothState { session, adapter, devices, error }
├── audio_state: AudioState { sinks, sources, cards, connected, error }
├── spawn_audio_state_loop() — reads from audio_state_rx channel
├── spawn_bluetooth_init() — one-shot init + monitor loop
└── spawn_bluetooth_command_handler() — dispatches actions
```

- PA thread: `std::thread` running `run_pa_loop()` → sends `AudioState` snapshots via `mpsc::UnboundedSender`
- Bluetooth: Tokio tasks via `tokio_task()` → `oneshot::Receiver` for results
- Monitor: `run_monitor()` spawns per-device D-Bus event listeners, re-syncs device set every 5s

## Goals / Non-Goals

**Goals:**
- Detect when BlueZ or PulseAudio becomes unavailable and display a clear error state in the UI
- Automatically reconnect when the subsystem comes back online
- Use reactive detection (D-Bus signals) where possible; use bounded backoff polling only where no reactive mechanism exists
- Preserve existing UI patterns (error banners, loading states, page-level error display)
- Keep the app functional during subsystem outages — no crashes, no stale data displayed as current

**Non-Goals:**
- Persisting device state across restarts (e.g., remembering which devices were connected)
- Handling adapter hardware changes (e.g., USB Bluetooth dongle removed)
- Supporting multiple Bluetooth adapters
- Migrating from PulseAudio to PipeWire's native API
- Providing a manual "reconnect" button (automatic reconnection covers this)

## Decisions

### Decision 1: Bluetooth disconnect detection via D-Bus `NameOwnerChanged` + adapter property monitoring

**Choice**: Subscribe to D-Bus `NameOwnerChanged` signals for `org.bluez` on the session bus. Additionally, monitor adapter-level D-Bus property changes (specifically `Powered`) to detect rfkill block/unblock events.

**Why**: BlueZ registers on D-Bus as `org.bluez`. When the service restarts, D-Bus emits `NameOwnerChanged` with an empty owner (lost) and then a new owner (appeared). This is fully reactive — no polling. The `bluer` crate exposes this via `Session::name_owner_changed_signal()` or we can use the lower-level `zbus` signal directly.

However, `NameOwnerChanged` only fires when the BlueZ daemon starts/stops. It does NOT fire for rfkill block/unblock (`rfkill block bluetooth` / `rfkill unblock bluetooth`) because BlueZ remains running — only the adapter's `Powered` property changes. To detect this reactively, the monitor must also watch adapter-level D-Bus property changes via `adapter.events()` (which emits `AdapterProperty` events including `Powered(bool)`).

**Alternatives considered**:
- *Polling `session.default_adapter()` on a timer*: Simpler but introduces latency and unnecessary D-Bus traffic. Rejected because a reactive signal exists.
- *Catching errors from existing D-Bus calls*: Would only detect the issue when we next try to use the adapter, which could be arbitrarily delayed. Rejected because it's not proactive.
- *Monitor the `run_monitor` task exit*: The monitor task may exit when BlueZ goes down, but it doesn't distinguish between "monitor bug" and "BlueZ gone." Not reliable as sole detection. Used as a secondary signal.
- *Relying on the 5s monitor re-sync for rfkill*: Would work (devices disappear/reappear) but has up to 10s latency and doesn't surface a clear error state. Rejected in favor of reactive detection.

**Implementation sketch**:
1. In `spawn_bluetooth_init()`, after successful init, spawn a `NameOwnerChanged` watcher task.
2. In `run_monitor()`, add an adapter-level event listener alongside the per-device listeners. Watch for `AdapterProperty::Powered(false)` → signal disconnect, `AdapterProperty::Powered(true)` → signal reconnect.
3. When the signal indicates `org.bluez` lost its owner OR adapter powers off → set `bt_state.subsystem_status = Disconnected(...)`, notify pages.
4. When the signal indicates `org.bluez` gained a new owner OR adapter powers on → enter reconnecting state, re-attempt `BluetoothState::new()`.
5. On successful reconnect, re-register the agent, resume the monitor loop.

### Decision 2: Audio disconnect detection via channel closure + PA context state

**Choice**: Detect PA thread exit by observing `audio_state_rx` channel closure. Detect PA context failure within the thread by checking `ContextState::Failed | Terminated`.

**Why**: The PA thread runs `run_pa_loop()` which returns (and drops the sender) when the mainloop exits or the context enters a failed state. The `spawn_audio_state_loop()` in `app.rs` can detect `rx.recv()` returning `None` (channel closed) as a definitive signal that PA is gone.

**Alternatives considered**:
- *D-Bus NameOwnerChanged for PulseAudio*: PA doesn't always register on the session D-Bus (especially when running under PipeWire). PipeWire's D-Bus name varies by distribution. Not reliable. Rejected.
- *Checking PA context state from GPUI thread*: Would require unsafe pointer sharing across threads. Violates the current architecture where PA state is only accessed from the PA thread. Rejected.
- *Heartbeat polling from GPUI to PA thread*: Adds complexity and still requires the thread to be alive to respond. Channel closure is a cleaner signal. Rejected.

**Implementation sketch**:
1. In `spawn_audio_state_loop()`, when `rx.recv()` returns `None` → set `audio_state.connected = false`, set `audio_state.error`, notify pages.
2. In `run_pa_loop()`, if `ContextState::Failed` or `Terminated` is observed during the event loop, send a final `AudioState` with `connected: false` and `error: Some(...)`, then break.
3. Spawn a reconnect loop in `app.rs` that attempts to restart the PA thread on a backoff schedule.

### Decision 3: Audio reconnection via short fixed-interval retry (polling)

**Choice**: When audio disconnects, spawn a reconnect task that attempts to create a new PA thread on a short fixed interval (3 seconds between attempts, no exponential growth). Reset immediately on successful connection.

**Why**: Unlike Bluetooth (which has D-Bus `NameOwnerChanged`), there is no universal reactive signal for "PulseAudio is available again." PipeWire/PA may or may not register on D-Bus. The most reliable detection is to attempt a connection. A short fixed interval ensures the app recovers within a few seconds of the subsystem coming back — PipeWire restarts typically complete in 1–2 seconds, so a 3s interval means we catch it on the first or second attempt. Exponential backoff is unnecessary here because: (a) the failure mode is a clean restart, not a flaky connection, (b) the retry cost is negligible (one PA connect attempt), and (c) users expect near-instant recovery.

**Alternatives considered**:
- *Exponential backoff (1s → 30s)*: Overly conservative for a subsystem restart scenario. A 30s wait after PipeWire is already back up is poor UX. Rejected.
- *inotify on PulseAudio socket*: Fragile, platform-specific, doesn't cover PipeWire. Rejected.
- *D-Bus signal for PipeWire*: PipeWire's D-Bus interface varies. Not all installations use it. Rejected.
- *User-triggered reconnect*: Poor UX. The user shouldn't need to know the internals. Rejected.

**Implementation sketch**:
1. `spawn_audio_state_loop()` detects channel closure.
2. Enter a `reconnect_audio()` loop: attempt `run_pa_thread_from_channels()`, wait for wakeup handle with a 2s timeout.
3. If wakeup received → success, exit reconnect loop, wire new channels into pages.
4. If timeout (2s) → failure, sleep 3s, retry.
5. During reconnecting, pages show "Reconnecting to PulseAudio..." state.

### Decision 4: SubsystemStatus enum for unified UI states

**Choice**: Introduce a `SubsystemStatus` enum used by both subsystems. This replaces ALL boolean/option combinations (`connected`, `error`, `initialized`) with a single state machine. Pages match on the enum to decide what to render.

```rust
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SubsystemStatus {
    /// Initial app startup — subsystem is connecting for the first time.
    Connecting,
    /// Connected and operational.
    Connected,
    /// Subsystem is down. Contains a user-facing error message.
    Disconnected(String),
    /// Attempting to reconnect after a disconnect.
    Reconnecting,
}
```

**Why**: Both Bluetooth and Audio pages need to display the same set of states (connecting, connected, disconnected with error, reconnecting). A shared enum ensures consistent behavior and makes it easy for the UI to render the right state without ad-hoc boolean combinations. The `Connecting` variant replaces the `!initialized && error.is_none()` pattern on the Bluetooth page and the `!connected` pattern on the Audio page — everything is a single `match` on the enum.

**Alternatives considered**:
- *Separate booleans (`connected`, `reconnecting`, `error`)*: Error-prone combinations (what if `connected=true` and `error=Some(...)`?). The existing code uses `!initialized && error.is_none()` which is already confusing. Rejected.
- *Per-subsystem status types*: Duplicates the same concept. Rejected.
- *Keeping `initialized` bool + `error` option alongside the enum*: Defeats the purpose — two sources of truth for the same concept. Rejected.

### Decision 5: Reconnection rewire pattern — channel recreation

**Choice**: On reconnect, create fresh `mpsc` channels and re-assign them to the page entities. The old channels are dropped.

**Why**: The PA thread owns the sender end. When it dies, the sender is dropped. Creating a new thread means new channels. The pages hold cloned senders, so they need to receive the new sender. This is cleaner than trying to keep channels alive across thread restarts.

**For Bluetooth**: The `bluer::Session` is re-created from scratch. The `BluetoothPageCommand` channel (futures mpsc) survives because it's owned by the app, not BlueZ. The monitor task is re-spawned with the new adapter.

**Implementation sketch**:
1. `BludioApp` holds a method `reconnect_audio(&mut self, window, cx)` that:
   - Creates new `(cmd_tx, cmd_rx)` and `(state_tx, state_rx)` channels
   - Spawns new PA thread
   - Updates `audio_cmd_tx` in `self`, `audio_output_page`, `audio_input_page`, `configuration_page`
   - Spawns new `audio_state_loop` with the new `state_rx`
2. For Bluetooth, `reconnect_bluetooth(&mut self, window, cx)` re-runs the init sequence and re-spawns the monitor.

### Decision 6: UI rendering — page-level full-state replacement

**Choice**: When a subsystem is disconnected, the page renders a full-area error/reconnecting message in the **page content area** (the same region used for "Connecting to..." loading states) instead of the device list. When reconnected, the device list reappears with fresh data.

**Why**: Showing stale device data during a disconnect is misleading. A full-area message (reusing the existing "Connecting to..." pattern) is clear and honest. The transition back to the device list happens automatically when `sync_state()`/`sync_rows()` is called with fresh data from the reconnected subsystem.

**Important**: The error banner component (`error_banner`) is NOT used for subsystem-level disconnect/reconnecting states. The banner is reserved for transient device-level action errors (e.g., "Failed to connect to device X") while the subsystem is connected. Subsystem health states use the page content area because they represent a fundamentally different situation — the entire backend is unavailable, not a single action failure.

**Existing patterns replaced**:
- Bluetooth page: `!initialized && error.is_none()` → "Connecting to Bluetooth..." — replaced by `match status { Connecting => ... }`
- Bluetooth page: `error.clone()` → error display — replaced by `match status { Disconnected(msg) => ... }`
- Audio page: `!connected` → "Connecting to PulseAudio..." — replaced by `match status { Connecting => ... }`
- Audio page: `self.error.clone()` → error display — replaced by `match status { Disconnected(msg) => ... }`

All rendering is driven by a single `match` on `SubsystemStatus`. No booleans, no `.is_none()` checks.

## Risks / Trade-offs

**[Risk] Reconnection storm if subsystem is unstable** → Mitigation: Fixed 3s retry interval with no exponential growth. Only retries while disconnected. If the subsystem is truly down for an extended period, the retry cost is one PA connect attempt every 3 seconds — negligible.

**[Risk] Race between reconnect and user action** → Mitigation: During `Reconnecting` state, command channels are not yet wired. Actions sent during this window will fail silently (the old channel sender is dropped). The UI should disable action buttons in the reconnecting state. Alternatively, buffer commands — but this adds complexity for a rare case. **Decision**: Disable actions during reconnecting. The state is transient (< 2s typically).

**[Risk] PA thread restart changes device indices** → Mitigation: PA sink/source indices are not stable across PA restarts anyway. The full state rebuild on reconnect replaces all device data. No index persistence is assumed.

**[Risk] `bluer::Session::name_owner_changed_signal()` API availability** → Mitigation: Verify the API exists in the current `bluer` version. If not, fall back to spawning a `zbus` proxy or monitoring the existing session's adapter handle for errors (secondary detection).

**[Trade-off] Channel recreation vs. channel wrapping**: Recreating channels is simpler but requires updating all page entities with new senders. Wrapping in `Arc<Mutex<Option<Sender>>>` avoids this but adds lock contention and complexity. **Chosen**: Channel recreation — it happens rarely (on reconnect) and the update is a single `page.update()` call per page.
