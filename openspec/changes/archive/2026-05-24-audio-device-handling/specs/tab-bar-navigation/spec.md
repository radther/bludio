## MODIFIED Requirements

### Requirement: Left-mounted tab bar renders icon-only vertical tabs

The system SHALL render a fixed-width vertical tab bar on the left side of the application window. Each tab SHALL display only an SVG icon with no text label. The tab bar SHALL have a fixed width of 48 pixels and SHALL span the full height of the window.

#### Scenario: Tab bar is visible on app launch

- **WHEN** the application starts
- **THEN** a vertical bar SHALL appear on the left side of the window
- **THEN** the bar SHALL contain three icon buttons (tabs): Bluetooth, Output Devices, and Input Devices
- **THEN** the bar SHALL be 48 pixels wide

#### Scenario: Tab bar icons are loaded from SVG files

- **WHEN** the application renders the tab bar
- **THEN** each tab SHALL display an icon loaded from an SVG file in the icons directory
- **THEN** icons SHALL be rendered at 16×16 pixels within the tab

## REMOVED Requirements

### Requirement: Placeholder page renders centered text

**Reason**: The "Page 2" placeholder tab was a temporary fixture to verify tab bar navigation worked. With Output Devices and Input Devices tabs providing real functionality, the placeholder is no longer needed and its code should be removed.
**Migration**: Remove the `Page::Page2` enum variant and its render match arm from `src/app.rs`. The tab bar will have 3 tabs: Bluetooth (index 0), Output Devices (index 1), Input Devices (index 2).

## ADDED Requirements

### Requirement: Output Devices tab navigates to output device page

The system SHALL provide an "Output Devices" tab in the tab bar. When clicked, the main content area SHALL render the output devices page showing all PulseAudio sinks with their properties and controls.

#### Scenario: Output Devices tab is present in tab bar

- **WHEN** the application starts
- **THEN** a tab with an audio device icon SHALL appear in the tab bar at index 1
- **THEN** the tab SHALL have the tooltip "Output Devices"

#### Scenario: Clicking Output Devices tab renders output page

- **WHEN** the user clicks the Output Devices tab
- **THEN** the main content area SHALL render the output devices list
- **THEN** the Output Devices tab SHALL be visually highlighted as active

### Requirement: Input Devices tab navigates to input device page

The system SHALL provide an "Input Devices" tab in the tab bar. When clicked, the main content area SHALL render the input devices page showing all PulseAudio sources with their properties and controls.

#### Scenario: Input Devices tab is present in tab bar

- **WHEN** the application starts
- **THEN** a tab with a microphone icon SHALL appear in the tab bar at index 2
- **THEN** the tab SHALL have the tooltip "Input Devices"

#### Scenario: Clicking Input Devices tab renders input page

- **WHEN** the user clicks the Input Devices tab
- **THEN** the main content area SHALL render the input devices list
- **THEN** the Input Devices tab SHALL be visually highlighted as active
