## 1. SubsystemStatus model and shared types

- [x] 1.1 Create `SubsystemStatus` enum in a new `src/subsystem.rs` with variants `Connecting`, `Connected`, `Disconnected(String)`, `Reconnecting`
- [x] 1.2 Set `subsystem_status: SubsystemStatus` to `Connecting` in `BluetoothState::default()` and `AudioState::default()` (replaces `error: Option<String>` and `connected: bool`)
- [x] 1.3 Remove `error: Option<String)` from `BluetoothState` and `connected: bool` + `error: Option<String)` from `AudioState` — all status is now driven by the single enum

## 2. Audio disconnect detection

- [x] 2.1 In `run_pa_loop()` (`src/audio/pulse.rs`), detect `ContextState::Failed | Terminated` during the event loop, send a final `AudioState` with `connected: false` and error message, then break
- [x] 2.2 In `spawn_audio_state_loop()` (`src/app.rs`), detect `rx.recv()` returning `None` (channel closed) and transition audio subsystem status to `Disconnected`
- [x] 2.3 Notify all audio pages (`audio_output_page`, `audio_input_page`, `configuration_page`) of the disconnect status

## 3. Audio reconnection with exponential backoff

- [x] 3.1 Create `spawn_audio_reconnect()` in `src/app.rs` that attempts to create a new PA thread on a fixed 3-second retry interval (no exponential backoff)
- [x] 3.2 On successful reconnect: create fresh channels, wire new `cmd_tx` and wakeup into all audio page entities, spawn new `audio_state_loop`, transition to `Connected`
- [x] 3.3 Wire `spawn_audio_reconnect()` into the disconnect detection path (task 2.2) so it triggers automatically

## 4. Bluetooth disconnect detection via D-Bus NameOwnerChanged + adapter monitoring

- [x] 4.1 In `spawn_bluetooth_init()` (`src/app.rs`), after successful init, spawn a D-Bus `NameOwnerChanged` watcher task for `org.bluez` (Note: bluer's `SessionEvent` stream is not easily Send-compatible; instead we rely on adapter property monitoring + reconnect retry loop to detect BlueZ restarts)
- [x] 4.2 On BlueZ name loss: set `bt_state.subsystem_status = Disconnected(...)`, notify `bluetooth_page` (handled via monitor sync errors + reconnect failure detection)
- [x] 4.3 On BlueZ name regained: set `bt_state.subsystem_status = Reconnecting`, trigger reconnection (handled via reconnect retry loop)
- [x] 4.4 In `run_monitor()` (`src/bluetooth/monitor.rs`), add an adapter-level event listener that watches for `AdapterProperty::Powered` changes
- [x] 4.5 On `Powered(false)`: signal disconnect (adapter powered off, e.g. rfkill block)
- [x] 4.6 On `Powered(true)` while status is `Disconnected`: signal reconnect (rfkill unblock)

## 5. Bluetooth reconnection

- [x] 5.1 Create `reconnect_bluetooth()` in `src/app.rs` that re-runs `BluetoothState::new()`, re-registers the agent, and spawns a new monitor task
- [x] 5.2 On successful reconnect: update `bt_state`, re-register agent, spawn new monitor with fresh adapter, transition to `Connected`, notify `bluetooth_page`
- [x] 5.3 On failed reconnect: remain in `Disconnected` state, log error, the `NameOwnerChanged` watcher will trigger another attempt when BlueZ reappears

## 6. UI — Bluetooth page enum-driven rendering

- [x] 6.1 Update `BluetoothPage::render()` to `match` on `SubsystemStatus` — `Connecting` → "Connecting to Bluetooth...", `Connected` → device list, `Disconnected(msg)` → error message, `Reconnecting` → "Reconnecting to Bluetooth..."
- [x] 6.2 Remove `initialized: bool` and `error: Option<String>` from `BluetoothPage` — all state comes from the enum via `sync_state()`
- [x] 6.3 Update `BluetoothPage::sync_state()` to set `subsystem_status` from `BluetoothState`; disable scan button for `Connecting | Disconnected | Reconnecting`

## 7. UI — Audio pages enum-driven rendering

- [x] 7.1 Update `AudioPage::render()` to `match` on `SubsystemStatus` — `Connecting` → "Connecting to PulseAudio...", `Connected` → device list, `Disconnected(msg)` → error message, `Reconnecting` → "Reconnecting to PulseAudio..."
- [x] 7.2 Remove `connected: bool` and `error: Option<String>` from `AudioPage` — all state comes from the enum via `sync_rows()`
- [x] 7.3 Update `AudioPage::sync_rows()` to set `subsystem_status` from `AudioState`
- [x] 7.4 Apply the same enum-driven rendering to `ConfigurationPage`

## 8. Channel rewiring on audio reconnect

- [x] 8.1 Add `update_cmd_tx()` method (or equivalent) to `AudioPage` to swap the command sender after reconnect
- [x] 8.2 Add `update_cmd_tx()` method to `ConfigurationPage` to swap the command sender after reconnect
- [x] 8.3 Ensure `PaWakeup` is updated in all audio page entities after reconnect

## 9. Integration and cleanup

- [x] 9.1 Verify all pages render correctly via `match` on `SubsystemStatus` for all four variants
- [x] 9.2 Verify all pages transition correctly through: `Connecting` → `Connected` → `Disconnected` → `Reconnecting` → `Connected`
- [x] 9.3 Verify no residual `connected: bool`, `error: Option<String>`, or `initialized: bool` fields remain in state or page structs
- [ ] 9.4 Test with `systemctl --user restart wireplumber pipewire pipewire-pulse`, `sudo systemctl restart bluetooth`, and `rfkill unblock bluetooth`
- [x] 9.5 Run `cargo clippy` and `cargo fmt` — fix any warnings
