## MODIFIED Requirements

### Requirement: Window uses reasonable default dimensions

The system SHALL open the window with dimensions of 1100 pixels wide by 700 pixels tall, centered on the primary display.

#### Scenario: Window opens at specified size

- **WHEN** the application starts
- **THEN** the window is 1100×700 pixels
- **THEN** the window is centered on the display
- **THEN** the window has the app identity string `"dev.toomosin.bludio"`

### Requirement: App shell renders a horizontal two-column layout

The system SHALL render the application window as a horizontal flex row comprising a fixed-width left tab bar and a flexible right content area. The left tab bar SHALL span the full height of the window. The right content area SHALL fill the remaining horizontal space. The root element SHALL set the font family from the active theme.

#### Scenario: Layout is two-column horizontal split

- **WHEN** the application window is open
- **THEN** the left column SHALL be a tab bar entity (`TabBar`) with fixed width of 48 pixels
- **THEN** the right column SHALL fill the remaining window width
- **THEN** both columns SHALL span the full window height
- **THEN** the root element SHALL apply `font_family` from the active theme

#### Scenario: Content area renders the active page

- **WHEN** a tab is selected
- **THEN** the right content area SHALL render the page entity corresponding to the active tab
- **THEN** switching tabs SHALL instantly swap the content area contents
- **THEN** the active page entity SHALL be one of: `BluetoothPage`, `AudioPage` (output), `AudioPage` (input), `ConfigurationPage`, or `DevTestPage`

### Requirement: Tab bar supports five pages

The system SHALL provide five tabs in the left tab bar: Bluetooth Devices, Output Devices, Input Devices, Configuration, and Text Field Test. Each tab SHALL display an icon and a tooltip. The Bluetooth Devices tab SHALL be active on startup.

#### Scenario: Five tabs are present

- **WHEN** the application starts
- **THEN** the tab bar SHALL contain five tabs at indices 0–4
- **THEN** tab 0 SHALL be Bluetooth Devices (icon: `radar`, tooltip: "Bluetooth Devices")
- **THEN** tab 1 SHALL be Output Devices (icon: `speaker`, tooltip: "Output Devices")
- **THEN** tab 2 SHALL be Input Devices (icon: `mic`, tooltip: "Input Devices")
- **THEN** tab 3 SHALL be Configuration (icon: `form`, tooltip: "Configuration")
- **THEN** tab 4 SHALL be Text Field Test (icon: `test-tube-diagonal`, tooltip: "Text Field Test")
- **THEN** tab 0 (Bluetooth Devices) SHALL be the active tab on startup

#### Scenario: Keyboard shortcuts switch tabs

- **WHEN** the user presses Ctrl+1 through Ctrl+5
- **THEN** the corresponding tab (index 0–4) SHALL become active
- **THEN** the content area SHALL render the page for that tab

### Requirement: Content area renders audio output devices page

The system SHALL render the `AudioPage` entity (configured for `DeviceKind::Output`) in the content area when the Output Devices tab is active. The page SHALL display a list of PulseAudio sinks with per-device controls.

#### Scenario: Output devices page is rendered when tab is active

- **WHEN** the Output Devices tab is the active tab
- **THEN** the right content area SHALL display the `AudioPage` entity for output devices
- **THEN** the page SHALL show a scrollable list of output device rows
- **THEN** each device row SHALL include a name, volume slider, text field, mute toggle, default button, and status strip

### Requirement: Content area renders audio input devices page

The system SHALL render the `AudioPage` entity (configured for `DeviceKind::Input`) in the content area when the Input Devices tab is active. The page SHALL display a list of PulseAudio sources with per-device controls.

#### Scenario: Input devices page is rendered when tab is active

- **WHEN** the Input Devices tab is the active tab
- **THEN** the right content area SHALL display the `AudioPage` entity for input devices
- **THEN** the page SHALL show a scrollable list of input device rows
- **THEN** each device row SHALL include a name, volume slider, text field, mute toggle, default button, and status strip

### Requirement: Content area renders configuration page

The system SHALL render the `ConfigurationPage` entity in the content area when the Configuration tab is active. The page SHALL display a list of PulseAudio hardware cards with profile dropdowns.

#### Scenario: Configuration page is rendered when tab is active

- **WHEN** the Configuration tab is the active tab
- **THEN** the right content area SHALL display the `ConfigurationPage` entity
- **THEN** the page SHALL show a scrollable list of card rows
- **THEN** each card row SHALL include a display name and a profile dropdown

## ADDED Requirements

### Requirement: App applies theme colors and font at root level

The system SHALL apply the active theme's background color, text color, and font family to the root layout element so all child elements inherit these values.

#### Scenario: Root element uses theme styling

- **WHEN** the application renders
- **THEN** the root `h_flex` SHALL set `.bg(colors.background)`, `.text_color(colors.text)`, and `.font_family(theme.font_family)`
- **THEN** child elements SHALL inherit the font family via GPUI's cascade
