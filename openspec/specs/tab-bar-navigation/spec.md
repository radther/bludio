# tab-bar-navigation

## Purpose

Left-mounted icon tab bar providing page navigation for the application. Supports multiple pages with visual active-tab highlighting, hover effects, and tab click handling. The component is reusable and self-contained with no page-specific logic.

## Requirements

### Requirement: Left-mounted tab bar renders icon-only vertical tabs

The system SHALL render a fixed-width vertical tab bar on the left side of the application window. Each tab SHALL display only an SVG icon with no text label. The tab bar SHALL have a fixed width of 48 pixels and SHALL span the full height of the window.

#### Scenario: Tab bar is visible on app launch

- **WHEN** the application starts
- **THEN** a vertical bar SHALL appear on the left side of the window
- **THEN** the bar SHALL contain at least three icon buttons (tabs): Bluetooth, Output Devices, and Input Devices
- **THEN** the bar SHALL be 48 pixels wide

#### Scenario: Tab bar icons are loaded from SVG files

- **WHEN** the application renders the tab bar
- **THEN** each tab SHALL display an icon loaded from an SVG file in the icons directory
- **THEN** icons SHALL be rendered at 16×16 pixels within the tab

### Requirement: Tab selection switches the active page

The system SHALL allow users to click a tab icon to switch the main content area to the corresponding page. Only one tab SHALL be active at any time. Clicking the already-active tab SHALL have no effect.

#### Scenario: Clicking an inactive tab switches pages

- **WHEN** the user clicks a tab that is not currently active
- **THEN** the main content area SHALL render the page associated with the clicked tab
- **THEN** the clicked tab SHALL visually indicate it is now active

#### Scenario: Clicking the active tab has no effect

- **WHEN** the user clicks the tab that is already active
- **THEN** the main content area SHALL remain unchanged
- **THEN** no re-render of the page SHALL occur

### Requirement: Active tab is visually highlighted

The system SHALL visually distinguish the active tab from inactive tabs through background color. The active tab background SHALL use a contrasting surface color. Inactive tabs SHALL show a hover effect on mouse-over.

#### Scenario: Active tab has distinct background

- **WHEN** a tab is the active tab
- **THEN** its background color SHALL differ from inactive tabs
- **THEN** the active tab background SHALL contrast with the tab bar background

#### Scenario: Hovering an inactive tab shows visual feedback

- **WHEN** the user moves the mouse over an inactive tab
- **THEN** the tab SHALL change its background color to indicate hover state
- **THEN** the cursor SHALL change to a pointing hand

### Requirement: Bluetooth devices page is the default active page

The system SHALL set the Bluetooth devices page as the active page on application startup. The first tab in the tab bar SHALL correspond to the Bluetooth devices page.

#### Scenario: Bluetooth devices page is active on launch

- **WHEN** the application starts
- **THEN** the Bluetooth devices page SHALL be rendered in the main content area
- **THEN** the first tab (Bluetooth icon) SHALL be visually highlighted as active

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

### Requirement: Tab bar is reusable and self-contained

The tab bar component SHALL be implemented as a reusable module that accepts configuration data (tab definitions) and an active page identifier, emitting a callback when a tab is clicked. The component SHALL not contain any page-specific rendering logic or Bluetooth domain knowledge.

#### Scenario: Tab bar accepts tab definitions as input

- **WHEN** the tab bar is rendered
- **THEN** it SHALL accept a slice of tab configuration structs specifying icon and tooltip for each tab
- **THEN** it SHALL not hard-code page names or Bluetooth-specific logic
