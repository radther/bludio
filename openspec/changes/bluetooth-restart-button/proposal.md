## Why

Bluetooth service failures sometimes require a manual restart, but users currently have no way to do this from within Bludio. Adding a restart button provides a self-service recovery path without leaving the app. Using `pkexec` delegates authentication to the system's PolicyKit agent, so Bludio never handles passwords.

## What Changes

- Add a "Restart Bluetooth" button to the tab bar, visually separated at the bottom (not grouped with navigation tabs)
- Button triggers `pkexec <action-id>` to restart the Bluetooth service via systemd
- Ship a PolicyKit policy file (`dev.toomosin.bludio.policy`) that defines the allowed action
- Button shows a tooltip ("Restart Bluetooth service") and uses an existing icon as placeholder
- PKGBUILD updated to install the policy file to `/usr/share/polkit-1/actions/`

## Capabilities

### New Capabilities
- `privileged-operations`: Running system commands via pkexec with PolicyKit policy management, including error handling for cancelled/failed authentication

### Modified Capabilities
- `tab-bar-navigation`: Adding a non-tab action button anchored to the bottom of the tab bar, separate from the navigation tab group

## Impact

- New dependency: `polkit` (runtime) — already present on most Linux desktops
- New file: PolicyKit policy XML (`dev.toomosin.bludio.policy`)
- Modified: `TabBar` component to support a bottom-anchored action button
- Modified: `BludioApp` to handle the restart action (spawn pkexec, surface errors)
- Modified: `PKGBUILD` to install the policy file
- No new Rust crate dependencies — uses `tokio::process::Command` (already available)
