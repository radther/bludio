## MODIFIED Requirements

### Requirement: Pair, trust, and connect a discovered device

The system SHALL provide a Pair & Trust button for each discovered (unpaired) device. Clicking this button SHALL call `Device1.Pair`, set `Trusted = true`, and then call `Device1.Connect` to establish profile connections. A pairing agent SHALL be registered once at startup to handle passkey/confirmation dialogs. During the operation, the system SHALL display intermediate status on the device row (as specified in the `pairing-status-indicator` capability).

#### Scenario: Pair, trust, and connect a new device
- **WHEN** user clicks the Pair & Trust button on an unpaired device
- **THEN** the system SHALL call `Device1.Connect` to establish the ACL connection
- **THEN** the system SHALL call `Device1.Pair` on the device
- **THEN** upon successful pairing, the system SHALL set `Trusted = true`
- **THEN** the system SHALL call `Device1.Connect` to establish profiles
- **THEN** the device SHALL update its status to paired and connected in the UI

#### Scenario: Pairing agent handles passkey confirmation
- **WHEN** BlueZ requests passkey or PIN confirmation during pairing
- **THEN** the registered agent SHALL auto-accept the request
- **THEN** pairing SHALL proceed without blocking the UI

#### Scenario: Pairing fails
- **WHEN** the Pair operation fails (e.g., timeout, rejection)
- **THEN** the device row SHALL display a visible failure indicator ("failed to pair" status with red dot)
- **THEN** a detailed error message SHALL be logged to the console via `eprintln!`
- **THEN** the device SHALL remain in its current state in the UI
- **THEN** the failure SHALL NOT crash or hang the application
