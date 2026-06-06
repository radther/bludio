# High Contrast Themes

## Purpose

Accessibility-optimized theme variants that maximize luminance contrast for users with visual impairments. Includes High Contrast Light and High Contrast Dark options.

## Requirements

### Requirement: High Contrast Light theme exists
The system SHALL provide a "High Contrast Light" theme variant with a palette that maximizes luminance contrast. The theme SHALL have the stable ID `"high-contrast-light"` and SHALL be categorized as a light theme.

#### Scenario: High Contrast Light colors
- **WHEN** the "High Contrast Light" theme is active
- **THEN** the background SHALL be near-white (`#FFFFFF` or equivalent)
- **THEN** the text SHALL be near-black (`#000000` or equivalent)
- **THEN** accent colors SHALL be highly saturated and distinguishable from the background
- **THEN** all semantic color tokens SHALL be populated with high-contrast values

### Requirement: High Contrast Dark theme exists
The system SHALL provide a "High Contrast Dark" theme variant with a palette that maximizes luminance contrast. The theme SHALL have the stable ID `"high-contrast-dark"` and SHALL be categorized as a dark theme.

#### Scenario: High Contrast Dark colors
- **WHEN** the "High Contrast Dark" theme is active
- **THEN** the background SHALL be near-black (`#000000` or equivalent)
- **THEN** the text SHALL be near-white (`#FFFFFF` or equivalent)
- **THEN** accent colors SHALL be highly saturated and distinguishable from the background
- **THEN** all semantic color tokens SHALL be populated with high-contrast values

### Requirement: High contrast themes are selectable from Settings
The system SHALL include the high-contrast themes in the Light Theme and Dark Theme dropdowns on the Settings page. Selecting a high-contrast theme SHALL switch the active theme immediately.

#### Scenario: Select High Contrast Light
- **WHEN** the user selects "High Contrast Light" from the Light Theme dropdown
- **THEN** `SettingsPage` SHALL emit a light-theme-changed event with ID `"high-contrast-light"`
- **THEN** `BludioApp` SHALL call `update_settings()` to save the new light theme ID
- **THEN** if the current mode is Light, the active theme SHALL become High Contrast Light
- **THEN** the UI SHALL re-render with the high-contrast palette

#### Scenario: Select High Contrast Dark
- **WHEN** the user selects "High Contrast Dark" from the Dark Theme dropdown
- **THEN** `SettingsPage` SHALL emit a dark-theme-changed event with ID `"high-contrast-dark"`
- **THEN** `BludioApp` SHALL call `update_settings()` to save the new dark theme ID
- **THEN** if the current mode is Dark, the active theme SHALL become High Contrast Dark
- **THEN** the UI SHALL re-render with the high-contrast palette
