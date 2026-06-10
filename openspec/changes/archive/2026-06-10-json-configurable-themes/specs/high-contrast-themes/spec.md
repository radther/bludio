## MODIFIED Requirements

### Requirement: High Contrast Light theme exists
The system SHALL provide a "High Contrast Light" theme variant with a palette that maximizes luminance contrast. The theme SHALL be loaded from `assets/themes/light/high-contrast-light.json`. The theme SHALL have the stable ID `"high-contrast-light"` and SHALL be categorized as a light theme.

#### Scenario: High Contrast Light colors
- **WHEN** the "High Contrast Light" theme is active
- **THEN** the background SHALL be near-white (`#FFFFFF` or equivalent)
- **THEN** the text SHALL be near-black (`#000000` or equivalent)
- **THEN** accent colors SHALL be highly saturated and distinguishable from the background
- **THEN** all semantic color tokens SHALL be populated with high-contrast values

#### Scenario: High Contrast Light is loaded from JSON
- **WHEN** the application starts
- **THEN** the theme SHALL be loaded from `assets/themes/light/high-contrast-light.json`
- **THEN** the loaded theme SHALL match the high-contrast color values

### Requirement: High Contrast Dark theme exists
The system SHALL provide a "High Contrast Dark" theme variant with a palette that maximizes luminance contrast. The theme SHALL be loaded from `assets/themes/dark/high-contrast-dark.json`. The theme SHALL have the stable ID `"high-contrast-dark"` and SHALL be categorized as a dark theme.

#### Scenario: High Contrast Dark colors
- **WHEN** the "High Contrast Dark" theme is active
- **THEN** the background SHALL be near-black (`#000000` or equivalent)
- **THEN** the text SHALL be near-white (`#FFFFFF` or equivalent)
- **THEN** accent colors SHALL be highly saturated and distinguishable from the background
- **THEN** all semantic color tokens SHALL be populated with high-contrast values

#### Scenario: High Contrast Dark is loaded from JSON
- **WHEN** the application starts
- **THEN** the theme SHALL be loaded from `assets/themes/dark/high-contrast-dark.json`
- **THEN** the loaded theme SHALL match the high-contrast color values

### Requirement: High contrast themes are selectable from Settings
The system SHALL include the high-contrast themes in the Light Theme and Dark Theme dropdowns on the Settings page. Selecting a high-contrast theme SHALL switch the active theme immediately. The display names SHALL come from the JSON `display_name` fields.

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

## REMOVED Requirements

### Requirement: High Contrast Light theme constructor
**Reason**: Themes are no longer constructed via Rust functions. They are loaded from JSON files.
**Migration**: The High Contrast Light theme is defined in `assets/themes/light/high-contrast-light.json`.

### Requirement: High Contrast Dark theme constructor
**Reason**: Themes are no longer constructed via Rust functions. They are loaded from JSON files.
**Migration**: The High Contrast Dark theme is defined in `assets/themes/dark/high-contrast-dark.json`.
