## ADDED Requirements

### Requirement: Settings page has an Accessibility section header
The system SHALL render an "Accessibility" section header on the Settings page above the Disable Animations toggle. The header SHALL use the `heading2` text style.

#### Scenario: Accessibility header is visible
- **WHEN** the Settings page renders
- **THEN** an "Accessibility" header SHALL appear above the Disable Animations toggle

### Requirement: Settings page has an Appearance section header
The system SHALL render an "Appearance" section header on the Settings page above the theme controls (Theme Mode, Light Theme, Dark Theme, and Font dropdown). The header SHALL use the `heading2` text style.

#### Scenario: Appearance header is visible
- **WHEN** the Settings page renders
- **THEN** an "Appearance" header SHALL appear above the theme mode toggle
- **THEN** the Light Theme and Dark Theme dropdowns SHALL appear below the Appearance header

### Requirement: Disable Animations toggle emits event on change
The system SHALL provide a "Disable Animations" control in the Accessibility section that emits a `SettingsEvent::DisableAnimationsChanged(bool)` when toggled.

#### Scenario: Toggle Disable Animations
- **WHEN** the user toggles the Disable Animations control
- **THEN** `SettingsPage` SHALL emit a `DisableAnimationsChanged` event with the new boolean value

### Requirement: Font dropdown emits event on selection change
The system SHALL provide a "Font" dropdown in the Appearance section that emits a `SettingsEvent::FontChanged(String)` when the selection changes.

#### Scenario: Change font
- **WHEN** the user selects a different font from the Font dropdown
- **THEN** `SettingsPage` SHALL emit a `FontChanged` event with the selected font family name

## MODIFIED Requirements

### Requirement: Settings page is accessible from the tab bar
The system SHALL render a Settings page when the Settings tab (index 5) is active. The page SHALL be an entity (`SettingsPage`) owned by `BludioApp`.

#### Scenario: Settings tab is active
- **WHEN** the user clicks the Settings tab (index 5, icon `bolt`)
- **THEN** the content area SHALL render the `SettingsPage` entity
- **THEN** the page SHALL display a scrollable list of settings controls including Accessibility and Appearance sections

### Requirement: Settings page uses scrollable header layout
The system SHALL render the Settings page with the same scrollable header layout as other pages. The header SHALL contain a static info text describing the page purpose.

#### Scenario: Settings page header is visible
- **WHEN** the Settings page renders
- **THEN** a header bar SHALL appear at the top with the text "Configure application appearance and behavior."
- **THEN** the content below SHALL be scrollable if it exceeds the available height
- **THEN** the content SHALL include the Accessibility section followed by the Appearance section
