## MODIFIED Requirements

### Requirement: Volume level is displayed and adjustable per device

The system SHALL render a volume control for each audio device (sink or source) consisting of a `SliderState` Entity displaying the current volume as a filled horizontal track and a `TextField` Entity for precise numeric volume entry. Both controls SHALL stay in sync with each other. The slider fill color SHALL use the audio accent color (or muted color when the device is muted).

#### Scenario: Volume bar shows current level

- **WHEN** a device has volume set to 75%
- **THEN** the slider SHALL show 75% of the track as filled
- **THEN** the text field SHALL display "75"

#### Scenario: Clicking volume bar sets new volume

- **WHEN** the user clicks on the slider at a position corresponding to 50%
- **THEN** the system SHALL send `AudioCommand::SetVolume` with the device kind, index, and 0.5
- **THEN** the PA wakeup SHALL be triggered
- **THEN** the slider SHALL update to show 50% filled
- **THEN** the text field SHALL update to show "50"

#### Scenario: Dragging volume bar sets new volume

- **WHEN** the user presses and holds left mouse button on the slider and drags horizontally
- **THEN** the slider SHALL emit `SliderEvent::Change` events continuously as the value changes
- **THEN** each `SliderEvent::Change` SHALL send `AudioCommand::SetVolume` to the PA backend
- **THEN** the slider fill SHALL track the cursor position during the drag
- **THEN** the text field SHALL update to reflect the current volume

#### Scenario: Drag continues beyond slider bounds

- **WHEN** the user drags the cursor beyond the left or right edges of the slider
- **THEN** the volume SHALL be clamped to 0% or 100% respectively
- **THEN** the slider SHALL continue to respond when the cursor returns to within bounds

#### Scenario: Typing a value in the text field sets new volume

- **WHEN** the user types "80" into the volume text field and presses Enter
- **THEN** the system SHALL send `AudioCommand::SetVolume` with 0.8 as the volume
- **THEN** the slider SHALL update to show 80% filled

#### Scenario: Text field rejects out-of-range values

- **WHEN** the user types a value outside 0–100 (e.g., "150" or "-10") into the text field
- **THEN** the value SHALL be clamped to the valid range (0 or 100) before sending to PulseAudio
- **THEN** the text field SHALL display the clamped value

#### Scenario: Text field rejects non-numeric input

- **WHEN** the user types non-numeric characters into the text field
- **THEN** the input SHALL be filtered by the TextField's `filter_char` configuration (digits only)

#### Scenario: Enter on empty buffer reverts to previous value

- **WHEN** the user backspaces to an empty buffer and presses Enter
- **THEN** the field SHALL revert to the current volume value (same as Escape)
- **THEN** no volume command SHALL be sent

#### Scenario: Text field reverts on blur

- **WHEN** the text field loses focus without the user pressing Enter
- **THEN** the text field SHALL revert to the current actual volume value
- **THEN** no volume command SHALL be sent

#### Scenario: Volume bar at 0%

- **WHEN** a device's volume is set to 0% (silent, not muted)
- **THEN** the slider SHALL show no filled portion

#### Scenario: Volume bar at 100%

- **WHEN** a device's volume is set to 100%
- **THEN** the slider SHALL be completely filled

### Requirement: Mute can be toggled per device

The system SHALL provide a mute toggle button for each audio device. Activating the toggle SHALL send `AudioCommand::SetMute` with the device kind, index, and mute state to the PulseAudio backend. The mute state SHALL be visually indicated on the device row.

#### Scenario: Muting an unmuted device

- **WHEN** the user clicks the "Mute" button on an unmuted device
- **THEN** the system SHALL send `AudioCommand::SetMute` with `mute=true`
- **THEN** the PA wakeup SHALL be triggered
- **THEN** the button label SHALL change to "Unmute" with danger colors

#### Scenario: Unmuting a muted device

- **WHEN** the user clicks the "Unmute" button on a muted device
- **THEN** the system SHALL send `AudioCommand::SetMute` with `mute=false`
- **THEN** the button label SHALL change to "Mute" with default element colors

#### Scenario: Mute state responds to external changes

- **WHEN** a device is muted or unmuted externally (e.g., via `pactl set-sink-mute`)
- **THEN** the mute button label and colors SHALL update to reflect the new state

### Requirement: Slider communicates via EventEmitter

The `SliderState` Entity SHALL implement `EventEmitter<SliderEvent>` and emit `SliderEvent::Change(f64)` on every value update and `SliderEvent::Release(f64)` when the user releases the mouse button. The `Slider` element is a `RenderOnce` wrapper that reads from the `SliderState` entity and renders the visual track.

#### Scenario: Parent subscribes to slider events

- **WHEN** a parent view subscribes to the `SliderState` entity via `cx.subscribe(&slider, ...)`
- **THEN** the parent SHALL receive `SliderEvent::Change` events as the user drags or clicks
- **THEN** the parent SHALL receive `SliderEvent::Release` events when the user releases the mouse

#### Scenario: Slider captures bounds via canvas

- **WHEN** the slider renders
- **THEN** a `canvas` element SHALL capture the slider's pixel bounds
- **THEN** the bounds SHALL be stored in `SliderState` for mapping mouse coordinates to values
