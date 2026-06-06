## ADDED Requirements

### Requirement: High Contrast Light theme constructor
The system SHALL provide a `high_contrast_light()` constructor that returns a complete `Theme` with a high-contrast light palette. The theme SHALL have the ID `"high-contrast-light"` and appearance `Appearance::Light`.

#### Scenario: Construct High Contrast Light
- **WHEN** `high_contrast_light()` is called
- **THEN** it SHALL return a `Theme` with ID `"high-contrast-light"`
- **THEN** the appearance SHALL be `Light`
- **THEN** all color tokens SHALL have high-contrast values
- **THEN** the font family SHALL be `"Noto Sans"` (or match the active user preference)

### Requirement: High Contrast Dark theme constructor
The system SHALL provide a `high_contrast_dark()` constructor that returns a complete `Theme` with a high-contrast dark palette. The theme SHALL have the ID `"high-contrast-dark"` and appearance `Appearance::Dark`.

#### Scenario: Construct High Contrast Dark
- **WHEN** `high_contrast_dark()` is called
- **THEN** it SHALL return a `Theme` with ID `"high-contrast-dark"`
- **THEN** the appearance SHALL be `Dark`
- **THEN** all color tokens SHALL have high-contrast values
- **THEN** the font family SHALL be `"Noto Sans"` (or match the active user preference)

### Requirement: Active font family is applied at the root
The system SHALL apply the active font family from `Settings` to the root UI element so that all child elements inherit it. The font family SHALL be accessible via `settings(cx).font_family`.

#### Scenario: Root element applies font family
- **WHEN** `BludioApp` renders its root element
- **THEN** it SHALL call `.font_family(settings(cx).font_family.clone())` on the root container
- **THEN** all text descendants SHALL inherit the selected font family

## MODIFIED Requirements

### Requirement: Theme provides font configuration
The system SHALL store font family and text style roles in the theme. The font family SHALL be "Noto Sans", provided as a bundled variable font (`fonts/NotoSans.ttf` and `fonts/NotoSans-Italic.ttf`) covering all weights and italic. The active font family at runtime SHALL come from `Settings`, overriding the theme's default if the user has selected a different bundled font.

#### Scenario: Font family is Noto Sans by default
- **WHEN** any text is rendered and the user has not changed the font
- **THEN** the default font family is "Noto Sans" (set on the root element via `.font_family()`)

#### Scenario: Font family switches to OpenDyslexic
- **WHEN** the user selects "OpenDyslexic" in Settings
- **THEN** the root element SHALL use `.font_family("OpenDyslexic")`
- **THEN** all rendered text SHALL use the OpenDyslexic font

### Requirement: Rose Pine theme variants
The system SHALL provide two Rose Pine theme variants: `rose_pine()` (dark) and `rose_pine_dawn()` (light). Each constructor returns a complete, usable `Theme` with Rose Pine palette colors mapped to semantic tokens. Each theme SHALL carry a stable string ID accessible via `theme.id()`. The Rose Pine variants SHALL remain the default themes.

#### Scenario: Rose Pine Dawn is the default
- **WHEN** the application starts with no saved settings
- **THEN** the system SHALL initialize `GlobalSettings` with default settings (light mode, `"rose-pine-dawn"`)
- **THEN** the cached active theme SHALL be Rose Pine Dawn

#### Scenario: Rose Pine dark theme is complete
- **WHEN** `rose_pine()` is called
- **THEN** all color fields, font configuration, and text styles are populated with Rose Pine dark palette values
- **THEN** the theme ID SHALL be `"rose-pine"`

### Requirement: Theme switching from DevTestPage
The system SHALL allow switching between Rose Pine (dark) and Rose Pine Dawn (light) themes at runtime from the DevTestPage. The page SHALL render "Dark" and "Light" toggle buttons. Switching themes SHALL call `update_settings()` which recomputes the active theme, persists to disk, and triggers a full UI re-render. The system SHALL also support switching to and from the high-contrast themes via the Settings page.

#### Scenario: Switch to dark theme
- **WHEN** the user clicks the "Dark" button on DevTestPage
- **THEN** `update_settings(|s| s.theme_mode = ThemeMode::Dark, cx)` SHALL be called
- **THEN** the active theme SHALL switch to the theme identified by the saved dark theme ID
- **THEN** all visible UI elements update to use the active theme's colors

#### Scenario: Switch to light theme
- **WHEN** the user clicks the "Light" button on DevTestPage
- **THEN** `update_settings(|s| s.theme_mode = ThemeMode::Light, cx)` SHALL be called
- **THEN** the active theme SHALL switch to the theme identified by the saved light theme ID
- **THEN** all visible UI elements update to use the active theme's colors
