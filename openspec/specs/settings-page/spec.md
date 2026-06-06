# Settings Page

## Purpose

A scrollable settings page for configuring application appearance and behavior. Provides controls for theme mode toggle and per-mode theme selection via dropdowns.

## Requirements

### Requirement: Settings page is accessible from the tab bar
The system SHALL render a Settings page when the Settings tab (index 5) is active. The page SHALL be an entity (`SettingsPage`) owned by `BludioApp`.

#### Scenario: Settings tab is active
- **WHEN** the user clicks the Settings tab (index 5, icon `bolt`)
- **THEN** the content area SHALL render the `SettingsPage` entity
- **THEN** the page SHALL display a scrollable list of settings controls

### Requirement: Settings page uses scrollable header layout
The system SHALL render the Settings page with the same scrollable header layout as other pages. The header SHALL contain a static info text describing the page purpose.

#### Scenario: Settings page header is visible
- **WHEN** the Settings page renders
- **THEN** a header bar SHALL appear at the top with the text "Configure application appearance and behavior."
- **THEN** the content below SHALL be scrollable if it exceeds the available height

### Requirement: Theme mode setting toggles between light and dark
The system SHALL provide a Theme Mode setting on the Settings page with two mutually exclusive buttons: "Light" and "Dark". Clicking a button SHALL emit an event that `BludioApp` handles by calling `update_settings()`, which switches the theme mode, recomputes the active theme, persists to disk, and triggers re-render.

#### Scenario: Switch to dark mode
- **WHEN** the user clicks the "Dark" button
- **THEN** `SettingsPage` SHALL emit a mode-changed event
- **THEN** `BludioApp` SHALL call `update_settings(|s| s.theme_mode = ThemeMode::Dark, cx)`
- **THEN** the active theme SHALL switch to the theme identified by the saved dark theme ID
- **THEN** the settings SHALL be persisted to disk
- **THEN** the UI SHALL re-render with the dark theme

#### Scenario: Switch to light mode
- **WHEN** the user clicks the "Light" button
- **THEN** `SettingsPage` SHALL emit a mode-changed event
- **THEN** `BludioApp` SHALL call `update_settings(|s| s.theme_mode = ThemeMode::Light, cx)`
- **THEN** the active theme SHALL switch to the theme identified by the saved light theme ID
- **THEN** the settings SHALL be persisted to disk
- **THEN** the UI SHALL re-render with the light theme

### Requirement: Light theme dropdown selects the active light theme
The system SHALL provide a Light Theme dropdown on the Settings page. The dropdown SHALL list all themes registered as light variants. Selecting a theme SHALL emit an event that `BludioApp` handles by calling `update_settings()`.

#### Scenario: Select a light theme
- **WHEN** the user selects a theme from the Light Theme dropdown
- **THEN** `SettingsPage` SHALL emit a light-theme-changed event
- **THEN** `BludioApp` SHALL call `update_settings()` to update the saved light theme ID
- **THEN** if the current theme mode is Light, the active theme SHALL switch to the selected theme
- **THEN** the settings SHALL be persisted to disk
- **THEN** the UI SHALL re-render

### Requirement: Dark theme dropdown selects the active dark theme
The system SHALL provide a Dark Theme dropdown on the Settings page. The dropdown SHALL list all themes registered as dark variants. Selecting a theme SHALL emit an event that `BludioApp` handles by calling `update_settings()`.

#### Scenario: Select a dark theme
- **WHEN** the user selects a theme from the Dark Theme dropdown
- **THEN** `SettingsPage` SHALL emit a dark-theme-changed event
- **THEN** `BludioApp` SHALL call `update_settings()` to update the saved dark theme ID
- **THEN** if the current theme mode is Dark, the active theme SHALL switch to the selected theme
- **THEN** the settings SHALL be persisted to disk
- **THEN** the UI SHALL re-render
