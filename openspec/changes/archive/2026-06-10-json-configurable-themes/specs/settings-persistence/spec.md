## MODIFIED Requirements

### Requirement: Settings are loaded at application startup
The system SHALL load user settings from a JSON file at application startup. If the file is missing or cannot be parsed, the system SHALL use default values: theme mode Light, light theme ID `"rose-pine-dawn"`, dark theme ID `"rose-pine"`, `disable_animations` false, and `font_family` `"Noto Sans"`. After loading settings, the system SHALL compute the active theme by looking up the appropriate theme ID in the dynamic theme registry.

#### Scenario: Settings file exists and is valid
- **WHEN** the application starts
- **THEN** the system SHALL read `~/.config/bludio/settings.json`
- **THEN** the loaded settings SHALL be stored in `GlobalSettings`
- **THEN** the active cached theme in `GlobalSettings` SHALL be recomputed from the loaded settings via the dynamic registry
- **THEN** the `disable_animations` and `font_family` values from the file SHALL be available

#### Scenario: Settings file is missing
- **WHEN** the application starts and no settings file exists
- **THEN** default settings SHALL be used
- **THEN** `disable_animations` SHALL be `false`
- **THEN** `font_family` SHALL be `"Noto Sans"`
- **THEN** the active theme SHALL be the default theme from the dynamic registry
- **THEN** no error SHALL be displayed

#### Scenario: Settings file is malformed
- **WHEN** the application starts and the settings file contains invalid JSON
- **THEN** default settings SHALL be used
- **THEN** `disable_animations` SHALL be `false`
- **THEN** `font_family` SHALL be `"Noto Sans"`
- **THEN** the active theme SHALL be the default theme from the dynamic registry
- **THEN** no error SHALL be displayed

### Requirement: Settings are saved when changed
The system SHALL write the current settings to the JSON file whenever a setting changes. The write SHALL be synchronous and SHALL overwrite the existing file. This includes changes to `disable_animations` and `font_family`. Theme changes are saved as theme IDs in the settings file.

#### Scenario: Theme mode is toggled
- **WHEN** the user toggles the theme mode
- **THEN** the settings file SHALL be updated with the new mode
- **THEN** the active theme SHALL be recomputed from the dynamic registry

#### Scenario: Light theme is changed
- **WHEN** the user selects a different light theme from the dropdown
- **THEN** the settings file SHALL be updated with the new light theme ID
- **THEN** the active theme SHALL be recomputed from the dynamic registry

#### Scenario: Dark theme is changed
- **WHEN** the user selects a different dark theme from the dropdown
- **THEN** the settings file SHALL be updated with the new dark theme ID
- **THEN** the active theme SHALL be recomputed from the dynamic registry

#### Scenario: Disable animations is changed
- **WHEN** the user toggles Disable Animations
- **THEN** the settings file SHALL be updated with the new `disable_animations` value

#### Scenario: Font family is changed
- **WHEN** the user selects a different font
- **THEN** the settings file SHALL be updated with the new `font_family` value
- **THEN** the root element SHALL re-render with the new font family

### Requirement: Settings struct includes disable_animations
The system SHALL include a `disable_animations: bool` field in the `Settings` struct. The field SHALL be serialized and deserialized alongside existing fields. When the field is missing from the settings file, it SHALL default to `false`.

#### Scenario: Settings file includes disable_animations
- **WHEN** the application loads a settings file containing `"disable_animations": true`
- **THEN** the loaded `Settings` SHALL have `disable_animations == true`

#### Scenario: Settings file missing disable_animations
- **WHEN** the application loads a settings file that does not contain `disable_animations`
- **THEN** the loaded `Settings` SHALL have `disable_animations == false`

### Requirement: Settings struct includes font_family
The system SHALL include a `font_family: String` field in the `Settings` struct. The field SHALL be serialized and deserialized alongside existing fields. When the field is missing from the settings file, it SHALL default to `"Noto Sans"`.

#### Scenario: Settings file includes font_family
- **WHEN** the application loads a settings file containing `"font_family": "OpenDyslexic"`
- **THEN** the loaded `Settings` SHALL have `font_family == "OpenDyslexic"`

#### Scenario: Settings file missing font_family
- **WHEN** the application loads a settings file that does not contain `font_family`
- **THEN** the loaded `Settings` SHALL have `font_family == "Noto Sans"`

### Requirement: New settings fields are saved when changed
The system SHALL write `disable_animations` and `font_family` to the settings JSON file whenever they change via `update_settings()`.

#### Scenario: Disable animations is toggled
- **WHEN** the user toggles Disable Animations
- **THEN** the settings file SHALL be updated with the new `disable_animations` value

#### Scenario: Font is changed
- **WHEN** the user selects a different font
- **THEN** the settings file SHALL be updated with the new `font_family` value

## ADDED Requirements

### Requirement: Missing theme ID falls back to default
The system SHALL detect when a saved theme ID does not exist in the dynamic registry (e.g., a user theme was deleted). In this case, the system SHALL fall back to the default theme: `"rose-pine-dawn"` for light mode and `"rose-pine"` for dark mode.

#### Scenario: Saved light theme ID is missing
- **WHEN** the settings file contains a light theme ID that is not in the registry
- **THEN** the system SHALL fall back to `"rose-pine-dawn"`
- **THEN** the settings file SHALL be updated with the fallback ID

#### Scenario: Saved dark theme ID is missing
- **WHEN** the settings file contains a dark theme ID that is not in the registry
- **THEN** the system SHALL fall back to `"rose-pine"`
- **THEN** the settings file SHALL be updated with the fallback ID
