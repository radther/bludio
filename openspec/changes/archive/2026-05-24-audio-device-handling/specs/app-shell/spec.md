## ADDED Requirements

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
