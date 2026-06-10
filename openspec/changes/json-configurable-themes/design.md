## Context

Bludio currently ships 14 themes, each as a separate Rust file under `src/ui/theme/`. Every theme file repeats the same pattern: palette constants (often with `#[allow(dead_code)]`), a `hsl!` macro, and a `Theme` constructor that maps those constants to `ThemeColors` fields. The `Theme` struct also carries `font_family` and `text_styles`, even though the font is already a user setting and text styles are unlikely to vary per theme.

Adding a theme requires:
1. Writing a new `.rs` file.
2. Re-exporting it from `mod.rs`.
3. Registering it in `registry.rs` with a manual `make_*()` wrapper and static string ID lists.
4. Adding a display name to `settings_page.rs`.
5. Rebuilding the application.

Users cannot add themes without forking the repository.

## Goals / Non-Goals

**Goals:**
- Define themes in JSON files that can be edited without recompilation.
- Convert all built-in themes to JSON and store them in `assets/themes/`.
- Allow users to drop custom `.json` theme files into `~/.config/bludio/themes/light/` or `~/.config/bludio/themes/dark/` and have them appear in the Settings dropdowns.
- Remove font family and text styles from the `Theme` struct; apply them globally via settings.
- Provide a hex color notation (`#RRGGBB`) in JSON that the loader converts to `Hsla`.
- Maintain backward compatibility: existing users with saved settings must continue to see the same default themes.

**Non-Goals:**
- Runtime theme hot-reloading (not required for MVP; themes are loaded once at startup).
- Theme editing UI within the app.
- Support for color formats other than hex (e.g., HSL, RGB tuples).
- Gradients or non-solid color values.

## Decisions

### 1. JSON structure: separate palette from semantic mapping
**Decision:** Each JSON file contains two dictionaries: `colors` (named palette entries) and `theme` (semantic tokens referencing palette names by string).

**Rationale:** This preserves the nice separation that already exists in the Rust files (palette constants at the top, semantic mapping in the constructor). It makes themes more readable and allows users to tweak a single palette value that affects multiple semantic tokens.

**Example:**
```json
{
  "id": "rose-pine-dawn",
  "display_name": "Rose Pine Dawn",
  "colors": {
    "base": "#faf4ed",
    "foam": "#56949f"
  },
  "theme": {
    "background": "base",
    "bluetooth_accent": "foam"
  }
}
```

### 2. Built-in themes bundled as JSON files, included via `include_str!`
**Decision:** Built-in themes are stored as `.json` files in `assets/themes/light/` and `assets/themes/dark/`. At startup, the loader iterates over a static list of embedded file paths (using `include_str!`) to load them.

**Rationale:** This keeps the themes in a standard data format while still benefiting from Rust's compile-time embedding of assets. No additional asset loading infrastructure is needed, and the binary remains self-contained.

**Alternative considered:** Loading from the filesystem at runtime from a system directory. Rejected because it would make the application dependent on installed data files, complicating distribution and packaging.

### 3. Dynamic registry replaces static `LazyLock<HashMap>`
**Decision:** The registry becomes a `LazyLock<ThemeRegistry>` where `ThemeRegistry` holds a `HashMap<String, Arc<Theme>>` and separate vectors for light and dark IDs. It is populated at first access by scanning both built-in and user directories.

**Rationale:** The static registry of function pointers cannot represent user-defined themes. Moving to a data-based registry with owned strings is necessary for runtime extensibility.

### 4. Font family and text styles removed from `Theme`
**Decision:** `Theme` is reduced to `id`, `appearance`, and `colors`. The font family is applied at the root element from `settings(cx).font_family`. Text styles use a single `Default` implementation.

**Rationale:** The font family is already a user setting (`Settings.font_family`). The `text_styles` have never varied between themes and are unlikely to. Removing them simplifies the `Theme` struct and the JSON format.

**Impact:** The `theme()` accessor no longer needs to mutate `theme.font_family` on every settings change. `GlobalSettings::compute_theme()` returns a pure `Arc<Theme>` from the registry.

### 5. Error handling for malformed user themes
**Decision:** Malformed user theme files are logged as warnings and skipped. The application continues to start with all valid themes (built-in + valid user themes).

**Rationale:** A user theme error should not prevent the application from starting. Built-in themes are guaranteed valid, so the app is always usable.

### 6. Fallback when a saved theme ID is missing
**Decision:** If the settings file references a theme ID that is not found in the registry (e.g., a deleted user theme), `compute_theme()` falls back to `"rose-pine-dawn"` for light mode or `"rose-pine"` for dark mode.

**Rationale:** This prevents the app from crashing or rendering without a theme when a user theme is removed.

## Risks / Trade-offs

- **[Risk]** Startup time increases slightly due to JSON parsing and directory scanning. **Mitigation:** The number of themes is small (< 50). JSON parsing is fast. Directory scanning is a single `read_dir` per mode.
- **[Risk]** User themes with invalid color names in the `theme` mapping reference non-existent palette entries. **Mitigation:** Validate during loading: every value in the `theme` mapping must exist as a key in `colors`. Fail the theme file with a descriptive error if not.
- **[Risk]** Removing `font_family` from `Theme` breaks code that references `theme.font_family`. **Mitigation:** All call sites are in the same codebase (`settings.rs` and `BludioApp` render). A single search-and-replace is sufficient.
- **[Trade-off]** The `id` and `display_name` in JSON are duplicated per file, whereas the old Rust files had only the ID. This is minor (a few bytes per theme) and improves readability.

## Migration Plan

1. **Phase 1 — Prepare JSON themes:** Create `assets/themes/light/` and `assets/themes/dark/`. Write all 14 existing themes as JSON files.
2. **Phase 2 — Implement loader:** Write `src/ui/theme/loader.rs` with JSON parsing, hex-to-Hsla conversion, and validation.
3. **Phase 3 — Update registry:** Rewrite `src/ui/theme/registry.rs` to use the dynamic loader.
4. **Phase 4 — Update types:** Remove `font_family` and `text_styles` from `Theme` in `types.rs`. Update `mod.rs` to remove per-theme re-exports.
5. **Phase 5 — Update settings:** Update `settings.rs` to remove font_family mutation from `compute_theme()`. Update `BludioApp` root render to apply font family from settings.
6. **Phase 6 — Update settings page:** Remove the hardcoded `theme_display_name()` function. Use `theme.display_name()` from the registry.
7. **Phase 7 — Delete old theme files:** Remove all `*_theme.rs` files.
8. **Phase 8 — Test:** Verify all themes render correctly, user themes are discoverable, and fallback behavior works.

## Open Questions

- Should we provide a JSON schema or validation file for user themes? (Not required for MVP, but could be added later.)
- Should we support a theme metadata field (e.g., author, description)? (Not required for MVP.)
