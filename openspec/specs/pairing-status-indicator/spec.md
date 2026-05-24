# pairing-status-indicator

## Purpose

Display a per-device visual status indicator on Bluetooth device rows during active pairing operations, showing the current step in the pairing/connection chain with a color-coded dot and brief descriptive text. Provide clear failure state display and automatic cleanup.

## Requirements

### Requirement: Device row displays pairing status during active operations

The system SHALL display a per-device status indicator on the device row while a Bluetooth pairing/connection operation is in progress. The indicator SHALL consist of a colored circle dot and brief descriptive text reflecting the current step in the operation chain.

#### Scenario: Pairing chain status progression
- **WHEN** user clicks "Pair & Trust" on an unpaired device
- **THEN** the device row SHALL show status "connecting…" with a blue dot while the connection is being established
- **THEN** the device row SHALL show status "pairing…" with a yellow dot while the pairing handshake is in progress
- **THEN** the device row SHALL show status "trusting…" with a green dot while trust is being set

#### Scenario: Pairing completes successfully
- **WHEN** the Pair & Trust operation completes successfully
- **THEN** the pairing status indicator SHALL be removed from the device row
- **THEN** the normal device status (connected/paired/discovered) SHALL be displayed

#### Scenario: Pairing fails
- **WHEN** any step in the Pair & Trust operation fails (connection timeout, pairing rejection, etc.)
- **THEN** the device row SHALL show status "failed to pair" with a red dot
- **THEN** a detailed error message SHALL be logged to the console via `eprintln!`
- **THEN** the device SHALL remain in its current paired/connected state (reverting any partial progress)
- **THEN** the failure status SHALL clear when the next full device list refresh runs

#### Scenario: Multiple devices can be operated independently
- **WHEN** user initiates a pairing operation on device A while device B is already pairing
- **THEN** each device SHALL independently track its own pairing status
- **THEN** operations on different devices SHALL not interfere with each other

### Requirement: Pairing status indicator uses brief, user-friendly language

The pairing status text SHALL be concise (one or two words) and descriptive. The system SHALL use standardized status labels for each stage.

#### Scenario: Status labels for each stage
- **WHEN** the connection is being established
- **THEN** the indicator SHALL display "connecting…"
- **WHEN** the pairing handshake is in progress
- **THEN** the indicator SHALL display "pairing…"
- **WHEN** trust is being set on the device
- **THEN** the indicator SHALL display "trusting…"
- **WHEN** pairing fails
- **THEN** the indicator SHALL display "failed to pair"

### Requirement: Pairing status dot follows established visual conventions

The status dot SHALL be a Unicode circle character with a color appropriate to the current state.

#### Scenario: Status dot colors
- **WHEN** the status is "connecting…"
- **THEN** the dot SHALL be rendered in a blue/cyan hue (matching the app's accent color)
- **WHEN** the status is "pairing…"
- **THEN** the dot SHALL be rendered in a yellow/amber hue
- **WHEN** the status is "trusting…"
- **THEN** the dot SHALL be rendered in a green hue
- **WHEN** the status is "failed to pair"
- **THEN** the dot SHALL be rendered in a red hue

### Requirement: Pairing status clears automatically on device list refresh

The pairing status SHALL be transient — it SHALL be cleared after the next full device list refresh (D-Bus signal-driven single-device refresh or 10s fallback poll) that shows the device in a final paired/connected state.

#### Scenario: Status clears after successful operation
- **WHEN** a Pair & Trust operation completes successfully
- **THEN** the pairing status field on the device SHALL be set to `None`
- **THEN** subsequent renders SHALL show only the normal connected/paired status
