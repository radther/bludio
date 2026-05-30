## Why

The current "Restart Bluetooth" action button only restarts the `bluetooth` system service. When the full audio stack gets into a bad state (e.g. PulseAudio/PipeWire conflicts, BlueZ adapter disappears, or rfkill soft-block), users must manually run multiple commands. Expanding the button to restart the complete audio stack in one click provides a single recovery action that matches common Linux audio troubleshooting workflows.

## What Changes

- Rename the tab-bar action from `restart-bluetooth` to `restart-audiostack` and update its tooltip/label.
- Change the command handler to execute a sequence of three commands in order:
  1. `systemctl --user restart wireplumber pipewire pipewire-pulse`
  2. `sudo systemctl restart bluetooth`
  3. `rfkill unblock bluetooth`
- Error handling: collect per-step failures and surface them in the Bluetooth page error banner.
- UI label/icon on the restart button updated to reflect the broader scope (`refresh-ccw` icon, "Restart Audio Stack" tooltip).

## Capabilities

### New Capabilities
*None — this is an enhancement to existing behavior with no new user-facing capabilities.*

### Modified Capabilities
- `privileged-operations`: The restart command requirement expands from Bluetooth-only to the full audio stack (PipeWire/WirePlumber user services, Bluetooth system service, and rfkill unblock). The button label and action ID change accordingly.
- `subsystem-health`: Restarting the audio stack will trigger existing disconnect/reconnect flows for both Audio and Bluetooth subsystems. No spec requirement changes, but the change exercises these paths.

## Impact

- `src/app.rs`: update `handle_restart_bluetooth` logic, rename action ID, adjust command sequence.
- `src/ui/tab_bar.rs`: update action button definition (icon, tooltip, action_id) if hard-coded there, or in `app.rs` where `TabAction` structs are constructed.
- `src/bluetooth/`: may need a brief adapter re-init delay after the sequence completes, or rely on existing reconnect loop.
- `src/audio/`: PulseAudio thread will detect the PipeWire/WirePlumber restart and reconnect via existing reconnection logic.
