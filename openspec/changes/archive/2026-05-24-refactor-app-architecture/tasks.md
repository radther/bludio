## 1. Move Bluetooth utility functions to `bluetooth/`

- [x] 1.1 Move `execute_device_action` from `app.rs` to `bluetooth/device.rs` as `pub(crate) async fn`; update `app.rs` import
- [x] 1.2 Move `quick_device_status` from `app.rs` to `bluetooth/device.rs` as `pub(crate) async fn`; update `app.rs` import
- [x] 1.3 Move `devices_changed` from `app.rs` to `bluetooth/device.rs` as `pub(crate) fn`; update `app.rs` import
- [x] 1.4 Move `run_discovery` from `app.rs` to `bluetooth/discovery.rs` as `pub(crate) async fn`; update `app.rs` import

## 2. Unify Bluetooth channels to `futures::channel::mpsc`

- [x] 2.1 Switch `bluetooth::discovery::start_scan` return type from `tokio::sync::mpsc::UnboundedReceiver` to `futures::channel::mpsc::UnboundedReceiver<DiscoveryEvent>`; use `futures::channel::mpsc::unbounded()` inside the Tokio-spawned task
- [x] 2.2 Switch `bluetooth::monitor::run_monitor` to use `futures::channel::mpsc::UnboundedSender<bluer::Address>` instead of `tokio::sync::mpsc`
- [x] 2.3 Route `execute_device_action` D-Bus calls through `crate::tokio_task()` in the command handler (currently called directly without Tokio context)
- [x] 2.4 Remove all `crate::TOKIO.enter()` calls from Bluetooth code paths in `app.rs` (monitor loop and `run_discovery`)

## 3. Clean up `BludioApp` entity

- [x] 3.1 Remove dead `initialized: bool` field from `BludioApp` struct and its one assignment in the BT init task
- [x] 3.2 Remove unused `_text_secondary` binding from `BludioApp::render()`
- [x] 3.3 Extract the PA audio state loop from `new()` into a private `fn spawn_audio_state_loop()` method
- [x] 3.4 Extract the Bluetooth init + monitor loop from `new()` into a private `fn spawn_bluetooth_init()` method
- [x] 3.5 Extract the Bluetooth command handler from `new()` into a private `fn spawn_bluetooth_command_handler()` method

## 4. Verify

- [x] 4.1 `cargo build` — ensure compilation succeeds
- [x] 4.2 `cargo clippy` — ensure no new warnings
- [x] 4.3 `cargo fmt` — ensure consistent formatting
