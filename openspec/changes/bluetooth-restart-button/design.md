## Context

Bludio manages Bluetooth devices via BlueZ D-Bus (through `bluer`). When the Bluetooth service enters a bad state, users must drop to a terminal and run `sudo systemctl restart bluetooth`. This change adds a one-click restart button that delegates authentication to PolicyKit via `pkexec`, so Bludio never sees or handles passwords.

The tab bar (`TabBar`) is a self-contained GPUI entity rendering a vertical column of icon tabs. It currently has no concept of "action buttons" separate from navigation tabs.

## Goals / Non-Goals

**Goals:**
- Add a "Restart Bluetooth" button anchored to the bottom of the tab bar
- Execute `pkexec dev.toomosin.bludio.restart-bluetooth-service` asynchronously
- Ship a PolicyKit policy file so pkexec knows the action is allowed
- Surface authentication cancellation and command failures to the user

**Non-Goals:**
- General-purpose command runner or terminal emulator
- Password caching or session-level auth state management
- Supporting multiple privileged operations (only bluetooth restart for now)
- Custom in-app password dialog (pkexec uses the system's polkit agent)

## Decisions

### 1. Where does the button live?

**Decision:** Extend `TabBar` to support bottom-anchored action items.

The `TabBar` already owns the full-height left column. Adding an `actions` field (rendered after a `flex_spacer()` push to the bottom) keeps the layout in one place rather than having the app manually compose a button below the tab bar entity.

**Alternative considered:** Render the button as a separate entity below `TabBar` in `BludioApp::render()`. Rejected because it splits the left-column layout across two locations and requires manual height coordination.

**Shape:**
```
┌──────────────────────────┐
│  [tabs]                  │
│  ...                     │
│                          │
│  ──────── flex_spacer ── │  ← pushes actions to bottom
│                          │
│  [restart button]        │  ← action item
└──────────────────────────┘
```

### 2. How is pkexec invoked?

**Decision:** `tokio::process::Command::new("pkexec").arg(action_id).output().await`

Run on the existing Tokio runtime via `cx.spawn_in()` (same pattern as all other async operations). The action ID is a string constant matching the policy file.

pkexec exit codes:
- `0` → success
- `126` → user cancelled authentication
- `127` → action not found or not authorized

### 3. How are errors surfaced?

**Decision:** Use the existing `error-banner` capability. The Bluetooth page already has `show_error()`. On pkexec failure, push a message through the error banner on the bluetooth page (since that's the context the user is in when they click the button).

The button lives in the `TabBar` which doesn't have direct access to the bluetooth page's error banner. The flow is:

```
TabBar emits ActionButtonClicked(id)
  → BludioApp handles it
    → spawns pkexec via tokio::process::Command
    → on failure, calls bluetooth_page.show_error(msg, cx)
```

### 4. Policy file installation

**Decision:** Ship `dev.toomosin.bludio.policy` in a `policy/` directory at the repo root. The PKGBUILD installs it to `/usr/share/polkit-1/actions/`.

### 5. Icon choice

**Decision:** Use `icons::bluetooth` as placeholder. The user will replace it with an appropriate restart/system icon later.

## Risks / Trade-offs

**[pkexec not installed]** → Mitigation: The PKGBUILD declares `polkit` as a dependency. If somehow missing, the error banner will show "pkexec not found".

**[User has no polkit agent running]** → Mitigation: On headless/minimal setups without a DE, pkexec will fail with exit code 127. The error message will indicate auth is unavailable. This is acceptable — Bludio targets desktop Linux users.

**[Button is disabled during execution]** → The button should be disabled while a pkexec call is in-flight to prevent double-invocations. This requires tracking an `in_progress` state in `TabBar` or `BludioApp`.

**[Policy file not installed]** → If the user runs from `cargo run` without installing, pkexec will fail with "not authorized". The error banner will guide them to install the policy file.
