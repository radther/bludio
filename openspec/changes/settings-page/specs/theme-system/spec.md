## ADDED Requirements

### Requirement: Theme is derived from GlobalSettings
The system SHALL store the active theme as a cached `Arc<Theme>` inside a `GlobalSettings` GPUI global. The active theme SHALL be recomputed automatically whenever settings change. The `theme(cx)` free function SHALL return `&Arc<Theme>` from the cached value.

#### Scenario: Access theme from any component
- **WHEN** any UI component calls `theme(cx)`
- **THEN** it SHALL receive `&Arc<Theme>` from `GlobalSettings` without needing access to `BludioApp`

### Requirement: Theme application from settings
The system SHALL allow the active theme to be set at runtime via `update_settings()`, which updates the `Settings` struct and recomputes the active theme from the registry based on the current mode and saved theme IDs.

#### Scenario: Apply theme by changing settings
- **WHEN** a page calls `update_settings(|s| s.theme_mode = ThemeMode::Dark, cx)`
- **THEN** the system SHALL look up the saved dark theme ID in the registry
- **THEN** the resulting theme SHALL become the active cached theme in `GlobalSettings`
- **THEN** the UI SHALL re-render

## MODIFIED Requirements

### Requirement: Rose Pine theme variants
The system SHALL provide two Rose Pine theme variants: `rose_pine()` (dark) and `rose_pine_dawn()` (light). Each constructor returns a complete, usable `Theme` with Rose Pine palette colors mapped to semantic tokens. Each theme SHALL carry a stable string ID accessible via `theme.id()`.

#### Scenario: Rose Pine Dawn is the default
- **WHEN** the application starts with no saved settings
- **THEN** the system SHALL initialize `GlobalSettings` with default settings (light mode, `"rose-pine-dawn"`)
- **THEN** the cached active theme SHALL be Rose Pine Dawn

#### Scenario: Rose Pine dark theme is complete
- **WHEN** `rose_pine()` is called
- **THEN** all color fields, font configuration, and text styles are populated with Rose Pine dark palette values
- **THEN** the theme ID SHALL be `"rose-pine"`

#### Scenario: Rose Pine Dawn has correct ID
- **WHEN** `rose_pine_dawn()` is called
- **THEN** the theme ID SHALL be `"rose-pine-dawn"`
