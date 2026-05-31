## ADDED Requirements

### Requirement: Settings tab is present
The system SHALL include a Settings tab at index 5 in the tab bar. The tab SHALL use the `bolt` icon and the tooltip "Settings".

#### Scenario: Settings tab is visible
- **WHEN** the application starts
- **THEN** tab 5 SHALL use `icons::bolt` with tooltip "Settings"

## MODIFIED Requirements

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

### Requirement: Tab bar supports keyboard shortcuts
The system SHALL observe keystrokes globally and switch tabs when Ctrl+1 through Ctrl+6 are pressed. The keyboard shortcut handler SHALL be registered via `cx.observe_keystrokes()`.

#### Scenario: Ctrl+1 switches to Bluetooth tab
- **WHEN** the user presses Ctrl+1
- **THEN** the Bluetooth Devices tab (index 0) SHALL become active

#### Scenario: Ctrl+6 switches to Settings tab
- **WHEN** the user presses Ctrl+6
- **THEN** the Settings tab (index 5) SHALL become active
