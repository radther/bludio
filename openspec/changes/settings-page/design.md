## Context

The application currently boots with Rose Pine Dawn (light) hardcoded in `main.rs`. Theme switching is only available on the `DevTestPage` via two buttons and is not persisted. The tab bar has five navigation tabs (indices 0–4) with keyboard shortcuts Ctrl+1 through Ctrl+5. There is no mechanism for storing or loading user preferences.

The existing theme system stores the active theme in a GPUI global called `GlobalTheme`. The goal is to fold theme state into a broader settings system, add a Settings page, and make settings accessible from any page in the application.

## Goals / Non-Goals

**Goals:**
- Add a Settings page with theme mode toggle and two theme dropdowns.
- Introduce stable string theme IDs and a fast lookup registry.
- Persist settings to a JSON file and load them at startup.
- Apply saved theme preferences on application launch.
- Update the tab bar to six tabs and keyboard shortcuts to Ctrl+1–6.
- Make settings readable from any page via a free function (`settings(cx)`).
- Make settings writable from any page via a single helper (`update_settings()`).
- Design the system so any future page can listen for settings changes without new infrastructure.

**Non-Goals:**
- Multiple themes per mode (only Rose Pine Dawn for light, Rose Pine dark for dark initially).
- A general-purpose preference UI framework (just the settings we need now).
- Cross-platform settings paths (Linux-only is fine for now).
- Implementing the settings-change listener mechanism now (it should be possible later with no structural changes).

## Decisions

### Replace `GlobalTheme` with `GlobalSettings`
The active theme is derived from `theme_mode` + `light_theme_id`/`dark_theme_id`. Keeping it in a separate `GlobalTheme` global risks the two globals drifting out of sync. Instead, a single `GlobalSettings` global stores the `Settings` struct and a **cached** `Arc<Theme>`.

```rust
struct GlobalSettings {
    settings: Settings,
    active_theme: Arc<Theme>,  // cached, recomputed on change
}
```

`theme(cx)` returns `&cx.global::<GlobalSettings>().active_theme`. `settings(cx)` returns `&cx.global::<GlobalSettings>().settings`. Both are zero-cost reads.

**Alternative considered**: Keep `GlobalTheme` and store `Settings` as a field on `BludioApp`. Rejected: pages outside `BludioApp`'s direct children (or async callbacks) cannot access `BludioApp` state without threading references through every constructor. A global is the correct GPUI pattern for universally accessible state.

### `update_settings()` is the only write path
A single helper function performs the full lifecycle of a settings change:

1. Update the `Settings` struct via a closure.
2. Recompute `active_theme` from the new settings (lookup theme ID in registry).
3. Write the settings JSON to disk.
4. Call `cx.notify()` to trigger re-render.

```rust
pub(crate) fn update_settings<F>(f: F, cx: &mut App)
where
    F: FnOnce(&mut Settings),
{
    cx.update_global::<GlobalSettings, _>(|global, _cx| {
        f(&mut global.settings);
        global.active_theme = compute_theme(&global.settings);
    });
    save_settings(cx);  // fire-and-forget background write
    cx.notify();
}
```

Any page can call this. A Bluetooth page toggle that needs persistence can call `update_settings(|s| s.some_bluetooth_pref = true, cx)` and it will save and re-render automatically.

**Alternative considered**: Let each page read/write the JSON file directly. Rejected: scatters persistence logic, makes it impossible to centralize change notification later.

### Theme IDs as `&'static str`
Each theme variant carries a `&'static str` ID (e.g., `"rose-pine-dawn"`, `"rose-pine"`). This avoids heap allocation, makes comparison cheap, and is easy to serialize. IDs are stored on the `Theme` struct itself.

**Alternative considered**: UUIDs or random strings. Rejected: unnecessary complexity for a small fixed set of themes.

### Static `HashMap<&'static str, fn() -> Arc<Theme>>` registry
A `LazyLock<HashMap<...>>` maps theme IDs to theme constructors. ID-to-theme lookup is O(1) and returns a fresh `Arc<Theme>`. This is created once at first access.

**Alternative considered**: A `Vec<Theme>` with linear scan. Rejected: reordering themes would break saved settings that reference by index.

### Settings file: `~/.config/bludio/settings.json`
Use the XDG config directory. On startup, read the file; if missing or malformed, fall back to defaults (light mode, Rose Pine Dawn for light, Rose Pine for dark). On any settings change, `update_settings` triggers a background write.

**Alternative considered**: A Rust struct serialized with `toml`. Rejected: `serde_json` is already in the dependency tree, JSON is sufficient for a flat key-value file.

### SettingsPage emits events; BludioApp subscribes
The `SettingsPage` entity owns its UI state (dropdown open/closed, etc.) but **does not own the `Settings` struct**. When the user toggles mode or changes a theme selection, the page emits an event. `BludioApp` subscribes and calls `update_settings()`. This keeps the page UI-focused and the app as the orchestrator.

**Alternative considered**: `SettingsPage` calls `update_settings()` directly. Rejected: the page doesn't have direct access to `App` context in all callback contexts; events are the standard GPUI pattern for child→parent communication.

### Dropdown options use `Vec<(&'static str, SharedString)>`
The `Dropdown` entity already accepts a list of strings for options. We will pass display labels. The selected value will be tracked by the settings struct using theme IDs, not indices. The page maps the current ID to an index when rendering.

**Alternative considered**: Extending `Dropdown` to store arbitrary IDs. Rejected: the current dropdown is simple and sufficient; mapping ID↔index is trivial with a small list.

### Settings page reuses `PageHeader` component
The same scrollable header layout used by `AudioPage` and `ConfigurationPage` is reused. A `SettingsPage` entity owns its focus handle and child dropdowns, matching the existing entity ownership pattern.

### Future listener support: `cx.emit()` inside `update_settings()`
When we need pages to react to settings changes beyond just re-rendering (e.g., a page that wants to know when a specific setting changed), we can add a single line inside `update_settings()`:

```rust
// cx.emit(SettingsChanged { /* diff info */ });
```

Because `update_settings()` is the only write path, all settings changes flow through it. Any page can already subscribe to any entity's events via `cx.subscribe_in()`. Adding the emit later requires no structural changes.

**Alternative considered**: A custom observer pattern or channel system. Rejected: GPUI already provides `EventEmitter` + `subscribe`; we just need to emit from the centralized write path.

## Risks / Trade-offs

- **[Risk]** `LazyLock` requires Rust 1.80+; our toolchain is 1.95.0 so this is safe. → No mitigation needed.
- **[Risk]** Writing settings on every change could be frequent if we save on every dropdown selection. → Mitigation: only save when the user makes a definitive change (on dropdown selection event, not on hover/focus). The current dropdown already emits on selection.
- **[Risk]** Theme IDs are hardcoded strings; renaming an ID in code breaks saved user settings. → Mitigation: treat IDs as stable API. If a theme is removed, the loader falls back to the default for that mode.
- **[Risk]** Adding a 6th tab pushes keyboard shortcuts; users with muscle memory for Ctrl+5 (Text Field Test) now use Ctrl+6. → Acceptable: Text Field Test is a dev page and this is a minor UX shift.
- **[Risk]** Replacing `GlobalTheme` with `GlobalSettings` means any code currently calling `theme(cx)` must still work. → Mitigation: `theme(cx)` remains a free function; it just reads from `GlobalSettings` instead of `GlobalTheme`. No call-site changes required.
