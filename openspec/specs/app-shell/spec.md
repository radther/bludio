# app-shell

## Purpose

The minimal GPUI application shell that opens a window and renders the app identity. This is the visual foundation all future features compose into.

## Requirements

### Requirement: Application launches and opens a window

The system SHALL compile as a Rust binary that, when executed, opens a native OS window using the GPUI rendering pipeline.

#### Scenario: Binary compiles successfully

- **WHEN** `cargo build` is run in the project root
- **THEN** the build completes without errors

#### Scenario: Application starts and creates a window

- **WHEN** the compiled binary is executed
- **THEN** a native OS window opens with no terminal output errors

### Requirement: Window uses reasonable default dimensions

The system SHALL open the window with dimensions of 1100 pixels wide by 700 pixels tall, centered on the primary display.

#### Scenario: Window opens at specified size

- **WHEN** the application starts
- **THEN** the window is 1100×700 pixels
- **THEN** the window is centered on the display
- **THEN** the window has an app identity string (e.g., `"com.bludio.app"`)

### Requirement: App shell renders a horizontal two-column layout

The system SHALL render the application window as a horizontal flex row comprising a fixed-width left tab bar and a flexible right content area. The left tab bar SHALL span the full height of the window. The right content area SHALL fill the remaining horizontal space.

#### Scenario: Layout is two-column horizontal split

- **WHEN** the application window is open
- **THEN** the left column SHALL be a tab bar with fixed width of 48 pixels
- **THEN** the right column SHALL fill the remaining window width
- **THEN** both columns SHALL span the full window height

#### Scenario: Content area renders the active page

- **WHEN** a tab is selected
- **THEN** the right content area SHALL render the page corresponding to the active tab
- **THEN** switching tabs SHALL instantly swap the content area contents

### Requirement: Content area renders audio output devices page

The system SHALL render the audio output devices page in the content area when the Output Devices tab is active. The page SHALL display a list of PulseAudio sinks with per-device controls following the same layout conventions as the Bluetooth devices page.

#### Scenario: Output devices page is rendered when tab is active

- **WHEN** the Output Devices tab is the active tab
- **THEN** the right content area SHALL display a scrollable list of output devices
- **THEN** each device row SHALL include a name, volume bar, mute toggle, and default indicator

### Requirement: Content area renders audio input devices page

The system SHALL render the audio input devices page in the content area when the Input Devices tab is active. The page SHALL display a list of PulseAudio sources with per-device controls following the same layout conventions as the Bluetooth devices page.

#### Scenario: Input devices page is rendered when tab is active

- **WHEN** the Input Devices tab is the active tab
- **THEN** the right content area SHALL display a scrollable list of input devices
- **THEN** each device row SHALL include a name, volume bar, mute toggle, and default indicator
