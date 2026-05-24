## MODIFIED Requirements

### Requirement: Volume level is displayed and adjustable per device

The system SHALL render a volume control for each audio device (sink or source) consisting of a horizontal Slider Entity showing the current volume level as a filled portion and a text field for precise numeric volume entry that also serves as the volume display. Clicking on the slider or dragging it SHALL set the volume to the corresponding percentage. Typing a number into the text field SHALL set the volume to that percentage. Both controls SHALL stay in sync with each other.

#### Scenario: Volume bar shows current level

- **WHEN** a device has volume set to 75%
- **THEN** the volume slider SHALL show the left 75% of the track as filled
- **THEN** the text field SHALL display "75"

#### Scenario: Clicking volume bar sets new volume

- **WHEN** the user clicks on the volume slider at a position corresponding to 50%
- **THEN** the system SHALL call the appropriate PulseAudio volume set operation for that device
- **THEN** the volume slider SHALL update to show 50% filled
- **THEN** the text field SHALL update to show "50"

#### Scenario: Dragging volume bar sets new volume

- **WHEN** the user clicks and holds on the volume slider and drags horizontally
- **THEN** the system SHALL continuously call the appropriate PulseAudio volume set operation as the slider value changes
- **THEN** the volume slider fill SHALL track the cursor position during the drag
- **THEN** the text field SHALL update to reflect the current volume

#### Scenario: Drag continues beyond slider bounds

- **WHEN** the user drags the cursor beyond the left or right edges of the volume slider
- **THEN** the volume SHALL be clamped to 0% or 100% respectively
- **THEN** the slider SHALL continue to respond when the cursor returns to within bounds

#### Scenario: Typing a value in the text field sets new volume

- **WHEN** the user types "80" into the volume text field and presses Enter or moves focus away
- **THEN** the system SHALL call the appropriate PulseAudio volume set operation for that device at 80%
- **THEN** the volume slider SHALL update to show 80% filled

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
- **THEN** the volume slider SHALL show no filled portion

#### Scenario: Volume bar at 100%

- **WHEN** a device's volume is set to 100%
- **THEN** the volume slider SHALL be completely filled
