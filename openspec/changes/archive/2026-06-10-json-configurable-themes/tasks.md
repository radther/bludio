## 1. JSON Theme Assets

- [x] 1.1 Create `assets/themes/light/` directory and add `rose-pine-dawn.json` with Rose Pine Dawn palette and semantic mapping
- [x] 1.2 Create `assets/themes/dark/` directory and add `rose-pine.json` with Rose Pine dark palette and semantic mapping
- [x] 1.3 Add `high-contrast-light.json` to `assets/themes/light/` with high-contrast light palette
- [x] 1.4 Add `high-contrast-dark.json` to `assets/themes/dark/` with high-contrast dark palette
- [x] 1.5 Add `extreme-high-contrast.json` to `assets/themes/light/` with extreme high-contrast light palette
- [x] 1.6 Add `extreme-high-contrast-dark.json` to `assets/themes/dark/` with extreme high-contrast dark palette
- [x] 1.7 Add `theme2.json` through `theme5.json` to `assets/themes/light/` with template palettes
- [x] 1.8 Add `theme2dark.json` through `theme5dark.json` to `assets/themes/dark/` with template palettes
- [x] 1.9 Verify all JSON files are valid and parseable with a quick test script

## 2. Theme Loader Implementation

- [x] 2.1 Create `src/ui/theme/loader.rs` with a `ThemeLoader` struct
- [x] 2.2 Implement hex-to-Hsla parsing function (`#RRGGBB` → `Hsla` with alpha 1.0)
- [x] 2.3 Implement JSON deserialization: `id`, `display_name`, `colors`, `theme` fields
- [x] 2.4 Implement semantic mapping validation: ensure every `theme` value exists in `colors`
- [x] 2.5 Implement required token validation: ensure all `ThemeColors` fields are present in the `theme` mapping
- [x] 2.6 Implement `load_theme_from_json(content: &str, appearance: Appearance) -> Result<Theme, String>`
- [x] 2.7 Add `loader` module to `src/ui/theme/mod.rs`
- [x] 2.8 Write unit tests for loader: valid theme, invalid hex, missing palette reference, missing semantic token

## 3. Dynamic Theme Registry

- [x] 3.1 Rewrite `src/ui/theme/registry.rs` to use a `ThemeRegistry` struct with `HashMap<String, Arc<Theme>>`
- [x] 3.2 Implement built-in theme loading: iterate over embedded JSON paths using `include_str!` and load each via the loader
- [x] 3.3 Implement user theme directory scanning: read `~/.config/bludio/themes/light/` and `~/.config/bludio/themes/dark/`
- [x] 3.4 Implement user theme loading: parse each `.json` file and add to registry, logging warnings for invalid files
- [x] 3.5 Implement `theme_for_id(id: &str) -> Option<Arc<Theme>>` on the dynamic registry
- [x] 3.6 Implement `light_theme_ids() -> Vec<&str>` and `dark_theme_ids() -> Vec<&str>` returning IDs from the registry
- [x] 3.7 Implement `theme_display_name(id: &str) -> Option<&str>` on the registry
- [x] 3.8 Ensure user themes override built-in themes with the same ID (last-write-wins)

## 4. Theme Type Simplification

- [x] 4.1 Remove `font_family` field from `Theme` struct in `src/ui/theme/types.rs`
- [x] 4.2 Remove `text_styles` field from `Theme` struct in `src/ui/theme/types.rs`
- [x] 4.3 Update `Theme` struct to store `display_name: String` (loaded from JSON)
- [x] 4.4 Update `TextStyleSet` to remain a standalone type but not be part of `Theme`
- [x] 4.5 Update any code referencing `theme.font_family` to use `settings(cx).font_family` instead
- [x] 4.6 Update any code referencing `theme.text_styles` to use `TextStyleSet::default()` or a shared global
- [x] 4.7 Update `BludioApp` root render to apply `.font_family(settings(cx).font_family.clone())`

## 5. Settings Integration

- [x] 5.1 Update `GlobalSettings::compute_theme()` to use the dynamic registry instead of static `theme_for_id()`
- [x] 5.2 Implement fallback logic: if saved theme ID is missing from registry, use `"rose-pine-dawn"` (light) or `"rose-pine"` (dark)
- [x] 5.3 Remove the `font_family` mutation from `compute_theme()` since `Theme` no longer carries it
- [x] 5.4 Update `Settings` struct serialization to remain backward compatible with existing settings files
- [x] 5.5 Verify that `update_settings()` still persists and triggers re-render correctly

## 6. Settings Page Updates

- [x] 6.1 Remove hardcoded `theme_display_name()` function from `settings_page.rs`
- [x] 6.2 Update `SettingsPage::new()` to query display names from the theme registry
- [x] 6.3 Update `SettingsPage::sync_dropdowns()` to work with dynamic registry
- [x] 6.4 Verify dropdowns correctly list all built-in and user themes
- [x] 6.5 Verify theme selection events still emit the correct theme IDs

## 7. Cleanup and Removal

- [x] 7.1 Delete `src/ui/theme/rose_pine_theme.rs`
- [x] 7.2 Delete `src/ui/theme/rose_pine_dawn_theme.rs`
- [x] 7.3 Delete `src/ui/theme/high_contrast_light_theme.rs`
- [x] 7.4 Delete `src/ui/theme/high_contrast_dark_theme.rs`
- [x] 7.5 Delete `src/ui/theme/extreme_high_contrast_theme.rs`
- [x] 7.6 Delete `src/ui/theme/extreme_high_contrast_dark_theme.rs`
- [x] 7.7 Delete `src/ui/theme/theme2_theme.rs` through `theme5_theme.rs`
- [x] 7.8 Delete `src/ui/theme/theme2dark_theme.rs` through `theme5dark_theme.rs`
- [x] 7.9 Update `src/ui/theme/mod.rs` to remove all per-theme re-exports and only re-export core types, loader, and registry
- [x] 7.10 Run `cargo build` to verify compilation
- [x] 7.11 Run `cargo test` to verify all tests pass
- [x] 7.12 Run `cargo clippy` to check for warnings
- [x] 7.13 Run `cargo fmt` to ensure formatting

## 8. Integration Testing

- [x] 8.1 Start the application and verify all built-in themes render correctly in both light and dark modes
- [x] 8.2 Create a custom user theme in `~/.config/bludio/themes/light/` and verify it appears in the dropdown
- [x] 8.3 Select the custom user theme and verify it applies correctly
- [x] 8.4 Delete the custom theme and verify the app falls back to the default theme
- [x] 8.5 Verify font family switching still works after removing it from `Theme`
- [x] 8.6 Verify settings persistence still works after all changes
- [x] 8.7 Test with a malformed user theme file and verify it is skipped with a warning
