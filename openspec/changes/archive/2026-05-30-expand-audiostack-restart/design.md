## Context

The app currently exposes a single bottom-of-tab-bar action button labeled "Restart Bluetooth" (covered by the `privileged-operations` spec). When clicked, it spawns `pkexec /usr/bin/systemctl restart bluetooth` via `tokio::process::Command`. The command result is piped back to the GPUI context and, on failure, shown in the Bluetooth page's error banner.

The audio stack on modern Linux distributions (Fedora, Ubuntu, Arch with PipeWire) consists of multiple user-level and system-level services:
- **User**: `wireplumber`, `pipewire`, `pipewire-pulse`
- **System**: `bluetooth`
- **Hardware unblock**: `rfkill unblock bluetooth`

Users who need to recover from a broken audio/Bluetooth state currently must run these commands manually. The goal is to make the existing button trigger the full recovery sequence.

## Goals / Non-Goals

**Goals:**
- Execute the three-step restart sequence in order when the user clicks the action button.
- Rename the action and UI labels to reflect the broader scope.
- Surface per-step errors in the existing Bluetooth page error banner.
- Re-use the existing `run_pkexec` helper and command-spawning pattern.

**Non-Goals:**
- Adding a progress indicator or multi-step UI state.
- Changing existing Bluetooth page requirements or device-list behavior.
- Persisting restart history or retry logic beyond the existing reconnect loops.
- Supporting non-systemd init systems.
- Modifying `subsystem-health` reconnection logic — the existing 3-second audio retry and BlueZ `NameOwnerChanged` detection already handle the recovery.

### References
- `privileged-operations` spec: defines the pkexec-based restart command, error mapping, and button disabled-during-execution behavior.
- `subsystem-health` spec: defines the disconnect/reconnect flows that will be triggered by PipeWire and BlueZ restarts.

## Decisions

### 1. Concurrent spawn with sequential result collection and fail-continue behavior
**Decision:** Spawn all three commands concurrently on the Tokio runtime, then collect their results sequentially in order. If a step fails, record the error and continue to the next result.
**Rationale:** The commands target independent subsystems (PipeWire user stack, BlueZ system service, rfkill) and do not depend on each other. Concurrent spawning is faster. Results are still collected in the defined order so error messages align with the command sequence. A failure in one (e.g. rfkill not installed) does not prevent the others from running. The user gets a consolidated error message listing all failures.
**Alternative considered:** Fail-fast — rejected because a missing `rfkill` binary should not abort the Bluetooth service restart.

### 2. Unified spawn helpers with correct privileges
**Decision:** Add a `run_command` helper for unprivileged commands alongside the existing `run_pkexec` helper. Both return the same `Result<(), PkexecError>` type so the calling code can collect results uniformly into a `Vec`. Command 1 (`systemctl --user restart ...`) and command 3 (`rfkill unblock bluetooth`) run via `run_command`. Command 2 (`systemctl restart bluetooth`) runs via `run_pkexec`.
**Rationale:** Keeps the calling code clean — a single `results` vec and one error-reporting path. The privilege model is still correct: user services run as the current user, system commands run via pkexec.
**Alternative considered:** Single `pkexec bash -c "..."` — rejected because it obscures per-step exit codes and stderr, making error attribution harder.

### 3. Rename action_id and update icon
**Decision:** Update the `TabAction` definition in `app.rs`: action_id `"restart-audiostack"`, tooltip `"Restart Audio Stack"`, icon `refresh-ccw`.
**Rationale:** Prevents confusion. The tooltip, icon, and action ID are defined alongside the `TabBar` instantiation in `BludioApp::new`.

## Risks / Trade-offs

- **[Risk]** PipeWire restart causes the PA thread's PulseAudio connection to drop, leading to temporary UI unavailability.
  → **Mitigation:** The existing PA thread already has a reconnect loop with a 1-second backoff. No new code is required.
- **[Risk]** `rfkill` may not be installed on all target systems.
  → **Mitigation:** Continue on failure; surface "rfkill not found" in the error banner so the user knows the command was skipped.
- **[Risk]** Rapid sequential commands may race with D-Bus service startup, causing the Bluetooth monitor to emit transient errors.
  → **Mitigation:** The existing Bluetooth monitor has a 10-second fallback refresh and a reconnect retry loop that catches BlueZ restarts. No additional delay is injected.
