## 1. Settings Model & Persistence

- [x] 1.1 Add `disable_animations: bool` and `font_family: String` fields to `Settings` struct in `src/settings.rs`
- [x] 1.2 Update `Settings::default()` to set `disable_animations: false` and `font_family: "Noto Sans".to_string()`
- [x] 1.3 Verify `serde` correctly handles missing fields by testing with an old settings file (or add `#[serde(default)]` if needed)
- [x] 1.4 Update `GlobalSettings::compute_theme` to apply the active font family to the computed theme (or apply it at the root element level)

## 2. Global Animation Disable

- [x] 2.1 Create `GlobalAccessibility` GPUI global in a new `src/ui/accessibility.rs` module (or inline in `src/settings.rs`) to hold `disable_animations`
- [x] 2.2 Initialize `GlobalAccessibility` at app startup with the loaded `disable_animations` value
- [x] 2.3 Update `FadeInAnimationExt::with_fade_in_up` in `src/ui/animation.rs` to check `GlobalAccessibility::disable_animations()`; if true, return an `AnimationElement` with `Duration::ZERO`
- [x] 2.4 Verify animation suppression works by observing that pages render instantly when the flag is true

## 3. Font Bundling

- [x] 3.1 Download all four OpenDyslexic style variants (Regular, Bold, Italic, Bold-Italic) from https://opendyslexic.org and place them in `fonts/OpenDyslexic-{Regular,Bold,Italic,BoldItalic}.ttf`
- [x] 3.2 Add the OpenDyslexic OFL license file to `fonts/` (e.g., `fonts/OpenDyslexic-LICENSE.txt`)
- [x] 3.3 Verify existing `fonts/NotoSans.ttf` and `fonts/NotoSans-Italic.ttf` are variable fonts (check file metadata or name table)
- [x] 3.4 Load all four OpenDyslexic font files in `src/main.rs` via `include_bytes!` alongside Noto Sans

## 4. High Contrast Themes

- [x] 4.1 Create `src/ui/theme/high_contrast_light_theme.rs` with a `high_contrast_light()` constructor using a high-contrast light palette
- [x] 4.2 Create `src/ui/theme/high_contrast_dark_theme.rs` with a `high_contrast_dark()` constructor using a high-contrast dark palette
- [x] 4.3 Export the new constructors from `src/ui/theme/mod.rs`
- [x] 4.4 Register `"high-contrast-light"` and `"high-contrast-dark"` in `src/ui/theme/registry.rs` with their constructors and mode lists

## 5. Settings Page UI Updates

- [x] 5.1 Add `DisableAnimationsChanged(bool)` and `FontChanged(String)` variants to `SettingsEvent` in `src/ui/settings_page.rs`
- [x] 5.2 Create a "Disable Animations" toggle control (can be a simple button or checkbox-like element) in the Settings page, placed under an "Accessibility" `heading`-styled header
- [x] 5.3 Add an "Appearance" `heading`-styled section header above the existing theme controls
- [x] 5.4 Create a Font dropdown in the Appearance section with items ["Noto Sans", "OpenDyslexic"], initialized from the current `font_family` setting
- [x] 5.5 Wire up the Font dropdown to emit `FontChanged` events via `cx.subscribe_in()`
- [x] 5.6 Wire up the Disable Animations toggle to emit `DisableAnimationsChanged` events
- [x] 5.7 Update `theme_display_name()` to include "High Contrast Light" and "High Contrast Dark"
- [x] 5.8 Update `sync_dropdowns()` to also sync the Font dropdown if one exists (or add a `sync_font_dropdown` method)

## 6. App-Level Event Handling

- [x] 6.1 Update `BludioApp`'s `SettingsPage` subscription in `src/app.rs` to handle `DisableAnimationsChanged(bool)` by calling `update_settings(|s| s.disable_animations = value, cx)`
- [x] 6.2 Update `BludioApp`'s `SettingsPage` subscription to handle `FontChanged(family)` by calling `update_settings(|s| s.font_family = family, cx)`
- [x] 6.3 Ensure `BludioApp::render` applies the active `font_family` from settings to the root element (check `src/app.rs` line ~1118)

## 7. Verification & Cleanup

- [x] 7.1 Run `cargo build` and fix any compilation errors
- [x] 7.2 Run `cargo clippy` and address warnings
- [x] 7.3 Run `cargo fmt` to ensure consistent formatting
- [x] 7.4 Test: start app with no settings file, verify defaults (Noto Sans, animations on, Rose Pine Dawn)
- [x] 7.5 Test: toggle Disable Animations, verify setting persists across restarts and animations are suppressed
- [x] 7.6 Test: select OpenDyslexic font, verify setting persists and text renders in OpenDyslexic
- [x] 7.7 Test: select High Contrast Light/Dark themes, verify colors are high-contrast and persist
- [x] 7.8 Test: start app with an old settings file missing new fields, verify defaults are applied without error
