# tab-bar-navigation

## Purpose

Left-mounted icon tab bar providing page navigation for the application. Supports multiple pages with visual active-tab highlighting, hover effects, and tab click handling. The component is reusable and self-contained with no page-specific logic.

## Requirements

### Requirement: Left-mounted tab bar renders icon-only vertical tabs

The system SHALL render a `TabBar` entity as a fixed-width vertical tab bar on the left side of the application window. Each tab SHALL display only an SVG icon with no text label. The tab bar SHALL have a fixed width of 48 pixels and SHALL span the full height of the window. The tab bar SHALL accept a vector of `Tab` structs (icon function + tooltip string) and an initial active index.

#### Scenario: Tab bar is visible on app launch

- **WHEN** the application starts
- **THEN** a vertical `TabBar` entity SHALL appear on the left side of the window
- **THEN** the bar SHALL contain six icon tabs: Bluetooth, Output Devices, Input Devices, Configuration, Text Field Test, and Settings
- **THEN** the bar SHALL be 48 pixels wide

#### Scenario: Tab bar accepts Tab definitions as input

- **WHEN** the `TabBar` is created
- **THEN** it SHALL accept a `Vec<Tab>` where each `Tab` has an icon function (`fn() -> Svg`) and a tooltip string
- **THEN** the `TabBar` SHALL not hard-code page names or domain-specific logic

#### Scenario: Tab bar icons are loaded from SVG files

- **WHEN** the application renders the tab bar
- **THEN** each tab SHALL display an icon loaded from an SVG file in the icons directory
- **THEN** icons SHALL be rendered at 16×16 pixels within the tab

### Requirement: Tab selection switches the active page

The system SHALL allow users to click a tab icon to switch the main content area to the corresponding page. The `TabBar` entity SHALL emit a `TabBarEvent::TabClicked(index)` event. The parent (`BludioApp`) SHALL subscribe to this event and call `switch_to_tab()` to update the active page.

#### Scenario: Clicking an inactive tab switches pages

- **WHEN** the user clicks a tab that is not currently active
- **THEN** the `TabBar` SHALL emit `TabBarEvent::TabClicked(index)`
- **THEN** the parent SHALL update `active_page` to the corresponding `Page` variant
- **THEN** the clicked tab SHALL visually indicate it is now active

#### Scenario: Clicking the active tab has no effect

- **WHEN** the user clicks the tab that is already active
- **THEN** the `TabBar` SHALL still emit `TabBarEvent::TabClicked(index)`
- **THEN** the parent SHALL detect `active_page == new_page` and skip the update

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

### Requirement: Six tabs are configured

The system SHALL configure six tabs at indices 0–5: Bluetooth Devices (index 0, icon `radar`), Output Devices (index 1, icon `speaker`), Input Devices (index 2, icon `mic`), Configuration (index 3, icon `form`), Text Field Test (index 4, icon `test-tube-diagonal`), and Settings (index 5, icon `bolt`). Each tab SHALL have a tooltip matching its page name. The tab bar SHALL also contain one action item: Restart Bluetooth (icon `bluetooth`, tooltip "Restart Bluetooth service", action ID `"restart-bluetooth"`), anchored to the bottom.

#### Scenario: Six tabs and one action are present

- **WHEN** the application starts
- **THEN** tab 0 SHALL use `icons::bluetooth` (radar.svg) with tooltip "Bluetooth Devices"
- **THEN** tab 1 SHALL use `icons::audio_output` (speaker.svg) with tooltip "Output Devices"
- **THEN** tab 2 SHALL use `icons::audio_input` (mic.svg) with tooltip "Input Devices"
- **THEN** tab 3 SHALL use `icons::audio_card` (form.svg) with tooltip "Configuration"
- **THEN** tab 4 SHALL use `icons::text_field_test` (test-tube-diagonal.svg) with tooltip "Text Field Test"
- **THEN** tab 5 SHALL use `icons::bolt` (bolt.svg) with tooltip "Settings"
- **THEN** tab 0 (Bluetooth Devices) SHALL be the active tab on startup
- **THEN** one action item SHALL render at the bottom with icon `icons::bluetooth` and tooltip "Restart Bluetooth service"

### Requirement: Tab bar supports bottom-anchored action items

The system SHALL support action items rendered at the bottom of the tab bar, visually separated from navigation tabs. Action items SHALL be configured via an `actions` field on the `TabBar` containing `TabAction` structs (icon function, tooltip string, action ID string). Action items SHALL be rendered below a flex spacer that pushes them to the bottom of the column.

#### Scenario: Action item appears at the bottom

- **WHEN** the `TabBar` is created with one or more actions
- **THEN** the action items SHALL render below all navigation tabs
- **THEN** the action items SHALL be anchored to the bottom of the tab bar
- **THEN** a visual separator or spacing SHALL exist between tabs and actions

#### Scenario: Action item has tooltip

- **WHEN** the user hovers over an action item
- **THEN** the tooltip text from the `TabAction` SHALL be displayed

### Requirement: Action item clicks emit events

The system SHALL emit `TabBarEvent::ActionButtonClicked(action_id)` when an action item is clicked. The parent (`BludioApp`) SHALL subscribe to this event to handle the action.

#### Scenario: Clicking an action item emits event

- **WHEN** the user clicks an action item
- **THEN** the `TabBar` SHALL emit `TabBarEvent::ActionButtonClicked` with the action's ID string
- **THEN** the tab bar SHALL NOT change the active tab

### Requirement: Action item disabled state

The system SHALL support disabling individual action items. A disabled action item SHALL be visually dimmed and SHALL not respond to clicks or emit events.

#### Scenario: Disabled action item is visually dimmed

- **WHEN** an action item's disabled flag is true
- **THEN** the action item SHALL render with reduced opacity
- **THEN** the cursor SHALL NOT change to pointing hand on hover

#### Scenario: Disabled action item does not emit events

- **WHEN** a disabled action item is clicked
- **THEN** no `ActionButtonClicked` event SHALL be emitted

### Requirement: Tab bar supports keyboard shortcuts

The system SHALL observe keystrokes globally and switch tabs when Ctrl+1 through Ctrl+6 are pressed. The keyboard shortcut handler SHALL be registered via `cx.observe_keystrokes()`.

#### Scenario: Ctrl+1 switches to Bluetooth tab

- **WHEN** the user presses Ctrl+1
- **THEN** the Bluetooth Devices tab (index 0) SHALL become active

#### Scenario: Ctrl+4 switches to Configuration tab

- **WHEN** the user presses Ctrl+4
- **THEN** the Configuration tab (index 3) SHALL become active

#### Scenario: Ctrl+6 switches to Settings tab

- **WHEN** the user presses Ctrl+6
- **THEN** the Settings tab (index 5) SHALL become active
