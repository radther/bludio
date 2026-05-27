## MODIFIED Requirements

### Requirement: Left-mounted tab bar renders icon-only vertical tabs

The system SHALL render a `TabBar` entity as a fixed-width vertical tab bar on the left side of the application window. Each tab SHALL display only an SVG icon with no text label. The tab bar SHALL have a fixed width of 48 pixels and SHALL span the full height of the window. The tab bar SHALL accept a vector of `Tab` structs (icon function + tooltip string) and an initial active index.

#### Scenario: Tab bar is visible on app launch

- **WHEN** the application starts
- **THEN** a vertical `TabBar` entity SHALL appear on the left side of the window
- **THEN** the bar SHALL contain five icon tabs: Bluetooth, Output Devices, Input Devices, Configuration, and Text Field Test
- **THEN** the bar SHALL be 48 pixels wide

#### Scenario: Tab bar accepts Tab definitions as input

- **WHEN** the `TabBar` is created
- **THEN** it SHALL accept a `Vec<Tab>` where each `Tab` has an icon function (`fn() -> Svg`) and a tooltip string
- **THEN** the `TabBar` SHALL not hard-code page names or domain-specific logic

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

### Requirement: Five tabs are configured

The system SHALL configure five tabs at indices 0–4: Bluetooth Devices (index 0, icon `radar`), Output Devices (index 1, icon `speaker`), Input Devices (index 2, icon `mic`), Configuration (index 3, icon `form`), and Text Field Test (index 4, icon `test-tube-diagonal`). Each tab SHALL have a tooltip matching its page name.

#### Scenario: Five tabs are present

- **WHEN** the application starts
- **THEN** tab 0 SHALL use `icons::bluetooth` (radar.svg) with tooltip "Bluetooth Devices"
- **THEN** tab 1 SHALL use `icons::audio_output` (speaker.svg) with tooltip "Output Devices"
- **THEN** tab 2 SHALL use `icons::audio_input` (mic.svg) with tooltip "Input Devices"
- **THEN** tab 3 SHALL use `icons::audio_card` (form.svg) with tooltip "Configuration"
- **THEN** tab 4 SHALL use `icons::text_field_test` (test-tube-diagonal.svg) with tooltip "Text Field Test"
- **THEN** tab 0 (Bluetooth Devices) SHALL be the active tab on startup

### Requirement: Tab bar supports keyboard shortcuts

The system SHALL observe keystrokes globally and switch tabs when Ctrl+1 through Ctrl+5 are pressed. The keyboard shortcut handler SHALL be registered via `cx.observe_keystrokes()`.

#### Scenario: Ctrl+1 switches to Bluetooth tab

- **WHEN** the user presses Ctrl+1
- **THEN** the Bluetooth Devices tab (index 0) SHALL become active

#### Scenario: Ctrl+4 switches to Configuration tab

- **WHEN** the user presses Ctrl+4
- **THEN** the Configuration tab (index 3) SHALL become active
