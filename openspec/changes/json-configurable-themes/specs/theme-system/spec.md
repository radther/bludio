## MODIFIED Requirements

### Requirement: Theme provides semantic color tokens
The system SHALL provide a `ThemeColors` struct with named color fields for background levels, text, accent colors, borders, and status indicators. All UI components MUST reference theme colors rather than hardcoded values.

#### Scenario: Dark theme colors are applied
- **WHEN** a dark theme is active
- **THEN** the background color is dark, text is light, and status colors are visible against the dark background

#### Scenario: Light theme colors are applied
- **WHEN** a light theme is active
- **THEN** the background color is light, text is dark, and status colors are visible against the light background

#### Scenario: Semantic color access
- **WHEN** a component needs the primary background color
- **THEN** it accesses `theme.colors.background` rather than `hsla(...)`

#### Scenario: Multiple accent colors
- **WHEN** a component needs an accent color
- **THEN** it uses the appropriate domain accent: `bluetooth_accent`, `audio_accent`, or `dev_accent`

### Requirement: Global theme access
The system SHALL store the active theme as a cached `Arc<Theme>` inside a `GlobalSettings` GPUI global. The active theme SHALL be recomputed automatically whenever settings change. The `theme(cx)` free function SHALL return `&Arc<Theme>` from the cached value.

#### Scenario: Access theme from any component
- **WHEN** any UI component calls `theme(cx)`
- **THEN** it SHALL receive `&Arc<Theme>` from `GlobalSettings` without needing access to `BludioApp`

#### Scenario: Theme survives async contexts
- **WHEN** an async task spawns and later updates UI state
- **THEN** it can clone the `Arc<Theme>` before spawning and use it to style elements created in the callback

### Requirement: TextStyle convenience method
The system SHALL provide a `.styled()` extension method (on the `StyledExt` trait) that applies font size and weight from a text style tuple. The `TextStyleSet` SHALL provide: `body` (1.0rem, medium), `body2` (0.875rem, medium), `heading` (1.5rem, extra bold), and `caption` (0.75rem, bold). The `TextStyleSet` is no longer stored on the `Theme` struct; it is constructed on demand or accessed via a shared default.

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
The system SHALL provide two Rose Pine theme variants: `rose-pine` (dark) and `rose-pine-dawn` (light). Each variant is loaded from a JSON file in `assets/themes/dark/` and `assets/themes/light/`. Each theme SHALL carry a stable string ID accessible via `theme.id`.

#### Scenario: Rose Pine Dawn is the default
- **WHEN** the application starts with no saved settings
- **THEN** the system SHALL initialize `GlobalSettings` with default settings (light mode, `"rose-pine-dawn"`)
- **THEN** the cached active theme SHALL be Rose Pine Dawn loaded from JSON

#### Scenario: Rose Pine dark theme is complete
- **WHEN** the `"rose-pine"` theme is loaded from JSON
- **THEN** all color fields are populated with Rose Pine dark palette values
- **THEN** the theme ID SHALL be `"rose-pine"`

### Requirement: Theme switching from DevTestPage
The system SHALL allow switching between light and dark themes at runtime from the DevTestPage. Switching themes SHALL call `update_settings()` which recomputes the active theme from the dynamic registry, persists to disk, and triggers a full UI re-render.

#### Scenario: Switch to dark theme
- **WHEN** the user clicks the "Dark" button on DevTestPage
- **THEN** `update_settings(|s| s.theme_mode = ThemeMode::Dark, cx)` SHALL be called
- **THEN** the active theme SHALL switch to the theme identified by the saved dark theme ID from the registry
- **THEN** all visible UI elements update to use the dark theme colors

#### Scenario: Switch to light theme
- **WHEN** the user clicks the "Light" button on DevTestPage
- **THEN** `update_settings(|s| s.theme_mode = ThemeMode::Light, cx)` SHALL be called
- **THEN** the active theme SHALL switch to the theme identified by the saved light theme ID from the registry
- **THEN** all visible UI elements update to use the light theme colors

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
The system SHALL allow the active theme to be set at runtime via `update_settings()`, which updates the `Settings` struct and recomputes the active theme from the dynamic registry based on the current mode and saved theme IDs.

#### Scenario: Apply theme by changing settings
- **WHEN** a page calls `update_settings(|s| s.theme_mode = ThemeMode::Dark, cx)`
- **THEN** the system SHALL look up the saved dark theme ID in the dynamic registry
- **THEN** the resulting theme SHALL become the active cached theme in `GlobalSettings`
- **THEN** the UI SHALL re-render

#### Scenario: Theme file structure
- **WHEN** a developer looks at the theme module
- **THEN** it SHALL be organized as `src/ui/theme/mod.rs` (module root) + `types.rs` (core types) + `registry.rs` (dynamic registry) + `loader.rs` (JSON parsing)
- **THEN** there SHALL be no per-theme Rust files

### Requirement: Theme struct is simplified
The `Theme` struct SHALL contain only `id`, `appearance`, and `colors`. The `font_family` and `text_styles` fields SHALL be removed from `Theme`. The font family SHALL be applied globally at the root element from `settings(cx).font_family`. The `text_styles` SHALL be accessed via a shared default or constructed on demand.

#### Scenario: Theme no longer contains font_family
- **WHEN** a `Theme` is loaded from JSON
- **THEN** it SHALL NOT have a `font_family` field
- **THEN** the root element SHALL apply `.font_family(settings(cx).font_family.clone())` instead

#### Scenario: Theme no longer contains text_styles
- **WHEN** a `Theme` is loaded from JSON
- **THEN** it SHALL NOT have a `text_styles` field
- **THEN** components SHALL use `TextStyleSet::default()` or a shared global for text styles

## REMOVED Requirements

### Requirement: Theme provides font configuration
**Reason**: Font family is now a user setting in `Settings`, not a theme property. Text styles are not theme-specific.
**Migration**: Access the active font family via `settings(cx).font_family`. Apply it at the root element. Access text styles via `TextStyleSet::default()` or a shared global.

### Requirement: Active font family is applied at the root
**Reason**: This requirement is now handled by the settings system, not the theme system.
**Migration**: The `BludioApp` root render applies `.font_family(settings(cx).font_family.clone())`. No theme involvement is needed.
