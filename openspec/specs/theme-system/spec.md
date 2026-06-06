# Theme System

**Purpose:** Centralized design token system providing semantic color tokens, font configuration, text style roles, and light/dark theme switching. All UI components reference theme values rather than hardcoded colors.

**Status:** Implemented (see `src/ui/theme/` module)

---

## Requirements

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
The system SHALL store font family and text style roles in the theme. The font family SHALL be "Noto Sans", provided as a bundled variable font (`fonts/NotoSans.ttf` and `fonts/NotoSans-Italic.ttf`) covering all weights and italic. The active font family at runtime SHALL come from `Settings`, overriding the theme's default if the user has selected a different bundled font.

#### Scenario: Font family is Noto Sans by default
- **WHEN** any text is rendered and the user has not changed the font
- **THEN** the default font family is "Noto Sans" (set on the root element via `.font_family()`)

#### Scenario: Font family switches to OpenDyslexic
- **WHEN** the user selects "OpenDyslexic" in Settings
- **THEN** the root element SHALL use `.font_family("OpenDyslexic")`
- **THEN** all rendered text SHALL use the OpenDyslexic font

#### Scenario: Font weights are named
- **WHEN** a component needs a text style for a specific role
- **THEN** it accesses the appropriate field from `text_styles` (e.g., `text_styles.heading`, `text_styles.caption`) and passes it to `.styled()`

### Requirement: Global theme access
The system SHALL store the active theme as a cached `Arc<Theme>` inside a `GlobalSettings` GPUI global. The active theme SHALL be recomputed automatically whenever settings change. The `theme(cx)` free function SHALL return `&Arc<Theme>` from the cached value.

#### Scenario: Access theme from any component
- **WHEN** any UI component calls `theme(cx)`
- **THEN** it SHALL receive `&Arc<Theme>` from `GlobalSettings` without needing access to `BludioApp`

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
The system SHALL provide two Rose Pine theme variants: `rose_pine()` (dark) and `rose_pine_dawn()` (light). Each constructor returns a complete, usable `Theme` with Rose Pine palette colors mapped to semantic tokens. Each theme SHALL carry a stable string ID accessible via `theme.id()`.

#### Scenario: Rose Pine Dawn is the default
- **WHEN** the application starts with no saved settings
- **THEN** the system SHALL initialize `GlobalSettings` with default settings (light mode, `"rose-pine-dawn"`)
- **THEN** the cached active theme SHALL be Rose Pine Dawn

#### Scenario: Rose Pine dark theme is complete
- **WHEN** `rose_pine()` is called
- **THEN** all color fields, font configuration, and text styles are populated with Rose Pine dark palette values
- **THEN** the theme ID SHALL be `"rose-pine"`

### Requirement: Theme switching from DevTestPage
The system SHALL allow switching between Rose Pine (dark) and Rose Pine Dawn (light) themes at runtime from the DevTestPage. The page SHALL render "Dark" and "Light" toggle buttons. Switching themes SHALL call `update_settings()` which recomputes the active theme, persists to disk, and triggers a full UI re-render. The system SHALL also support switching to and from the high-contrast themes via the Settings page.

#### Scenario: Switch to dark theme
- **WHEN** the user clicks the "Dark" button on DevTestPage
- **THEN** `update_settings(|s| s.theme_mode = ThemeMode::Dark, cx)` SHALL be called
- **THEN** the active theme SHALL switch to the theme identified by the saved dark theme ID
- **THEN** all visible UI elements update to use Rose Pine dark colors

#### Scenario: Switch to light theme
- **WHEN** the user clicks the "Light" button on DevTestPage
- **THEN** `update_settings(|s| s.theme_mode = ThemeMode::Light, cx)` SHALL be called
- **THEN** the active theme SHALL switch to the theme identified by the saved light theme ID
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

### Requirement: Theme application from settings
The system SHALL allow the active theme to be set at runtime via `update_settings()`, which updates the `Settings` struct and recomputes the active theme from the registry based on the current mode and saved theme IDs.

#### Scenario: Apply theme by changing settings
- **WHEN** a page calls `update_settings(|s| s.theme_mode = ThemeMode::Dark, cx)`
- **THEN** the system SHALL look up the saved dark theme ID in the registry
- **THEN** the resulting theme SHALL become the active cached theme in `GlobalSettings`
- **THEN** the UI SHALL re-render

#### Scenario: Theme file structure
- **WHEN** a developer looks at the theme module
- **THEN** it SHALL be organized as `src/ui/theme/mod.rs` (module root) + `types.rs` (core types) + `rose_pine_theme.rs` (dark) + `rose_pine_dawn_theme.rs` (light) + `registry.rs` (ID-to-constructor map)

### Requirement: High Contrast Light theme constructor
The system SHALL provide a `high_contrast_light()` constructor that returns a complete `Theme` with a high-contrast light palette. The theme SHALL have the ID `"high-contrast-light"` and appearance `Appearance::Light`.

#### Scenario: Construct High Contrast Light
- **WHEN** `high_contrast_light()` is called
- **THEN** it SHALL return a `Theme` with ID `"high-contrast-light"`
- **THEN** the appearance SHALL be `Light`
- **THEN** all color tokens SHALL have high-contrast values
- **THEN** the font family SHALL be `"Noto Sans"` (or match the active user preference)

### Requirement: High Contrast Dark theme constructor
The system SHALL provide a `high_contrast_dark()` constructor that returns a complete `Theme` with a high-contrast dark palette. The theme SHALL have the ID `"high-contrast-dark"` and appearance `Appearance::Dark`.

#### Scenario: Construct High Contrast Dark
- **WHEN** `high_contrast_dark()` is called
- **THEN** it SHALL return a `Theme` with ID `"high-contrast-dark"`
- **THEN** the appearance SHALL be `Dark`
- **THEN** all color tokens SHALL have high-contrast values
- **THEN** the font family SHALL be `"Noto Sans"` (or match the active user preference)

### Requirement: Active font family is applied at the root
The system SHALL apply the active font family from `Settings` to the root UI element so that all child elements inherit it. The font family SHALL be accessible via `settings(cx).font_family`.

#### Scenario: Root element applies font family
- **WHEN** `BludioApp` renders its root element
- **THEN** it SHALL call `.font_family(settings(cx).font_family.clone())` on the root container
- **THEN** all text descendants SHALL inherit the selected font family
