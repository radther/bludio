## Why

The current specs were written incrementally as features were built, but the codebase has evolved significantly since the initial specs were authored. Several specs are now out of date: they reference incorrect file paths, omit components and pages that have been added, describe theme defaults that have changed, and miss architectural patterns (Entity/EventEmitter, command channels) that the codebase now consistently uses. Reconciling specs with reality ensures they remain useful as living documentation and accurate context for future changes.

## What Changes

This is a **spec-only change** — no code will be modified. The following updates are needed:

- **app-shell**: Add missing Configuration and DevTest tabs; update app identity string from `"com.bludio.app"` to `"dev.toomosin.bludio"`; document keyboard shortcuts (Ctrl+1–5); reference the actual page entities rather than describing inline rendering.
- **audio-device-management**: Update to reflect that output device rows include a profile dropdown (not just the Configuration page); document that volume is displayed via a Slider entity + text field, not a "volume bar" and "percentage" label; add reference to `status_strip` component for default indicator.
- **audio-volume-control**: Align terminology with actual component names (`Slider`/`SliderState` entity, `TextField` entity); document the EventEmitter-based communication pattern; note that `SliderEvent::Release` exists but is currently unused by `AudioDeviceRow`.
- **bluetooth-device-management**: Document the `BluetoothPageCommand` channel pattern; add reference to `status_strip` for visual connection indicators; note that `PairAndTrust` is dispatched as a multi-step chain in `app.rs` rather than a single D-Bus call.
- **card-configuration-page**: Minor alignment — confirm `CardRow` entity pattern and `Dropdown` ownership match the spec.
- **pairing-status-indicator**: Confirm status labels match `PairingStatus::Display` impl — currently accurate, no changes expected.
- **slider-component**: Confirm architecture matches actual implementation (`SliderState` Entity + `Slider` RenderOnce element) — currently accurate, minor clarifications on `canvas` bounds capture.
- **tab-bar-navigation**: Update from 3 tabs to 5 (add Configuration and Text Field Test); document `TabBar` entity and `TabBarEvent` pattern; update icon names to match actual SVG files (`radar`, `speaker`, `mic`, `form`, `test-tube-diagonal`).
- **theme-system**: Update file path from `src/ui/theme.rs` to `src/ui/theme/` module; change default theme from dark to light (Rose Pine Dawn); document Rose Pine theme variants instead of generic "dark/light presets"; add `body2` text style to `TextStyleSet` documentation; note the three accent colors (`bluetooth_accent`, `audio_accent`, `dev_accent`) instead of a single `accent`.

## Capabilities

### New Capabilities

None — this change only updates existing specs.

### Modified Capabilities

- `app-shell`: Update tab list, app identity, keyboard shortcuts, page entity references.
- `audio-device-management`: Update device row layout (profile dropdown, slider+text field volume, status_strip).
- `audio-volume-control`: Align terminology with actual component names and EventEmitter pattern.
- `bluetooth-device-management`: Document command channel pattern, status_strip, PairAndTrust dispatch model.
- `card-configuration-page`: Minor alignment with CardRow entity pattern (likely no spec changes needed).
- `pairing-status-indicator`: Confirm accuracy (likely no changes needed).
- `slider-component`: Minor clarifications on canvas bounds capture and RenderOnce architecture.
- `tab-bar-navigation`: Update tab count, icon names, TabBar entity pattern.
- `theme-system`: Update file structure, default theme, theme variants, accent colors, text styles.

## Impact

- **Specs only** — no code changes, no dependency changes, no API changes.
- All modified spec files live under `openspec/specs/<capability>/spec.md`.
- This change produces no `design.md` or `tasks.md` artifacts since the work is purely documentation updates.
