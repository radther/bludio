## ADDED Requirements

### Requirement: User themes are loaded from ~/.config/bludio/themes/
The system SHALL scan the directories `~/.config/bludio/themes/light/` and `~/.config/bludio/themes/dark/` at application startup. Each `.json` file found SHALL be passed to the theme loader. Successfully loaded themes SHALL be added to the theme registry alongside built-in themes.

#### Scenario: User has custom light themes
- **WHEN** `~/.config/bludio/themes/light/` contains one or more `.json` files
- **THEN** the system SHALL load each valid file as a light theme
- **THEN** the loaded themes SHALL appear in the Light Theme dropdown on the Settings page

#### Scenario: User has custom dark themes
- **WHEN** `~/.config/bludio/themes/dark/` contains one or more `.json` files
- **THEN** the system SHALL load each valid file as a dark theme
- **THEN** the loaded themes SHALL appear in the Dark Theme dropdown on the Settings page

#### Scenario: User theme directory does not exist
- **WHEN** the user has not created `~/.config/bludio/themes/`
- **THEN** the system SHALL start normally with only built-in themes
- **THEN** no error SHALL be displayed

### Requirement: User themes override built-in themes with the same ID
If a user theme file has the same `id` as a built-in theme, the user theme SHALL take precedence in the registry. The built-in theme SHALL be shadowed.

#### Scenario: User overrides Rose Pine Dawn
- **WHEN** the user places a theme file with `"id": "rose-pine-dawn"` in `~/.config/bludio/themes/light/`
- **THEN** the system SHALL use the user's version of "Rose Pine Dawn" instead of the built-in one
- **THEN** the Settings page dropdown SHALL still show the user's `display_name`

### Requirement: Malformed user themes are skipped with a warning
If a user theme file fails to parse, validate, or load, the system SHALL log a warning and skip that file. The application SHALL continue to start with all valid themes.

#### Scenario: Invalid JSON in user theme
- **WHEN** a user places a file with invalid JSON syntax in the themes directory
- **THEN** the system SHALL log a warning with the filename and error
- **THEN** the application SHALL continue to start
- **THEN** the invalid file SHALL NOT appear in any dropdown

#### Scenario: Missing required field in user theme
- **WHEN** a user theme JSON omits a required field (e.g., `id` or `display_name`)
- **THEN** the system SHALL log a warning with the filename and error
- **THEN** the application SHALL continue to start
