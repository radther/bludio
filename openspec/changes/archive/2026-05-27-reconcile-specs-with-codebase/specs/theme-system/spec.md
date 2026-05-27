## MODIFIED Requirements

### Requirement: Theme provides semantic color tokens
The system SHALL provide a `ThemeColors` struct with named color fields for background levels, text, accent colors, borders, and status indicators. All UI components MUST reference theme colors rather than hardcoded values.

#### Scenario: Dark theme colors are applied
- **WHEN** the Rose Pine dark theme is active
- **THEN** the background color is a dark near-black (base), text is light, and status colors are visible against the dark background

#### Scenario: Light theme colors are applied
- **WHEN** the Rose Pine Dawn light theme is active
- **THEN** the background color is a warm off-white (base), text is dark, and status colors are visible against the light background

#### Scenario: Semantic color access
- **WHEN** a component needs the primary background color
- **THEN** it accesses `theme.colors.background` rather than `hsla(...)`

#### Scenario: Multiple accent colors
- **WHEN** a component needs an accent color
- **THEN** it uses the appropriate domain accent: `bluetooth_accent` (foam), `audio_accent` (iris), or `dev_accent` (pine)

### Requirement: Theme provides font configuration
The system SHALL store font family and text style roles in the theme. The font family SHALL be "Noto Sans", provided as a bundled variable font (`fonts/NotoSans.ttf` and `fonts/NotoSans-Italic.ttf`) covering all weights and italic.

#### Scenario: Font family is Noto Sans
- **WHEN** any text is rendered
- **THEN** the default font family is "Noto Sans" (set on the root element via `.font_family()`)

#### Scenario: Font weights are named
- **WHEN** a component needs a text style for a specific role
- **THEN** it accesses the appropriate field from `text_styles` (e.g., `text_styles.heading`, `text_styles.caption`) and passes it to `.styled()`

### Requirement: Global theme access
The system SHALL store the active theme as a GPUI global (`GlobalTheme`), accessible from any `App` or `Window` context. The theme SHALL be wrapped in `Arc<Theme>` for cheap sharing.

#### Scenario: Access theme from any component
- **WHEN** any UI component renders
- **THEN** it can call `theme(cx)` to retrieve the active `Arc<Theme>`

#### Scenario: Theme survives async contexts
- **WHEN** an async task spawns and later updates UI state
- **THEN** it can clone the `Arc<Theme>` before spawning and use it to style elements created in the callback

### Requirement: TextStyle convenience method
The system SHALL provide a `.styled()` extension method (on the `StyledExt` trait) that applies font size and weight from a text style tuple. The `TextStyleSet` SHALL provide: `body` (1.0rem, medium), `body2` (0.875rem, medium), `heading` (1.5rem, extra bold), and `caption` (0.75rem, bold).

#### Scenario: Apply heading text style
- **WHEN** a component calls `.styled(text_styles.heading)` on a `div`
- **THEN** the element gets extra bold weight and 1.5rem size

#### Scenario: Apply body2 text style
- **WHEN** a component calls `.styled(text_styles.body2)` on a `div`
- **THEN** the element gets medium weight and 0.875rem size

#### Scenario: Apply caption text style
- **WHEN** a component calls `.styled(text_styles.caption)` on a `div`
- **THEN** the element gets bold weight and 0.75rem size

### Requirement: Rose Pine theme variants
The system SHALL provide two Rose Pine theme variants: `rose_pine()` (dark) and `rose_pine_dawn()` (light). Each constructor returns a complete, usable `Theme` with Rose Pine palette colors mapped to semantic tokens.

#### Scenario: Rose Pine Dawn is the default
- **WHEN** the application starts
- **THEN** the Rose Pine Dawn (light) theme SHALL be active (set via `cx.set_global(GlobalTheme::new(rose_pine_dawn()))`)

#### Scenario: Rose Pine dark theme is complete
- **WHEN** `rose_pine()` is called
- **THEN** all color fields, font configuration, and text styles are populated with Rose Pine dark palette values

### Requirement: Theme switching from DevTestPage
The system SHALL allow switching between Rose Pine (dark) and Rose Pine Dawn (light) themes at runtime from the DevTestPage. The page SHALL render "Dark" and "Light" toggle buttons. Switching themes SHALL trigger a full UI re-render.

#### Scenario: Switch to dark theme
- **WHEN** the user clicks the "Dark" button on DevTestPage
- **THEN** `set_theme(rose_pine(), cx)` SHALL be called
- **THEN** all visible UI elements update to use Rose Pine dark colors

#### Scenario: Switch to light theme
- **WHEN** the user clicks the "Light" button on DevTestPage
- **THEN** `set_theme(rose_pine_dawn(), cx)` SHALL be called
- **THEN** all visible UI elements update to use Rose Pine Dawn colors

### Requirement: All existing hardcoded colors converted
Every `hsla(...)` call in the UI codebase that maps to a semantic color role SHALL be replaced with a theme reference. Interactive components (`TextField`, `Dropdown`, `Slider`) SHALL default to theme colors when no override is provided.

#### Scenario: Audio page uses theme surface color
- **WHEN** the AudioPage renders its header bar
- **THEN** the background color comes from `theme.colors.surface` rather than hardcoded values

#### Scenario: Bluetooth page uses theme accent color
- **WHEN** the BluetoothPage renders the scan button
- **THEN** the button color comes from `colors.bluetooth_accent` or `colors.element_background` rather than hardcoded values

#### Scenario: Tab bar uses theme colors
- **WHEN** the tab bar renders
- **THEN** its background, border, highlight, and hover colors come from the theme

#### Scenario: Theme file structure
- **WHEN** a developer looks at the theme module
- **THEN** it SHALL be organized as `src/ui/theme/mod.rs` (module root) + `types.rs` (core types) + `rose_pine_theme.rs` (dark) + `rose_pine_dawn_theme.rs` (light)
