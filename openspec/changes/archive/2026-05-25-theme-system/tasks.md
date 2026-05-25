## 1. Theme module scaffolding

- [x] 1.1 Create `src/ui/theme.rs` with module structure: `Appearance` enum, `ThemeColors` struct, `FontConfig` struct, `TextStyle` enum, `TextStyleSet`, `Theme` struct, `GlobalTheme` wrapper, and the `theme(cx)` / `window_theme(cx)` accessor functions
- [x] 1.2 Add `pub mod theme;` to `src/ui/mod.rs`

## 2. Font bundling

- [x] 2.1 Copy `NotoSans.ttf` and `NotoSans-Italic.ttf` variable fonts from `~/.local/share/fonts/noto-vf/` into a `fonts/` directory at project root
- [x] 2.2 In `main.rs`, embed fonts with `include_bytes!` and load via `cx.text_system().add_fonts(vec![...])` before opening the window

## 3. Theme initialization as GPUI global

- [x] 3.1 Implement `Theme::dark()` with all color fields matching current hardcoded values, and `Theme::light()` with warm light variant
- [x] 3.2 Implement `FontConfig` with `font_family: "Noto Sans"`, weight constants, and size scale
- [x] 3.3 Implement `TextStyleSet` mapping each `TextStyle` variant to its size + weight
- [x] 3.4 In `main.rs`, call `cx.set_global(GlobalTheme::new(Theme::dark()))` during app init
- [x] 3.5 Implement `pub(crate) fn theme(cx: &App) -> &Arc<Theme>` and `pub(crate) fn set_theme(theme: Theme, cx: &mut App)` free functions

## 4. TextStyle extension trait

- [x] 4.1 Add `TextStyleExt` trait to `src/ui/ext.rs` (or a new file) with a `.text_style(TextStyle)` method on any `Styled + Sized` type
- [x] 4.2 The method reads the active theme from `Window::theme()` and sets `.font_family()`, `.font_weight()`, and `.text_size()` accordingly

## 5. Component default colors

- [x] 5.1 Update `Dropdown::new()` to use theme defaults: `accent` → `theme.colors.accent`, `bg` → `theme.colors.element_background`, `hover_bg` → `theme.colors.element_hover`, `menu_bg` → `theme.colors.surface`, `menu_border` → `theme.colors.border`
- [x] 5.2 Update `SliderBar::new()` to use theme defaults: `track_color` → `theme.colors.element_background`, `fill_color` → `theme.colors.accent`, `border_color` → `theme.colors.accent`
- [x] 5.3 Update `TextField` placeholder color and cursor color to use theme values (currently hardcoded in the canvas closure)
- [x] 5.4 Update `action_btn` functions to use theme color defaults when not explicitly overridden (accept `Option<Hsla>` or continue as-is since callers will pass theme values)

## 6. App shell conversion

- [x] 6.1 In `app.rs` `BludioApp::render`, replace all `hsla(...)` references with theme colors: `bg`, `text_color`, tab bar border override (or pass theme to tab_bar)
- [x] 6.2 In `tab_bar.rs`, accept theme as parameter and replace hardcoded `bg`, `hover_color`, `highlight`, `border_color`, `icon_color` with theme values

## 7. Page conversions

- [x] 7.1 In `bluetooth/bluetooth_page.rs`, replace `surface`, `text_secondary`, `accent`, `accent_hover`, `danger`, `danger_hover`, `border_color`, `error_bg`, `error_text` with theme colors; replace `.font_weight(FontWeight::BOLD)` with `.text_style(TextStyle::Heading)`
- [x] 7.2 In `bluetooth/device_row.rs`, replace `text_secondary`, `accent`, `accent_hover`, `danger`, `danger_hover`, `amber`, `success`, `border_color`, hover bg with theme colors; replace font_weight calls with text_style
- [x] 7.3 In `audio/audio_page.rs`, replace `surface`, `text_secondary`, `border_color`, `error` colors with theme colors; replace `.font_weight(FontWeight::BOLD)` with `.text_style(TextStyle::Heading)`
- [x] 7.4 In `audio/device_row.rs`, replace all hardcoded colors (accent, accent_hover, vol_color, mute colors, action_btn colors, border_color, hover bg, dropdown colors) with theme references; replace font weights with text_style
- [x] 7.5 In `audio/configuration_page.rs`, replace `surface`, `text_secondary`, `border_color` with theme colors; replace font weight with text_style
- [x] 7.6 In `audio/card_row.rs`, replace `border_color`, hover bg with theme colors; replace font weight with text_style; update dropdown construction to use theme defaults (remove explicit color overrides)

## 8. DevTestPage conversion + theme switcher

- [x] 8.1 Replace all hardcoded colors in `dev_test_page.rs` with theme references
- [x] 8.2 Replace font_weight calls with `.text_style(TextStyle::...)` where appropriate
- [x] 8.3 Add "Dark" and "Light" buttons to the DevTestPage header that call `set_theme()`

## 9. Verification

- [x] 9.1 Run `cargo build` — ensure compilation succeeds
- [x] 9.2 Run `cargo clippy` — fix any warnings
- [x] 9.3 Run `cargo fmt` — ensure formatting
- [x] 9.4 Visual check: run `cargo run`, verify all pages look correct (dark theme matches current look), toggle light theme on DevTestPage and verify all pages update
