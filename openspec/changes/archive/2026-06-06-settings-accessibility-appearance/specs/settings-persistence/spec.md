## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Settings are loaded at application startup
The system SHALL load user settings from a JSON file at application startup. If the file is missing or cannot be parsed, the system SHALL use default values: theme mode Light, light theme ID `"rose-pine-dawn"`, dark theme ID `"rose-pine"`, `disable_animations` false, and `font_family` `"Noto Sans"`.

#### Scenario: Settings file exists and is valid
- **WHEN** the application starts
- **THEN** the system SHALL read `~/.config/bludio/settings.json`
- **THEN** the loaded settings SHALL be stored in `GlobalSettings`
- **THEN** the active cached theme in `GlobalSettings` SHALL be recomputed from the loaded settings
- **THEN** the `disable_animations` and `font_family` values from the file SHALL be available

#### Scenario: Settings file is missing
- **WHEN** the application starts and no settings file exists
- **THEN** default settings SHALL be used
- **THEN** `disable_animations` SHALL be `false`
- **THEN** `font_family` SHALL be `"Noto Sans"`
- **THEN** no error SHALL be displayed

#### Scenario: Settings file is malformed
- **WHEN** the application starts and the settings file contains invalid JSON
- **THEN** default settings SHALL be used
- **THEN** `disable_animations` SHALL be `false`
- **THEN** `font_family` SHALL be `"Noto Sans"`
- **THEN** no error SHALL be displayed

### Requirement: Settings are saved when changed
The system SHALL write the current settings to the JSON file whenever a setting changes. The write SHALL be synchronous and SHALL overwrite the existing file. This includes changes to `disable_animations` and `font_family`.

#### Scenario: Theme mode is toggled
- **WHEN** the user toggles the theme mode
- **THEN** the settings file SHALL be updated with the new mode

#### Scenario: Light theme is changed
- **WHEN** the user selects a different light theme
- **THEN** the settings file SHALL be updated with the new light theme ID

#### Scenario: Dark theme is changed
- **WHEN** the user selects a different dark theme
- **THEN** the settings file SHALL be updated with the new dark theme ID

#### Scenario: Disable animations is changed
- **WHEN** the user toggles Disable Animations
- **THEN** the settings file SHALL be updated with the new `disable_animations` value

#### Scenario: Font family is changed
- **WHEN** the user selects a different font
- **THEN** the settings file SHALL be updated with the new `font_family` value
