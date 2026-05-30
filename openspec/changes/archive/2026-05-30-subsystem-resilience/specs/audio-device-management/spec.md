## MODIFIED Requirements

### Requirement: Output devices are listed with properties

The system SHALL display all PulseAudio output devices (sinks) in a scrollable list within the "Output Devices" page. Each device row SHALL show the device name, a volume slider with inline text field for numeric entry, mute state, a "Default" button, and a profile dropdown for output devices with an associated card. The default sink SHALL be visually distinguished by a status strip colored with the audio accent and a "Default" text badge. When the Audio subsystem is disconnected or reconnecting, the page SHALL display the corresponding status state instead of the device list.

#### Scenario: Output devices page shows all sinks

- **WHEN** the user navigates to the "Output Devices" tab and the Audio subsystem is connected
- **THEN** a scrollable list of all available PulseAudio sinks SHALL be displayed
- **THEN** each row SHALL show the sink's description name
- **THEN** each row SHALL show a horizontal slider (filled portion proportional to volume) and a text field displaying the volume as a whole number (e.g., "75")
- **THEN** each row SHALL have a mute/unmute action button
- **THEN** each row SHALL have a "Default" action button
- **THEN** each row with an associated card SHALL show a profile dropdown listing available profiles
- **THEN** the default sink SHALL display a status strip in the audio accent color and a "Default" text badge

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

- **WHEN** the user navigates to the "Input Devices" tab and the Audio subsystem is connected
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
