# Theme Registry

## Purpose

A static, lookup-optimized registry mapping stable theme string IDs to theme constructors. Enables ID-based theme selection and categorizes themes by light/dark mode for settings UI dropdowns.

## Requirements

### Requirement: Each theme has a stable string ID
The system SHALL assign a unique `&'static str` ID to every theme variant. The ID SHALL be stored on the `Theme` struct and SHALL be accessible via a `theme.id()` method.

#### Scenario: Rose Pine Dawn has ID
- **WHEN** `rose_pine_dawn()` is called
- **THEN** the returned `Theme` SHALL have the ID `"rose-pine-dawn"`

#### Scenario: Rose Pine dark has ID
- **WHEN** `rose_pine()` is called
- **THEN** the returned `Theme` SHALL have the ID `"rose-pine"`

### Requirement: Theme registry maps IDs to theme constructors
The system SHALL provide a static theme registry (`LazyLock<HashMap<&'static str, fn() -> Arc<Theme>>>`) that maps each theme ID to a constructor function returning `Arc<Theme>`. The registry SHALL support O(1) lookup by ID.

#### Scenario: Lookup existing theme by ID
- **WHEN** the registry is queried with the ID `"rose-pine-dawn"`
- **THEN** it SHALL return a constructor function that produces a Rose Pine Dawn theme

#### Scenario: Lookup missing theme by ID
- **WHEN** the registry is queried with an unknown ID
- **THEN** it SHALL return `None`

### Requirement: Theme registry categorizes themes by mode
The system SHALL maintain separate lists of light theme IDs and dark theme IDs within or alongside the registry, so that the Settings page can populate its dropdowns with only relevant themes.

#### Scenario: Light themes list contains only light variants
- **WHEN** the system requests the list of light theme IDs
- **THEN** it SHALL contain `"rose-pine-dawn"`
- **THEN** it SHALL NOT contain `"rose-pine"`

#### Scenario: Dark themes list contains only dark variants
- **WHEN** the system requests the list of dark theme IDs
- **THEN** it SHALL contain `"rose-pine"`
- **THEN** it SHALL NOT contain `"rose-pine-dawn"`
