## ADDED Requirements

### Requirement: Settings are loaded at application startup
The system SHALL load user settings from a JSON file at application startup. If the file is missing or cannot be parsed, the system SHALL use default values: theme mode Light, light theme ID `"rose-pine-dawn"`, dark theme ID `"rose-pine"`.

#### Scenario: Settings file exists and is valid
- **WHEN** the application starts
- **THEN** the system SHALL read `~/.config/bludio/settings.json`
- **THEN** the loaded settings SHALL be stored in `GlobalSettings`
- **THEN** the active cached theme in `GlobalSettings` SHALL be recomputed from the loaded settings

#### Scenario: Settings file is missing
- **WHEN** the application starts and no settings file exists
- **THEN** default settings SHALL be used
- **THEN** no error SHALL be displayed

#### Scenario: Settings file is malformed
- **WHEN** the application starts and the settings file contains invalid JSON
- **THEN** default settings SHALL be used
- **THEN** no error SHALL be displayed

### Requirement: Settings are saved when changed
The system SHALL write the current settings to the JSON file whenever a setting changes. The write SHALL be synchronous and SHALL overwrite the existing file.

#### Scenario: Theme mode is toggled
- **WHEN** the user toggles the theme mode
- **THEN** the settings file SHALL be updated with the new mode

#### Scenario: Light theme is changed
- **WHEN** the user selects a different light theme
- **THEN** the settings file SHALL be updated with the new light theme ID

#### Scenario: Dark theme is changed
- **WHEN** the user selects a different dark theme
- **THEN** the settings file SHALL be updated with the new dark theme ID
