## ADDED Requirements

### Requirement: Theme JSON has a stable ID
The system SHALL require every theme JSON file to contain an `id` field. The value SHALL be a kebab-case string used as the stable identifier for settings keys and registry lookups.

#### Scenario: Valid theme ID
- **WHEN** a theme JSON contains `"id": "rose-pine-dawn"`
- **THEN** the loader SHALL use that string as the theme's `id`

#### Scenario: Duplicate ID detection
- **WHEN** two theme files (built-in or user) have the same `id`
- **THEN** the system SHALL use a last-write-wins policy: user themes override built-in themes with the same ID

### Requirement: Theme JSON has a display name
The system SHALL require every theme JSON file to contain a `display_name` field. The value SHALL be a human-readable string shown in the Settings dropdowns.

#### Scenario: Valid display name
- **WHEN** a theme JSON contains `"display_name": "Rose Pine Dawn"`
- **THEN** the Settings page SHALL show "Rose Pine Dawn" in the dropdown

### Requirement: Theme JSON defines a color palette
The system SHALL require every theme JSON file to contain a `colors` dictionary. Each key SHALL be a color name (e.g., `base`, `foam`, `gold`) and each value SHALL be a hex color string in the format `#RRGGBB`.

#### Scenario: Valid color palette
- **WHEN** a theme JSON contains `"colors": { "base": "#faf4ed", "foam": "#56949f" }`
- **THEN** the loader SHALL parse both values into `Hsla` colors

#### Scenario: Invalid hex color
- **WHEN** a color value is not a valid hex string (e.g., missing `#`, wrong length, or non-hex characters)
- **THEN** the loader SHALL return an error indicating the invalid color format

### Requirement: Theme JSON defines a semantic color mapping
The system SHALL require every theme JSON file to contain a `theme` dictionary. Each key SHALL be a semantic token name matching a field of `ThemeColors`. Each value SHALL be a string referencing a key in the `colors` dictionary.

#### Scenario: Valid semantic mapping
- **WHEN** a theme JSON contains `"theme": { "background": "base", "bluetooth_accent": "foam" }`
- **THEN** the loader SHALL resolve the palette references and populate the corresponding `ThemeColors` fields

#### Scenario: Unknown semantic token
- **WHEN** a theme JSON contains a key in the `theme` dictionary that does not match a known `ThemeColors` field
- **THEN** the loader SHALL return an error indicating the unknown semantic token

### Requirement: Theme mode is determined by directory
The system SHALL determine a theme's mode (Light or Dark) from the directory it is loaded from. Themes in `light/` directories SHALL have `Appearance::Light`. Themes in `dark/` directories SHALL have `Appearance::Dark`.

#### Scenario: Light theme from light directory
- **WHEN** a theme JSON is loaded from `assets/themes/light/`
- **THEN** the resulting `Theme` SHALL have `appearance: Appearance::Light`

#### Scenario: Dark theme from dark directory
- **WHEN** a theme JSON is loaded from `assets/themes/dark/`
- **THEN** the resulting `Theme` SHALL have `appearance: Appearance::Dark`
