## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Five tabs are configured

The system SHALL configure five tabs at indices 0–4: Bluetooth Devices (index 0, icon `radar`), Output Devices (index 1, icon `speaker`), Input Devices (index 2, icon `mic`), Configuration (index 3, icon `form`), and Text Field Test (index 4, icon `test-tube-diagonal`). Each tab SHALL have a tooltip matching its page name. The tab bar SHALL also contain one action item: Restart Bluetooth (icon `bluetooth`, tooltip "Restart Bluetooth service", action ID `"restart-bluetooth"`), anchored to the bottom.

#### Scenario: Five tabs and one action are present

- **WHEN** the application starts
- **THEN** tab 0 SHALL use `icons::bluetooth` (radar.svg) with tooltip "Bluetooth Devices"
- **THEN** tab 1 SHALL use `icons::audio_output` (speaker.svg) with tooltip "Output Devices"
- **THEN** tab 2 SHALL use `icons::audio_input` (mic.svg) with tooltip "Input Devices"
- **THEN** tab 3 SHALL use `icons::audio_card` (form.svg) with tooltip "Configuration"
- **THEN** tab 4 SHALL use `icons::text_field_test` (test-tube-diagonal.svg) with tooltip "Text Field Test"
- **THEN** tab 0 (Bluetooth Devices) SHALL be the active tab on startup
- **THEN** one action item SHALL render at the bottom with icon `icons::bluetooth` and tooltip "Restart Bluetooth service"
