## 1. Update TabBar action definition

- [x] 1.1 Rename `action_id` from `"restart-bluetooth"` to `"restart-audiostack"` in `app.rs`
- [x] 1.2 Update the action button tooltip from `"Restart Bluetooth"` to `"Restart Audio Stack"` in `app.rs`
- [x] 1.3 Update the action button icon to `refresh-ccw` in `app.rs`

## 2. Implement multi-step restart sequence

- [x] 2.1 Rename `handle_restart_bluetooth` to `handle_restart_audiostack` in `app.rs`
- [x] 2.2 Update the match arm in `TabBarEvent::ActionButtonClicked` to route `"restart-audiostack"` to the renamed handler
- [x] 2.3 Add a `run_command` helper (non-privileged) that returns `Result<(), PkexecError>` alongside the existing `run_pkexec` helper
- [x] 2.4 In the handler, spawn all three commands into a uniform `Vec<Result<(), PkexecError>>`:
  - `run_command` for `systemctl --user restart wireplumber pipewire pipewire-pulse`
  - `run_pkexec` for `systemctl restart bluetooth`
  - `run_command` for `rfkill unblock bluetooth`
- [x] 2.5 Iterate results and surface any failures in the Bluetooth page error banner

## 3. Verification

- [x] 3.1 Run `cargo build` to confirm no compilation errors
- [x] 3.2 Run `cargo clippy` and resolve any new warnings
- [x] 3.3 Run `cargo fmt` to ensure consistent formatting
