## ADDED Requirements

### Requirement: Theme provides semantic color tokens
The system SHALL provide a `ThemeColors` struct with named color fields for background levels, text, accent, borders, and status indicators. All UI components MUST reference theme colors rather than hardcoded values.

#### Scenario: Dark theme colors are applied
- **WHEN** the dark theme is active
- **THEN** the background color is a dark near-black, text is light, accent is blue, and status colors are visible against the dark background

#### Scenario: Light theme colors are applied
- **WHEN** the light theme is active
- **THEN** the background color is a warm off-white, text is dark, accent is blue, and status colors are visible against the light background

#### Scenario: Semantic color access
- **WHEN** a component needs the primary background color
- **THEN** it accesses `theme.colors.background` rather than `hsla(...)`

### Requirement: Theme provides font configuration
The system SHALL store font family, text size scale, and font weight roles in the theme. The font family SHALL be "Noto Sans", provided as a bundled variable font covering all weights and italic.

#### Scenario: Font family is Noto Sans
- **WHEN** any text is rendered
- **THEN** the default font family is "Noto Sans"

#### Scenario: Font weights are named
- **WHEN** a component needs a text style for a specific role
- **THEN** it accesses the appropriate field from `text_styles` (e.g., `text_styles.heading`, `text_styles.caption`) and passes it to `.styled()`

### Requirement: Global theme access
The system SHALL store the active theme as a GPUI global, accessible from any `App` or `Window` context. The theme SHALL be wrapped in `Arc<Theme>` for cheap sharing.

#### Scenario: Access theme from any component
- **WHEN** any UI component renders
- **THEN** it can call `theme(cx)` to retrieve the active theme

#### Scenario: Theme survives async contexts
- **WHEN** an async task spawns and later updates UI state
- **THEN** it can clone the `Arc<Theme>` before spawning and use it to style elements created in the callback

### Requirement: TextStyle convenience method
The system SHALL provide a `.styled()` extension method on any `Styled` element that applies font size and weight from a text style tuple defined in `TextStyleSet`. Font family SHALL be inherited from the root element via GPUI's cascade. The `TextStyleSet` SHALL provide at minimum: `body`, `heading`, `body_small`, and `caption` fields.

#### Scenario: Apply heading text style
- **WHEN** a component calls `.styled(text_styles.heading)` on a `div`
- **THEN** the element gets bold weight and heading size (1rem)

#### Scenario: Apply caption text style
- **WHEN** a component calls `.styled(text_styles.caption)` on a `div`
- **THEN** the element gets medium weight and caption size (0.75rem)

### Requirement: Dark and light theme presets
The system SHALL provide `Theme::dark()` and `Theme::light()` constructors returning complete, usable themes.

#### Scenario: Dark theme is the default
- **WHEN** the application starts
- **THEN** the dark theme is active

#### Scenario: Light theme is complete
- **WHEN** `Theme::light()` is called
- **THEN** all color fields, font configuration, and text styles are populated with light-appropriate values

### Requirement: Theme switching from DevTestPage
The system SHALL allow switching between dark and light themes at runtime from the developer test page. Switching themes SHALL trigger a full UI re-render.

#### Scenario: Switch to light theme
- **WHEN** the user clicks a "Light" button on DevTestPage
- **THEN** all visible UI elements update to use light theme colors

#### Scenario: Switch back to dark theme
- **WHEN** the user clicks a "Dark" button on DevTestPage
- **THEN** all visible UI elements update to use dark theme colors

### Requirement: All existing hardcoded colors converted
Every `hsla(...)` call in the UI codebase that maps to a semantic color role SHALL be replaced with a theme reference. Interactive components (TextField, Dropdown, Slider) SHALL default to theme colors when no override is provided.

#### Scenario: Audio page uses theme surface color
- **WHEN** the AudioPage renders its header bar
- **THEN** the background color comes from `theme.colors.surface` rather than `hsla(0.0, 0.0, 0.14, 1.0)`

#### Scenario: Bluetooth page uses theme accent color
- **WHEN** the BluetoothPage renders the scan button
- **THEN** the button color comes from `theme.colors.accent` rather than `hsla(210.0/360.0, 0.7, 0.55, 1.0)`

#### Scenario: Tab bar uses theme colors
- **WHEN** the tab bar renders
- **THEN** its background, border, highlight, and hover colors come from the theme
