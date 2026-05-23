## ADDED Requirements

### Requirement: Left-mounted tab bar renders icon-only vertical tabs

The system SHALL render a fixed-width vertical tab bar on the left side of the application window. Each tab SHALL display only an SVG icon with no text label. The tab bar SHALL have a fixed width of 48 pixels and SHALL span the full height of the window.

#### Scenario: Tab bar is visible on app launch

- **WHEN** the application starts
- **THEN** a vertical bar SHALL appear on the left side of the window
- **THEN** the bar SHALL contain at least two icon buttons (tabs)
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

### Requirement: Placeholder page renders centered text

The system SHALL include a second tab that, when active, displays a centered text placeholder in the main content area. This page SHALL have no functional behavior beyond displaying the placeholder text.

#### Scenario: Placeholder page displays centered text

- **WHEN** the user clicks the second tab
- **THEN** the main content area SHALL display the text "Page 2" centered horizontally and vertically
- **THEN** the first tab SHALL no longer be highlighted

### Requirement: Tab bar is reusable and self-contained

The tab bar component SHALL be implemented as a reusable module that accepts configuration data (tab definitions) and an active page identifier, emitting a callback when a tab is clicked. The component SHALL not contain any page-specific rendering logic or Bluetooth domain knowledge.

#### Scenario: Tab bar accepts tab definitions as input

- **WHEN** the tab bar is rendered
- **THEN** it SHALL accept a slice of tab configuration structs specifying icon and tooltip for each tab
- **THEN** it SHALL not hard-code page names or Bluetooth-specific logic
