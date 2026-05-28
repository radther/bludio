## MODIFIED Requirements

### Requirement: Device action buttons per list entry

The system SHALL render conditional action buttons for each device: Connect (paired + disconnected), Disconnect (connected), Forget (paired), and Pair & Trust (unpaired + no active pairing). Each button SHALL send a `BluetoothPageCommand::DeviceAction` through the command channel for async execution on the Tokio runtime. Failed actions SHALL display a human-readable error message in the Bluetooth page's error banner.

#### Scenario: Connect to a paired but disconnected device
- **WHEN** user clicks the Connect button on a paired, disconnected device
- **THEN** the system SHALL send `BluetoothPageCommand::DeviceAction { addr, action: Connect }` through the command channel
- **THEN** the connection status SHALL update in the UI upon success

#### Scenario: Connect fails
- **WHEN** the D-Bus connect call fails
- **THEN** the error banner SHALL display a human-readable error message (e.g., "Device is out of range" instead of the raw D-Bus error)
- **THEN** the device status SHALL remain unchanged

#### Scenario: Disconnect from a connected device
- **WHEN** user clicks the Disconnect button on a connected device
- **THEN** the system SHALL send `BluetoothPageCommand::DeviceAction { addr, action: Disconnect }` through the command channel
- **THEN** the connection status SHALL update to disconnected in the UI

#### Scenario: Disconnect fails
- **WHEN** the D-Bus disconnect call fails
- **THEN** the error banner SHALL display a human-readable error message
- **THEN** the device status SHALL remain unchanged

#### Scenario: Forget a paired device
- **WHEN** user clicks the Forget button on any paired device
- **THEN** the system SHALL send `BluetoothPageCommand::DeviceAction { addr, action: Forget }` through the command channel
- **THEN** the device SHALL be removed from the list after successful removal

#### Scenario: Forget fails
- **WHEN** the D-Bus remove_device call fails
- **THEN** the error banner SHALL display a human-readable error message
- **THEN** the device SHALL remain in the list

#### Scenario: Pair & Trust button is hidden during active pairing
- **WHEN** a device has an active pairing status (Connecting, Pairing, or Trusting)
- **THEN** the Pair & Trust button SHALL NOT be displayed on that device row
