## Why

Bludio currently hardcodes colors in every UI component — `hsla(0.0, 0.0, 0.14, 1.0)`, `hsla(210.0/360.0, 0.7, 0.55, 1.0)`, etc. — and has no font configuration. As the UI grows, inconsistent values accumulate and there is no way to switch between light and dark appearances. A theme system centralizes design tokens, eliminates color duplication, and provides a foundation for future theming (multiple themes, user customization, font scaling).

## What Changes

- **New `Theme` struct** with semantic color fields (background, surface, text, accent, status colors) and font configuration (family, weights, size scale)
- **New `TextStyle` enum** (`Body`, `Heading`, `Caption`, etc.) with a convenience method on `Styled` that sets font family, weight, and size in one call — following Zed's pattern where `window.text_style()` provides a resolved `TextStyle` struct, but adapted as a lightweight builder method for Bludio
- **Light and dark theme presets** (dark is current look, light is a warm light variant) switchable from `DevTestPage`
- **Global theme stored in a GPUI global** (similar to Zed's `GlobalTheme`), accessible via `cx.theme()` / `window.theme()` convenience methods
- **Convert all existing hardcoded colors and fonts** across the app to use theme tokens — every `hsla(...)` call that maps to a semantic token is replaced; interactive components (dropdown, text field, slider) that accept color overrides retain the override pattern but default to theme colors
- **Font**: use "Noto Sans" as the default font family (matching the rss-reader example)

## Capabilities

### New Capabilities
- `theme-system`: Centralized theme with color tokens, font configuration, text styles, and light/dark switching

### Modified Capabilities
<!-- No existing specs have requirement-level changes — only implementation details change -->
_None_

## Impact

- **New files**: `src/ui/theme.rs` (Theme struct, colors, text styles), possibly `src/ui/text_style.rs` (TextStyle enum + extension trait) or integrated into `theme.rs`
- **Modified files**: Every UI module that uses hardcoded colors — `app.rs`, `tab_bar.rs`, `dev_test_page.rs`, `audio/audio_page.rs`, `audio/device_row.rs`, `audio/card_row.rs`, `audio/configuration_page.rs`, `bluetooth/bluetooth_page.rs`, `bluetooth/device_row.rs`, `components/dropdown.rs`, `components/slider.rs`, `components/text_field.rs`
- **New dependency**: `Noto Sans` font (bundled or system-provided)
- **No API contract changes** — same UI, same behavior, just themed
