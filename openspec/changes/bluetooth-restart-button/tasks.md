## 1. PolicyKit Policy File

- [x] 1.1 Create `policy/dev.toomosin.bludio.policy` with the `restart-bluetooth-service` action (exec path `/usr/bin/systemctl`, args `restart bluetooth`, allow_active `auth_admin_keep`)
- [x] 1.2 Update PKGBUILD to install the policy file to `/usr/share/polkit-1/actions/`

## 2. TabBar Action Items

- [x] 2.1 Add `TabAction` struct (icon fn, tooltip, action_id) and `actions: Vec<TabAction>` field to `TabBar`
- [x] 2.2 Add `ActionButtonClicked(String)` variant to `TabBarEvent`
- [x] 2.3 Render action items at the bottom of the tab bar after a `flex_spacer()`, with visual separation from tabs
- [x] 2.4 Add `disabled: bool` support to action items — dimmed visual state, no click events when disabled
- [x] 2.5 Add `set_action_disabled(action_id, disabled)` method to `TabBar` for parent to toggle state

## 3. Restart Command Execution

- [x] 3.1 Add `RestartBluetooth` variant to `BluetoothPageCommand` in `src/ui/bluetooth/mod.rs`
- [x] 3.2 Create `restart_bluetooth_service()` async function that runs `pkexec dev.toomosin.bludio.restart-bluetooth-service` via `tokio::process::Command`
- [x] 3.3 Handle pkexec exit codes: 0 (success), 126 (cancelled), 127 (not authorized), other (stderr)
- [x] 3.4 Wire `TabBarEvent::ActionButtonClicked("restart-bluetooth")` in `BludioApp` to spawn the restart command

## 4. Error Handling & State

- [x] 4.1 Track `restart_in_progress: bool` in `BludioApp` and pass to `TabBar` via `set_action_disabled()`
- [x] 4.2 On pkexec failure, surface error message via `bluetooth_page.show_error()`
- [x] 4.3 On pkexec spawn failure (binary not found), surface appropriate error message

## 5. Integration

- [x] 5.1 Configure the restart action item in `BludioApp::new()` when creating the `TabBar`
- [x] 5.2 Verify button is disabled during execution and re-enables after completion
