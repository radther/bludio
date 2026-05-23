## ADDED Requirements

### Requirement: Volume level is displayed and adjustable per device

The system SHALL render a volume control for each audio device (sink or source) consisting of a horizontal bar showing the current volume level as a filled portion and a text field for precise numeric volume entry that also serves as the volume display. Clicking on the bar SHALL set the volume to the corresponding percentage. Typing a number into the text field SHALL set the volume to that percentage. Both controls SHALL stay in sync with each other.

#### Scenario: Volume bar shows current level

- **WHEN** a device has volume set to 75%
- **THEN** the volume bar SHALL show the left 75% of the bar as filled
- **THEN** the text field SHALL display "75"

#### Scenario: Clicking volume bar sets new volume

- **WHEN** the user clicks on the volume bar at a position corresponding to 50%
- **THEN** the system SHALL call the appropriate PulseAudio volume set operation for that device
- **THEN** the volume bar SHALL update to show 50% filled
- **THEN** the text field SHALL update to show "50"

#### Scenario: Typing a value in the text field sets new volume

- **WHEN** the user types "80" into the volume text field and presses Enter or moves focus away
- **THEN** the system SHALL call the appropriate PulseAudio volume set operation for that device at 80%
- **THEN** the volume bar SHALL update to show 80% filled

#### Scenario: Text field rejects out-of-range values

- **WHEN** the user types a value outside 0-100 (e.g., "150" or "-10") into the volume text field
- **THEN** the value SHALL be clamped to the valid range (0 or 100) before sending to PulseAudio
- **THEN** the text field SHALL display the clamped value

#### Scenario: Text field rejects non-numeric input

- **WHEN** the user types non-numeric characters into the volume text field
- **THEN** the input SHALL be ignored or the field SHALL revert to the previous valid value on Enter/blur

#### Scenario: Enter on empty buffer reverts to previous value

- **WHEN** the user backspaces to an empty buffer and presses Enter
- **THEN** the field SHALL revert to the previous volume value (same as Escape)
- **THEN** no volume command SHALL be sent

#### Scenario: Volume bar at 0%

- **WHEN** a device's volume is set to 0% (silent, not muted)
- **THEN** the volume bar SHALL show no filled portion

#### Scenario: Volume bar at 100%

- **WHEN** a device's volume is set to 100%
- **THEN** the volume bar SHALL be completely filled

### Requirement: Mute can be toggled per device

The system SHALL provide a mute toggle button for each audio device. Activating the toggle SHALL mute or unmute the device via PulseAudio's mute operation. The mute state SHALL be visually indicated on the device row.

#### Scenario: Muting an unmuted device

- **WHEN** the user clicks the mute button on an unmuted device
- **THEN** the system SHALL call the PulseAudio mute operation with `mute=true`
- **THEN** the device row SHALL visually indicate muted state

#### Scenario: Unmuting a muted device

- **WHEN** the user clicks the mute button on a muted device
- **THEN** the system SHALL call the PulseAudio mute operation with `mute=false`
- **THEN** the device row SHALL no longer indicate muted state

#### Scenario: Mute state responds to external changes

- **WHEN** a device is muted or unmuted externally (e.g., via `pactl set-sink-mute`)
- **THEN** the mute toggle SHALL update to reflect the new state

### Requirement: Volume control is compatible with both output and input devices

The volume bar and mute toggle SHALL work identically for both output devices (sinks) and input devices (sources), using the appropriate PulseAudio operations for each device type.

#### Scenario: Volume bar works on input device

- **WHEN** the user adjusts volume on an input device (source)
- **THEN** the system SHALL call the source-specific volume set operation
- **THEN** the input device's volume SHALL be updated

#### Scenario: Mute toggle works on input device

- **WHEN** the user toggles mute on an input device (source)
- **THEN** the system SHALL call the source-specific mute operation
