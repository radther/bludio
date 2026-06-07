## MODIFIED Requirements

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
