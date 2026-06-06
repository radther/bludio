## ADDED Requirements

### Requirement: Font dropdown is shown in Appearance section
The system SHALL render a "Font" dropdown in the Appearance section of the Settings page. The dropdown SHALL list available bundled fonts: "Noto Sans" and "OpenDyslexic". The selected index SHALL reflect the current `font_family` setting.

#### Scenario: Font dropdown is visible
- **WHEN** the Settings page renders
- **THEN** a "Font" dropdown SHALL appear in the Appearance section
- **THEN** the dropdown SHALL contain exactly two items: "Noto Sans" and "OpenDyslexic"
- **THEN** the currently selected font SHALL be the one matching `font_family`

### Requirement: Selecting a font persists and applies globally
The system SHALL emit an event when the Font dropdown selection changes. `BludioApp` SHALL handle the event by calling `update_settings()` to mutate `font_family`, which persists the setting to disk and triggers a UI re-render.

#### Scenario: User selects OpenDyslexic
- **WHEN** the user selects "OpenDyslexic" from the Font dropdown
- **THEN** `SettingsPage` SHALL emit a font-changed event with value "OpenDyslexic"
- **THEN** `BludioApp` SHALL call `update_settings()` to set `font_family = "OpenDyslexic"`
- **THEN** the settings SHALL be persisted to disk
- **THEN** the UI SHALL re-render with the OpenDyslexic font family

#### Scenario: User selects Noto Sans
- **WHEN** the user selects "Noto Sans" from the Font dropdown
- **THEN** `SettingsPage` SHALL emit a font-changed event with value "Noto Sans"
- **THEN** `BludioApp` SHALL call `update_settings()` to set `font_family = "Noto Sans"`
- **THEN** the UI SHALL re-render with the Noto Sans font family

### Requirement: Noto Sans variable font is bundled
The system SHALL bundle Noto Sans as a variable font file covering all weights and italic styles. The font SHALL be loaded at application startup via `AppContext::add_fonts()`.

#### Scenario: Noto Sans is bundled
- **WHEN** the application is built
- **THEN** `fonts/NotoSans.ttf` and `fonts/NotoSans-Italic.ttf` SHALL exist in the project
- **THEN** the font files SHALL be variable fonts supporting multiple weights

#### Scenario: Noto Sans is loaded at startup
- **WHEN** the application starts
- **THEN** Noto Sans SHALL be loaded into GPUI's font system via `include_bytes!`
- **THEN** text rendered with font family "Noto Sans" SHALL display correctly

### Requirement: OpenDyslexic font is bundled
The system SHALL bundle the OpenDyslexic font for users who prefer it. Because OpenDyslexic does not ship as a variable font, the system SHALL bundle all four style variants (Regular, Bold, Italic, Bold-Italic) to preserve readability-optimized glyph shapes at every weight and style. The fonts SHALL be loaded at application startup alongside Noto Sans.

#### Scenario: OpenDyslexic is bundled
- **WHEN** the application is built
- **THEN** `fonts/OpenDyslexic-Regular.ttf`, `fonts/OpenDyslexic-Bold.ttf`, `fonts/OpenDyslexic-Italic.ttf`, and `fonts/OpenDyslexic-BoldItalic.ttf` SHALL exist in the project
- **THEN** the font files SHALL be included in version control

#### Scenario: OpenDyslexic is loaded at startup
- **WHEN** the application starts
- **THEN** all four OpenDyslexic font files SHALL be loaded into GPUI's font system via `include_bytes!`
- **THEN** text rendered with font family "OpenDyslexic" SHALL display correctly at all weights and styles

### Requirement: Font family defaults to Noto Sans
The system SHALL initialize `font_family` to `"Noto Sans"` when no saved value exists, ensuring the default appearance is unchanged for existing users.

#### Scenario: New user starts app
- **WHEN** the application starts with no settings file
- **THEN** `font_family` SHALL default to `"Noto Sans"`
- **THEN** all text SHALL render in Noto Sans

#### Scenario: Legacy settings file without font_family
- **WHEN** the application starts with a settings file that lacks the `font_family` field
- **THEN** `font_family` SHALL default to `"Noto Sans"`
- **THEN** all text SHALL render in Noto Sans
