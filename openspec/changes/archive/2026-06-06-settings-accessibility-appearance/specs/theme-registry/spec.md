## ADDED Requirements

### Requirement: High Contrast Light is registered as a light theme
The system SHALL register the `"high-contrast-light"` theme ID in the theme registry with its constructor mapped. The ID SHALL be included in the list of light theme IDs returned by `light_theme_ids()`.

#### Scenario: Lookup High Contrast Light
- **WHEN** the registry is queried with the ID `"high-contrast-light"`
- **THEN** it SHALL return a constructor function that produces a High Contrast Light theme
- **THEN** `light_theme_ids()` SHALL contain `"high-contrast-light"`

### Requirement: High Contrast Dark is registered as a dark theme
The system SHALL register the `"high-contrast-dark"` theme ID in the theme registry with its constructor mapped. The ID SHALL be included in the list of dark theme IDs returned by `dark_theme_ids()`.

#### Scenario: Lookup High Contrast Dark
- **WHEN** the registry is queried with the ID `"high-contrast-dark"`
- **THEN** it SHALL return a constructor function that produces a High Contrast Dark theme
- **THEN** `dark_theme_ids()` SHALL contain `"high-contrast-dark"`

### Requirement: Theme display names include high-contrast variants
The system SHALL provide human-friendly display names for the high-contrast theme IDs so that the Settings page dropdowns can show readable labels.

#### Scenario: Display name for High Contrast Light
- **WHEN** the Settings page needs a display name for `"high-contrast-light"`
- **THEN** it SHALL be "High Contrast Light"

#### Scenario: Display name for High Contrast Dark
- **WHEN** the Settings page needs a display name for `"high-contrast-dark"`
- **THEN** it SHALL be "High Contrast Dark"

## MODIFIED Requirements

### Requirement: Theme registry categorizes themes by mode
The system SHALL maintain separate lists of light theme IDs and dark theme IDs within or alongside the registry, so that the Settings page can populate its dropdowns with only relevant themes.

#### Scenario: Light themes list contains all light variants
- **WHEN** the system requests the list of light theme IDs
- **THEN** it SHALL contain `"rose-pine-dawn"` and `"high-contrast-light"`
- **THEN** it SHALL NOT contain `"rose-pine"` or `"high-contrast-dark"`

#### Scenario: Dark themes list contains all dark variants
- **WHEN** the system requests the list of dark theme IDs
- **THEN** it SHALL contain `"rose-pine"` and `"high-contrast-dark"`
- **THEN** it SHALL NOT contain `"rose-pine-dawn"` or `"high-contrast-light"`
