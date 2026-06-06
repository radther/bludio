## Why

The application currently has no persistent user preferences. Theme switching exists only on the DevTestPage and resets on restart. Users need a central place to configure application behavior, starting with theme preferences that survive relaunches.

## What Changes

- Add a **Settings** page accessible from the tab bar, using the `bolt` icon.
- Add a **Theme Mode** setting with Light / Dark toggle buttons.
- Add a **Light Theme** dropdown and a **Dark Theme** dropdown, each populated from a theme registry.
- Introduce **stable theme IDs** (string identifiers) so themes are referenced by ID rather than index, preventing breakage if themes are reordered.
- Add a **theme registry** (a static map/dictionary) for fast ID-based lookup.
- Add **persistent settings storage** (JSON file on disk) that loads at startup and saves on change.
- Replace the existing `GlobalTheme` GPUI global with a unified **`GlobalSettings`** global that holds both the `Settings` struct and the active `Arc<Theme>`.
- Expose `settings(cx)` and `theme(cx)` free functions so any page can read current settings or the active theme directly, without threading data through `BludioApp`.
- Provide an **`update_settings()`** helper that any page can call to change a setting, recompute the active theme, persist to disk, and trigger a UI re-render.
- Update the tab bar from 5 to 6 tabs and keyboard shortcuts from Ctrl+1–5 to Ctrl+1–6.
- Settings page uses the same scrollable header layout as other pages.

## Capabilities

### New Capabilities
- `settings-page`: A scrollable settings page with theme mode toggle and per-mode theme dropdowns.
- `settings-persistence`: Loading and saving user preferences (JSON file) on application startup and when settings change.
- `theme-registry`: A static, lookup-optimized map of theme IDs to `Arc<Theme>` constructors, replacing index-based selection.

### Modified Capabilities
- `theme-system`: Add theme ID strings to each theme variant, expose a registry for ID-based lookup, and replace `GlobalTheme` with `GlobalSettings` (which caches the active theme derived from settings).
- `tab-bar-navigation`: Expand from 5 to 6 navigation tabs, add the Settings tab (`bolt` icon), and extend keyboard shortcuts to Ctrl+1 through Ctrl+6.
- `app-shell`: Include `SettingsPage` in the active page enumeration, initialize `GlobalSettings` at startup, and render it in the content area.

## Impact

- New files: `src/ui/settings_page.rs`, `src/settings.rs`, `src/ui/theme/registry.rs`.
- Modified files: `src/app.rs`, `src/ui/tab_bar.rs`, `src/ui/theme/mod.rs`, `src/ui/theme/types.rs`, `src/ui/theme/rose_pine_theme.rs`, `src/ui/theme/rose_pine_dawn_theme.rs`, `src/main.rs`.
- Removed/replaced: `GlobalTheme` global in `src/ui/theme/types.rs` becomes part of `GlobalSettings`.
- Dependencies: No new crates; use `std::fs`/`serde_json` (already in dependency tree via GPUI).
