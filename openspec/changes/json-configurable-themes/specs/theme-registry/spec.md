## MODIFIED Requirements

### Requirement: Each theme has a stable string ID
The system SHALL assign a unique string ID to every theme. The ID SHALL be stored on the `Theme` struct and SHALL be accessible via `theme.id`. IDs are read from the JSON `id` field at load time.

#### Scenario: Rose Pine Dawn has ID
- **WHEN** the "Rose Pine Dawn" theme is loaded from JSON
- **THEN** the returned `Theme` SHALL have the ID `"rose-pine-dawn"`

#### Scenario: Rose Pine dark has ID
- **WHEN** the "Rose Pine" theme is loaded from JSON
- **THEN** the returned `Theme` SHALL have the ID `"rose-pine"`

### Requirement: Theme registry maps IDs to theme instances
The system SHALL provide a dynamic theme registry that maps each theme ID to an `Arc<Theme>`. The registry SHALL be populated at startup by scanning built-in JSON theme files (embedded via `include_str!`) and user theme directories. The registry SHALL support O(1) lookup by ID.

#### Scenario: Lookup existing theme by ID
- **WHEN** the registry is queried with the ID `"rose-pine-dawn"`
- **THEN** it SHALL return the `Arc<Theme>` loaded from the corresponding JSON file

#### Scenario: Lookup missing theme by ID
- **WHEN** the registry is queried with an unknown ID
- **THEN** it SHALL return `None`

### Requirement: High Contrast Light is registered as a light theme
The system SHALL load the `"high-contrast-light"` theme from `assets/themes/light/high-contrast-light.json` and register it in the dynamic theme registry. The ID SHALL be included in the list of light theme IDs.

#### Scenario: Lookup High Contrast Light
- **WHEN** the registry is queried with the ID `"high-contrast-light"`
- **THEN** it SHALL return the `Arc<Theme>` loaded from the JSON file
- **THEN** `light_theme_ids()` SHALL contain `"high-contrast-light"`

### Requirement: High Contrast Dark is registered as a dark theme
The system SHALL load the `"high-contrast-dark"` theme from `assets/themes/dark/high-contrast-dark.json` and register it in the dynamic theme registry. The ID SHALL be included in the list of dark theme IDs.

#### Scenario: Lookup High Contrast Dark
- **WHEN** the registry is queried with the ID `"high-contrast-dark"`
- **THEN** it SHALL return the `Arc<Theme>` loaded from the JSON file
- **THEN** `dark_theme_ids()` SHALL contain `"high-contrast-dark"`

### Requirement: Theme display names come from JSON
The system SHALL provide human-friendly display names for themes via the `display_name` field in each JSON theme file. The Settings page dropdowns SHALL read display names from the registry rather than a hardcoded function.

#### Scenario: Display name for High Contrast Light
- **WHEN** the Settings page needs a display name for `"high-contrast-light"`
- **THEN** it SHALL read the `display_name` from the loaded `Theme` metadata

#### Scenario: Display name for a user theme
- **WHEN** a user adds a custom theme with `"display_name": "My Custom Theme"`
- **THEN** the Settings page SHALL show "My Custom Theme" in the dropdown

### Requirement: Theme registry categorizes themes by mode
The system SHALL maintain separate lists of light theme IDs and dark theme IDs within the dynamic registry, populated according to the directory each theme was loaded from. The Settings page SHALL populate its dropdowns with only relevant themes.

#### Scenario: Light themes list contains all light variants
- **WHEN** the system requests the list of light theme IDs
- **THEN** it SHALL contain `"rose-pine-dawn"`, `"high-contrast-light"`, and any user light themes
- **THEN** it SHALL NOT contain any dark theme IDs

#### Scenario: Dark themes list contains all dark variants
- **WHEN** the system requests the list of dark theme IDs
- **THEN** it SHALL contain `"rose-pine"`, `"high-contrast-dark"`, and any user dark themes
- **THEN** it SHALL NOT contain any light theme IDs

## ADDED Requirements

### Requirement: Dynamic registry is populated at startup
The registry SHALL be populated once at application startup by loading all built-in JSON themes and all valid user themes from `~/.config/bludio/themes/`. The registry SHALL be accessible via a `LazyLock` or similar mechanism.

#### Scenario: Registry contains all built-in themes
- **WHEN** the application starts
- **THEN** the registry SHALL contain all built-in themes loaded from `assets/themes/`

#### Scenario: Registry contains user themes
- **WHEN** the user has valid custom themes in `~/.config/bludio/themes/`
- **THEN** the registry SHALL contain those themes alongside the built-in ones

### Requirement: Registry provides display name lookup
The system SHALL provide a function that returns the `display_name` for a given theme ID from the registry.

#### Scenario: Display name lookup
- **WHEN** the system calls `theme_display_name("rose-pine-dawn")`
- **THEN** it SHALL return the display name from the loaded JSON theme file
