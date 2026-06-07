# combined-sink-output

## Purpose

Provide combined sink creation and management for PulseAudio output devices, allowing users to merge multiple sinks into a single virtual output device.

## Requirements

### Requirement: Combined sink can be created from the Output Devices page via selection mode

The system SHALL provide a combine button on the Output Devices page header. When the user presses the button, the system SHALL enter selection mode. In selection mode, each real sink row SHALL display a checkbox on the right side. When the user presses the combine button again, the system SHALL load `module-combine-sink` with a unique auto-generated name (e.g., "Bludio-combined-1") and the selected sinks as the `slaves=` argument. The combine button SHALL be displayed in the audio accent color when selection mode is active.

#### Scenario: User enters selection mode

- **WHEN** the user presses the combine button on the Output Devices page header
- **WHEN** the system is not in selection mode
- **THEN** the system SHALL enter selection mode
- **THEN** the combine button SHALL be displayed in the audio accent color
- **THEN** each real sink row SHALL display a checkbox (using the custom checkbox component) on the right side
- **THEN** combined sink rows SHALL NOT display a checkbox (or display a disabled checkbox)

#### Scenario: User creates a combined sink from selected sinks

- **WHEN** the user is in selection mode
- **WHEN** the user has selected one or more sinks via their checkboxes
- **WHEN** the user presses the combine button again
- **THEN** the system SHALL exit selection mode
- **THEN** the system SHALL load `module-combine-sink` with a unique auto-generated name and the selected sinks as `slaves=`
- **THEN** the new combined sink SHALL be set as the default output device
- **THEN** the combined sink row SHALL appear in the output devices list with the merge icon
- **THEN** the combine button SHALL return to its default color

#### Scenario: User exits selection mode without creating a combined sink

- **WHEN** the user is in selection mode
- **WHEN** no sinks are selected (or the user deselects all)
- **WHEN** the user presses the combine button again
- **THEN** the system SHALL exit selection mode without creating a combined sink
- **THEN** the combine button SHALL return to its default color

### Requirement: Combined sink can be deleted

The system SHALL display a delete button (danger-colored) on each combined sink row. Pressing the delete button SHALL unload the `module-combine-sink` module that created that combined sink. PulseAudio SHALL automatically set a new default sink after deletion.

#### Scenario: User deletes a combined sink

- **WHEN** the user presses the delete button on a combined sink row
- **THEN** the system SHALL unload the `module-combine-sink` module for that combined sink
- **THEN** PulseAudio SHALL automatically set a new default output device
- **THEN** the combined sink row SHALL disappear from the output devices list
- **THEN** the remaining slave sinks SHALL return to normal state (no "Combined" badge)

#### Scenario: Combined sink with unavailable first slave is deleted

- **WHEN** the user deletes a combined sink
- **WHEN** the previously default slave sink is no longer available (e.g., unplugged)
- **THEN** PulseAudio SHALL automatically select a new default sink from available sinks

### Requirement: Multiple combined sinks are supported

The system SHALL allow the user to create multiple combined sinks. Each combined sink SHALL be a separate `module-combine-sink` instance with a unique auto-generated name. The system SHALL display all combined sinks in the output devices list with the same visual treatment (icon + delete button).

#### Scenario: User creates a second combined sink

- **WHEN** the user has already created one combined sink
- **WHEN** the user enters selection mode and selects a different set of sinks
- **WHEN** the user presses the combine button again
- **THEN** the system SHALL create a second combined sink with a unique auto-generated name
- **THEN** both combined sinks SHALL appear in the output devices list
- **THEN** each combined sink SHALL have its own delete button

### Requirement: Combined sink state is discovered automatically from PulseAudio

The system SHALL detect the presence of combined sinks by inspecting the loaded module list for `module-combine-sink`. When the Output Devices page synchronizes with PulseAudio state, it SHALL determine which sinks are combined sinks and update the UI accordingly (icon, delete button). This detection SHALL work for combined sinks created both inside and outside the app.

#### Scenario: External combined sink is detected

- **WHEN** PulseAudio state is refreshed
- **WHEN** a `module-combine-sink` module is loaded externally (e.g., via `pactl`)
- **THEN** the Output Devices page SHALL display the combined sink row with the special icon
- **THEN** the combined sink row SHALL display a delete button
- **THEN** the system SHALL treat the external combined sink the same as an internally-created one

#### Scenario: External combined sink is unloaded

- **WHEN** a `module-combine-sink` module is unloaded externally
- **WHEN** PulseAudio state is refreshed
- **THEN** the combined sink row SHALL disappear from the output devices list
- **THEN** the remaining sinks SHALL return to normal state
