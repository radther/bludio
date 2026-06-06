## Why

Bludio currently bundles a single font and two Rose Pine theme variants, with no way for users to adjust motion or typography to their needs. Adding accessibility options (animation control, font selection, and high-contrast themes) makes the application usable for a wider audience, including those with vestibular disorders, dyslexia, or visual impairments.

## What Changes

1. **Disable Animations toggle** — Add an Accessibility section to the Settings page with a "Disable Animations" toggle. When enabled, all GPUI view animations are globally suppressed at the animation layer rather than checked per-view.
2. **Appearance section header & font selection** — Rename the existing theme section to "Appearance" and add a Font dropdown. The dropdown offers "Noto Sans" (current bundled variable font) and "OpenDyslexic" (newly bundled). Verify that Noto Sans is correctly bundled as a variable font.
3. **High-contrast themes** — Add "High Contrast Light" and "High Contrast Dark" theme variants to the theme registry and make them selectable via the existing Light Theme / Dark Theme dropdowns.
4. **Settings persistence** — New fields (`disable_animations`, `font_family`) are persisted alongside existing settings and restored on startup.

## Capabilities

### New Capabilities
- `accessibility-options`: Global animation disable toggle and related accessibility settings.
- `font-selection`: Runtime font switching with bundled Noto Sans and OpenDyslexic fonts.
- `high-contrast-themes`: High Contrast Light and High Contrast Dark theme variants.

### Modified Capabilities
- `settings-page`: Add "Accessibility" and "Appearance" section headers; add Disable Animations toggle and Font dropdown; existing theme dropdowns remain under the Appearance header.
- `theme-system`: Add high-contrast theme constructors; add font family field to theme (or global settings) so the active font can be switched at runtime.
- `theme-registry`: Register `high-contrast-light` and `high-contrast-dark` theme IDs with mode categorization.
- `settings-persistence`: New settings fields (`disable_animations`, `font_family`) are loaded and saved alongside existing fields.

## Impact

- **UI**: Settings page layout gains section headers; new controls added.
- **Theme module**: New theme files (`high_contrast_light.rs`, `high_contrast_dark.rs`); theme struct may gain a font family field.
- **Animation layer**: All animated views (page transitions, etc.) will read a global flag to decide whether to run animations.
- **Assets**: `fonts/` directory gains `OpenDyslexic-Regular.otf` (or equivalent); existing `NotoSans.ttf`/`NotoSans-Italic.ttf` verified.
- **Settings persistence**: `Settings` struct gains `disable_animations: bool` and `font_family: String`.
