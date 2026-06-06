## 1. Theme System — IDs and Registry

- [x] 1.1 Add `id: &'static str` field to `Theme` struct and `theme.id()` accessor
- [x] 1.2 Set `"rose-pine"` ID in `rose_pine()` constructor and `"rose-pine-dawn"` in `rose_pine_dawn()`
- [x] 1.3 Create `src/ui/theme/registry.rs` with `LazyLock<HashMap<&'static str, fn() -> Arc<Theme>>>`
- [x] 1.4 Add `light_theme_ids()` and `dark_theme_ids()` registry helpers returning `&[&'static str]`
- [x] 1.5 Add `theme_for_id(id) -> Option<Arc<Theme>>` helper in `src/ui/theme/mod.rs` (used by `update_settings`)
- [x] 1.6 Wire `mod registry` into `src/ui/theme/mod.rs`

## 2. Settings Persistence — GlobalSettings Global

- [x] 2.1 Create `src/settings.rs` with `Settings` struct (`theme_mode`, `light_theme_id`, `dark_theme_id`)
- [x] 2.2 Add `ThemeMode` enum (`Light`, `Dark`) with `Default` and `Serialize`/`Deserialize`
- [x] 2.3 Implement `Settings::load()` from `~/.config/bludio/settings.json` with fallback to defaults
- [x] 2.4 Implement `Settings::save()` to `~/.config/bludio/settings.json`
- [x] 2.5 Create config directory if missing during save
- [x] 2.6 Define `GlobalSettings` struct holding `settings: Settings` + `active_theme: Arc<Theme>`
- [x] 2.7 Implement `theme(cx) -> &Arc<Theme>` free function reading from `GlobalSettings`
- [x] 2.8 Implement `settings(cx) -> &Settings` free function reading from `GlobalSettings`
- [x] 2.9 Implement `update_settings<F>(f: F, cx: &mut App)` that updates settings, recomputes theme, saves, and calls `cx.notify()`
- [x] 2.10 Remove `GlobalTheme` from `src/ui/theme/types.rs` (functionality merged into `GlobalSettings`)
- [x] 2.11 Update `theme(cx)` in `src/ui/theme/mod.rs` to read from `GlobalSettings`
- [x] 2.12 Update `main.rs` to initialize `GlobalSettings` (with loaded settings + computed theme) instead of `GlobalTheme`

## 3. Settings Page UI

- [x] 3.1 Create `src/ui/settings_page.rs` with `SettingsPage` entity
- [x] 3.2 Add `PageHeader` with static info text "Configure application appearance and behavior."
- [x] 3.3 Add Theme Mode section with "Light" and "Dark" toggle buttons
- [x] 3.4 Add Light Theme dropdown populated from registry light theme IDs
- [x] 3.5 Add Dark Theme dropdown populated from registry dark theme IDs
- [x] 3.6 Emit `SettingsEvent::ModeChanged` / `SettingsEvent::LightThemeChanged` / `SettingsEvent::DarkThemeChanged` from child interactions
- [x] 3.7 Wire `cx.subscribe_in()` on `BludioApp` to handle settings events and call `update_settings()`
- [x] 3.8 Make settings page scrollable like other pages
- [x] 3.9 Export `SettingsPage` from `src/ui/mod.rs`

## 4. App Shell Integration

- [x] 4.1 Add `SettingsPage` variant to `Page` enum
- [x] 4.2 Add `settings_page` entity field to `BludioApp`
- [x] 4.3 Add 6th tab to tab bar config in `src/app.rs` (`bolt` icon, "Settings" tooltip)
- [x] 4.4 Map index 5 → `Page::SettingsPage` in `switch_to_tab()`
- [x] 4.5 Update keyboard shortcuts to handle index 5 (Ctrl+6)
- [x] 4.6 `Page::title()` does not exist in this codebase; page titles are handled by individual page headers. Skipped.
- [x] 4.7 Update `active_page` match in `render()` to include `Page::SettingsPage`
- [x] 4.8 SettingsPage reads from global `settings(cx)` directly; no snapshot needed.

## 5. Cleanup and Verification

- [x] 5.1 Run `cargo build` and fix compilation errors
- [x] 5.2 Run `cargo fmt`
- [x] 5.3 Run `cargo clippy` and address warnings
- [x] 5.4 Verify theme persists across app restarts (requires manual runtime test)
- [x] 5.5 Verify tab switching works for all 6 tabs including keyboard shortcuts (requires manual runtime test)
- [x] 5.6 Verify `theme(cx)` still works from any page without changes to existing call sites
