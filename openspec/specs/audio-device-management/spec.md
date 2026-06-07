# audio-device-management

## Purpose

Display and manage PulseAudio audio devices (sinks and sources), including device properties, card profiles, and default device selection.

## Requirements

### Requirement: Output devices are listed with properties

The system SHALL display all PulseAudio output devices (sinks) in a scrollable list within the "Output Devices" page. Each device row SHALL show the device name, a volume slider with inline text field for numeric entry, mute state, a default button, and a profile dropdown for output devices with an associated card. When the system is in selection mode, each real sink row SHALL display a checkbox on the right side for toggling selection. Combined sink rows SHALL display a special icon between the name and the "Default" label, and a delete button in addition to the default button. The default sink SHALL be visually distinguished by a status strip colored with the audio accent and a "Default" text badge. When the Audio subsystem is disconnected or reconnecting, the page SHALL display the corresponding status state instead of the device list.

#### Scenario: Output devices page shows all sinks

- **WHEN** the user navigates to the "Output Devices" tab
- **THEN** a scrollable list of all available PulseAudio sinks SHALL be displayed
- **THEN** each row SHALL show the sink's description name
- **THEN** each row SHALL show a horizontal slider (filled portion proportional to volume) and a text field displaying the volume as a whole number (e.g., "75")
- **THEN** each row SHALL have a mute/unmute action button
- **THEN** each real sink row SHALL have a "Default" action button
- **THEN** each combined sink row SHALL have a delete action button (danger-colored) in addition to a "Default" button
- **THEN** each combined sink row SHALL display the merge icon between the name and any label
- **THEN** each row with an associated card SHALL show a profile dropdown listing available profiles
- **THEN** the default sink SHALL display a status strip in the audio accent color and a "Default" text badge

#### Scenario: Selection mode is active

- **WHEN** the user presses the combine button on the Output Devices page header
- **THEN** the system SHALL enter selection mode
- **THEN** each real sink row SHALL display a checkbox (using the custom checkbox component) on the right side
- **THEN** the checkbox SHALL be unchecked by default
- **THEN** combined sink rows SHALL NOT display a checkbox (or display a disabled checkbox)
- **THEN** the "Default" button on real sinks SHALL be hidden while in selection mode

#### Scenario: Selection mode is exited after creating a combined sink

- **WHEN** the user presses the combine button again while in selection mode
- **WHEN** at least one sink is selected
- **THEN** the system SHALL exit selection mode
- **THEN** the checkboxes SHALL disappear
- **THEN** the "Default" button SHALL reappear on real sinks
- **THEN** the new combined sink row SHALL appear with a special icon and delete button

#### Scenario: Empty state when no sinks available

- **WHEN** PulseAudio reports zero sinks
- **THEN** the output devices page SHALL display a "No output devices found" message

#### Scenario: Connection state while PulseAudio is initializing

- **WHEN** PulseAudio is not yet connected
- **THEN** the Audio subsystem status SHALL be `Connecting`
- **THEN** the output devices page SHALL display a "Connecting to PulseAudio..." message in the content area via a `match` on `SubsystemStatus::Connecting`
- **THEN** the device list SHALL not be displayed

#### Scenario: PulseAudio disconnected after runtime failure

- **WHEN** the Audio subsystem status transitions to `Disconnected` after initial connection
- **THEN** the output devices page SHALL display the error message
- **THEN** the device list, volume sliders, and action buttons SHALL NOT be rendered

#### Scenario: PulseAudio reconnecting

- **WHEN** the Audio subsystem status is `Reconnecting`
- **THEN** the output devices page SHALL display "Reconnecting to PulseAudio..."
- **THEN** the device list SHALL NOT be rendered

#### Scenario: PulseAudio reconnected successfully

- **WHEN** the Audio subsystem status transitions back to `Connected`
- **THEN** the output devices page SHALL resume displaying the device list with fresh data
- **THEN** volume sliders and action buttons SHALL become interactive

### Requirement: Input devices are listed with properties

The system SHALL display all PulseAudio input devices (sources, excluding monitor sources) in a scrollable list within the "Input Devices" page. Each device row SHALL show the device name, a volume slider with inline text field, mute state, and a "Default" button. Input device rows SHALL NOT show a profile dropdown. When the Audio subsystem is disconnected or reconnecting, the page SHALL display the corresponding status state instead of the device list.

#### Scenario: Input devices page shows all sources

- **WHEN** the user navigates to the "Input Devices" tab
- **THEN** a scrollable list of all hardware PulseAudio sources SHALL be displayed
- **THEN** each row SHALL show the source's description name
- **THEN** each row SHALL show a horizontal slider and text field for volume
- **THEN** each row SHALL have a mute/unmute action button
- **THEN** each row SHALL have a "Default" action button
- **THEN** the default source SHALL display a status strip in the audio accent color and a "Default" text badge

#### Scenario: Monitor sources are excluded from input devices list

- **WHEN** PulseAudio has monitor sources (virtual sources that record sink output)
- **THEN** monitor sources SHALL NOT appear in the input devices list

#### Scenario: Empty state when no sources available

- **WHEN** PulseAudio reports zero hardware sources
- **THEN** the input devices page SHALL display a "No input devices found" message

#### Scenario: PulseAudio disconnected after runtime failure

- **WHEN** the Audio subsystem status transitions to `Disconnected` after initial connection
- **THEN** the input devices page SHALL display the error message
- **THEN** the device list, volume sliders, and action buttons SHALL NOT be rendered

#### Scenario: PulseAudio reconnecting

- **WHEN** the Audio subsystem status is `Reconnecting`
- **THEN** the input devices page SHALL display "Reconnecting to PulseAudio..."
- **THEN** the device list SHALL NOT be rendered

#### Scenario: PulseAudio reconnected successfully

- **WHEN** the Audio subsystem status transitions back to `Connected`
- **THEN** the input devices page SHALL resume displaying the device list with fresh data

### Requirement: Card profiles are displayed and switchable via dropdown for output devices

The system SHALL display the active card profile for each output device row (when the sink has an associated card) and SHALL allow the user to switch between available profiles via a dropdown selector. Selecting a profile SHALL send `AudioCommand::SetCardProfile` to the PulseAudio backend thread and trigger a PA wakeup.

#### Scenario: Active profile shown on output device row

- **WHEN** an output device has an associated card with available profiles
- **THEN** a dropdown selector SHALL be displayed on the device row showing the active profile name

#### Scenario: User selects a different profile from dropdown

- **WHEN** the user selects a different profile from the dropdown
- **THEN** the system SHALL send `AudioCommand::SetCardProfile` with the card index and selected profile name
- **THEN** the PA wakeup SHALL be triggered
- **THEN** the dropdown SHALL display the newly selected profile after state refresh

### Requirement: Default output device can be set

The system SHALL allow the user to set any real sink or combined sink as the system default output device. The operation SHALL use PulseAudio's `set_default_sink` operation and the UI SHALL reflect the change. Combined sink rows SHALL display a "Default" button (unless in selection mode, in which case they display no action button).

#### Scenario: Setting a real sink as default

- **WHEN** the user activates the "set default" control on a real sink row
- **THEN** the system SHALL call `set_default_sink` with that sink's name
- **THEN** that sink SHALL become marked as the default
- **THEN** the previously default sink SHALL lose its default indicator

#### Scenario: Setting a combined sink as default

- **WHEN** the user activates the "set default" control on a combined sink row
- **THEN** the system SHALL call `set_default_sink` with that combined sink's name
- **THEN** that combined sink SHALL become marked as the default
- **THEN** the previously default sink SHALL lose its default indicator
- **THEN** audio SHALL be routed to all slave sinks of the combined sink

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

The system SHALL subscribe to PulseAudio events (sink, source, card, server) and SHALL refresh the displayed device list when any device property changes externally (e.g., volume changed via pactl, new device plugged in, profile switched by another app). When the Audio subsystem disconnects, event processing SHALL halt and resume upon reconnection with a fresh event subscription.

#### Scenario: External volume change is reflected

- **WHEN** a sink's volume is changed externally (e.g., via `pactl set-sink-volume`)
- **THEN** the output devices page SHALL update the slider position and text field for that sink within a reasonable time
- **THEN** no user interaction SHALL be required to see the update

#### Scenario: New device appears after plugging in

- **WHEN** a new audio device is plugged in and registered with PulseAudio
- **THEN** the appropriate device list (outputs or inputs) SHALL show the new device

#### Scenario: PA event subscription halts on disconnect

- **WHEN** the PulseAudio context enters a failed or terminated state
- **THEN** the event subscription callback SHALL stop processing events
- **THEN** the PA thread SHALL exit cleanly

#### Scenario: PA event subscription resumes on reconnect

- **WHEN** a new PA thread is created during reconnection
- **THEN** a fresh event subscription SHALL be registered for Sink, Source, Card, and Server events
- **THEN** external changes SHALL be detected and reflected in the UI as before
