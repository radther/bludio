# audio-device-management

## Purpose

Display and manage PulseAudio audio devices (sinks and sources), including device properties, card profiles, and default device selection.

## Requirements

### Requirement: Output devices are listed with properties

The system SHALL display all PulseAudio output devices (sinks) in a scrollable list within the "Output Devices" page. Each device row SHALL show the device name, current volume level as a percentage, mute state, active profile name, and an indicator if the device is the system default sink.

#### Scenario: Output devices page shows all sinks

- **WHEN** the user navigates to the "Output Devices" tab
- **THEN** a scrollable list of all available PulseAudio sinks SHALL be displayed
- **THEN** each row SHALL show the sink's description name
- **THEN** each row SHALL show the current volume as a percentage (e.g., "75%")
- **THEN** each row SHALL indicate if the sink is muted
- **THEN** the default sink SHALL be visually distinguished from other sinks

#### Scenario: Empty state when no sinks available

- **WHEN** PulseAudio reports zero sinks
- **THEN** the output devices page SHALL display a message indicating no output devices found

### Requirement: Input devices are listed with properties

The system SHALL display all PulseAudio input devices (sources, excluding monitor sources) in a scrollable list within the "Input Devices" page. Each device row SHALL show the device name, current volume level as a percentage, mute state, and an indicator if the device is the system default source.

#### Scenario: Input devices page shows all sources

- **WHEN** the user navigates to the "Input Devices" tab
- **THEN** a scrollable list of all hardware PulseAudio sources SHALL be displayed
- **THEN** each row SHALL show the source's description name
- **THEN** each row SHALL show the current volume as a percentage
- **THEN** each row SHALL indicate if the source is muted
- **THEN** the default source SHALL be visually distinguished from other sources

#### Scenario: Monitor sources are excluded from input devices list

- **WHEN** PulseAudio has monitor sources (virtual sources that record sink output)
- **THEN** monitor sources SHALL NOT appear in the input devices list

#### Scenario: Empty state when no sources available

- **WHEN** PulseAudio reports zero hardware sources
- **THEN** the input devices page SHALL display a message indicating no input devices found

### Requirement: Card profiles are displayed and switchable via dropdown for output devices

The system SHALL display the active card profile for each output device and SHALL allow the user to switch between available profiles via a dropdown selector control. The dropdown SHALL show the currently active profile when collapsed and SHALL list all available profiles when expanded (e.g., "A2DP Sink", "Headset Head Unit", "Off"). Selecting a profile SHALL use PulseAudio's `set_card_profile_by_index` operation.

#### Scenario: Active profile shown on output device row

- **WHEN** an output device has an associated card with an active profile
- **THEN** a dropdown selector SHALL be displayed on the device row showing the active profile name

#### Scenario: User opens profile dropdown

- **WHEN** the user clicks the profile dropdown on an output device row
- **THEN** the dropdown SHALL expand to show all available profiles for that card
- **THEN** the currently active profile SHALL be visually indicated in the expanded list

#### Scenario: User selects a different profile from dropdown

- **WHEN** the user selects a different profile from the expanded dropdown
- **THEN** the system SHALL call `set_card_profile_by_index` with the selected profile name
- **THEN** the dropdown SHALL collapse and display the newly selected profile
- **THEN** the device list SHALL update to reflect the new active profile

### Requirement: Default output device can be set

The system SHALL allow the user to set any sink as the system default output device. The operation SHALL use PulseAudio's `set_default_sink` operation and the UI SHALL reflect the change.

#### Scenario: Setting a sink as default

- **WHEN** the user activates the "set default" control on a non-default sink
- **THEN** the system SHALL call `set_default_sink` with that sink's name
- **THEN** that sink SHALL become marked as the default
- **THEN** the previously default sink SHALL lose its default indicator

#### Scenario: Default sink is visually indicated

- **WHEN** a sink is the system default output device
- **THEN** its row in the output devices list SHALL display a default indicator (e.g., "● Default" badge)

### Requirement: Default input device can be set

The system SHALL allow the user to set any source as the system default input device. The operation SHALL use PulseAudio's `set_default_source` operation and the UI SHALL reflect the change.

#### Scenario: Setting a source as default

- **WHEN** the user activates the "set default" control on a non-default source
- **THEN** the system SHALL call `set_default_source` with that source's name
- **THEN** that source SHALL become marked as the default

### Requirement: Device list reacts to external changes

The system SHALL subscribe to PulseAudio events (sink, source, card, server) and SHALL refresh the displayed device list when any device property changes externally (e.g., volume changed via pactl, new device plugged in, profile switched by another app).

#### Scenario: External volume change is reflected

- **WHEN** a sink's volume is changed externally (e.g., via `pactl set-sink-volume`)
- **THEN** the output devices page SHALL update the displayed volume for that sink within a reasonable time
- **THEN** no user interaction SHALL be required to see the update

#### Scenario: New device appears after plugging in

- **WHEN** a new audio device is plugged in and registered with PulseAudio
- **THEN** the appropriate device list (outputs or inputs) SHALL show the new device
