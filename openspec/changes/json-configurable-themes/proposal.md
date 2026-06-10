## Why

Currently every theme is a hand-written Rust file containing a full `Theme` struct construction with palette constants, font family, text styles, and semantic color mapping. Adding a new theme means creating a new `.rs` file and rebuilding the application. Users cannot create or share themes without modifying the source code. This proposal makes themes data-driven via JSON files that can be edited without recompilation, and allows users to add custom themes in `~/.config/bludio/themes/`.

## What Changes

- **Move font family and text styles out of `Theme`**: The font family is already a user setting (via `Settings.font_family`). Text styles are not theme-specific and will use a single global default. **BREAKING**: `Theme` struct no longer carries `font_family` or `text_styles` fields.
- **Introduce JSON theme format**: Each theme is defined in a `.json` file with: an `id` (kebab-case, used for settings keys), a `display_name` (human-readable), a `colors` palette (named hex values), and a `theme` mapping (semantic token names → color name references).
- **Support hex color notation**: JSON themes use `#RRGGBB` hex colors. The loader parses these into GPUI `Hsla` at runtime.
- **Built-in themes as JSON files**: All existing themes (Rose Pine, Rose Pine Dawn, High Contrast, etc.) are converted to JSON and stored in `assets/themes/light/` and `assets/themes/dark/`.
- **User theme directory**: The loader also scans `~/.config/bludio/themes/light/` and `~/.config/bludio/themes/dark/` for additional `.json` files at startup. User themes are merged into the registry and appear in the Settings dropdowns.
- **Dynamic theme registry**: The static `HashMap` registry is replaced by a runtime registry populated by scanning built-in and user theme directories. Display names come from the JSON `display_name` field, eliminating the hardcoded `theme_display_name()` function.
- **Theme struct simplification**: `Theme` keeps only `id`, `appearance`, and `colors`. Font family and text styles are applied globally via settings and styling helpers.
- **Remove individual theme `.rs` files**: All `*_theme.rs` files are deleted. The loader becomes the single source of truth.

## Capabilities

### New Capabilities
- `json-theme-loader`: Runtime scanning, parsing, and validation of JSON theme files from built-in and user directories. Produces a `Theme` from JSON data.
- `theme-json-format`: The schema for a valid theme JSON file: `id`, `display_name`, `colors`, and `theme` keys with hex color support and semantic mapping.
- `user-theme-directory`: Discovery and loading of user-provided themes from `~/.config/bludio/themes/` at startup.

### Modified Capabilities
- `theme-system`: The `Theme` struct is simplified. Font family and text styles are removed from the theme. The `theme()` accessor and `GlobalSettings::compute_theme()` are updated to work with the dynamic registry. The `Appearance` and `ThemeColors` types remain but are populated from JSON.
- `theme-registry`: The static `LazyLock<HashMap>` registry is replaced by a runtime-populated registry that merges built-in JSON themes and user themes. `theme_for_id()`, `light_theme_ids()`, and `dark_theme_ids()` become dynamic lookups.
- `high-contrast-themes`: The High Contrast Light and Dark themes are converted to JSON files in `assets/themes/light/` and `assets/themes/dark/` instead of Rust constructors. The spec requirements for color values remain unchanged.
- `settings-persistence`: The settings system must work with the new dynamic registry. If a settings file references a theme ID that no longer exists (e.g., a removed user theme), it falls back to the default.

## Impact

- All `src/ui/theme/*_theme.rs` files are deleted (14 files).
- `src/ui/theme/types.rs` is updated to remove `font_family` and `text_styles` from `Theme`.
- `src/ui/theme/registry.rs` is rewritten to use a dynamic registry.
- `src/ui/theme/mod.rs` is simplified to remove per-theme re-exports.
- `src/backend/settings.rs` is updated to work with the dynamic registry and apply font family globally.
- `src/ui/pages/settings/settings_page.rs` is updated to remove the hardcoded `theme_display_name()` function and use JSON-provided names.
- New assets directory: `assets/themes/light/` and `assets/themes/dark/` with JSON theme files.
- `Cargo.toml` may need a dependency for JSON parsing (already available via `serde_json`).
