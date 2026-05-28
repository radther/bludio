# bluetooth-device-management

## Purpose

Manage Bluetooth devices via the BlueZ D-Bus interface. Provides a scrollable UI list of known devices with per-device action buttons (connect, disconnect, forget, pair+trust), real-time device scanning with name resolution, and reactive state tracking via D-Bus property-change signals.

## Requirements

### Requirement: Application displays a scrollable list of Bluetooth devices

The system SHALL query the BlueZ D-Bus service on startup and render a vertically scrollable list of known Bluetooth devices (paired and recently discovered). Each list entry SHALL display the device name, MAC address, current connection/pairing status, and a colored status strip on the left edge. Unnamed devices SHALL be hidden by default, matching blueman's default filter.

#### Scenario: No devices known
- **WHEN** no Bluetooth devices are paired and no devices have been discovered
- **THEN** the list area SHALL display a "No devices. Press \"scan\" to discover." placeholder

#### Scenario: Multiple devices present
- **WHEN** one or more Bluetooth devices are known
- **THEN** the list SHALL display each device's name, MAC address, and status text (connected/paired/discovered)
- **THEN** each device row SHALL have a colored status strip: green for connected, secondary text color for others
- **THEN** the list SHALL be vertically scrollable when content exceeds the visible area
- **THEN** devices SHALL be sorted: paired first, then connected, then alphabetically

#### Scenario: BlueZ D-Bus service unavailable
- **WHEN** the BlueZ D-Bus service is not running or not accessible
- **THEN** the application SHALL display an error banner with the error message
- **THEN** the application SHALL continue running without crashing

#### Scenario: Bluetooth initializing
- **WHEN** Bluetooth is initializing and no adapter is available yet
- **THEN** the page SHALL display a "Connecting to Bluetooth..." message

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

### Requirement: Scan for nearby Bluetooth devices

The system SHALL provide a Scan button that sends `BluetoothPageCommand::ToggleScan` through the command channel. The app handler SHALL toggle `bt_state.discovering` and spawn `run_discovery` when starting. During scanning, newly discovered devices SHALL appear in the UI list in real time.

#### Scenario: Start scanning
- **WHEN** user clicks the Scan button
- **THEN** the system SHALL send `BluetoothPageCommand::ToggleScan`
- **THEN** the Scan button SHALL change to a Stop button (danger background) while discovery is active

#### Scenario: Device discovered during scan
- **WHEN** a new Bluetooth device is detected during an active scan
- **THEN** the device SHALL appear in the list with its resolved name and address
- **THEN** the device SHALL remain in the list after scanning stops

#### Scenario: Stop scanning
- **WHEN** user clicks the Stop button during an active scan
- **THEN** the system SHALL send `BluetoothPageCommand::ToggleScan` to stop discovery
- **THEN** the button SHALL revert to Scan
- **THEN** a final full device list refresh SHALL run to resolve any outstanding device names

### Requirement: Pair, trust, and connect a discovered device

The system SHALL execute a multi-step Pair & Trust sequence for each discovered (unpaired) device: Connect → Pair → Trust + Reconnect. The sequence SHALL be dispatched from the app's command handler as a background task. Each step SHALL run on the Tokio runtime via `tokio_task`. During the operation, the system SHALL display intermediate status on the device row via the `PairingStatus` model.

#### Scenario: Pair, trust, and connect a new device
- **WHEN** user clicks the Pair & Trust button on an unpaired device
- **THEN** the system SHALL set `pairing_status = Some(PairingStatus::Connecting)` immediately
- **THEN** the system SHALL spawn `execute_pair_and_trust` as a background task
- **THEN** the task SHALL call `Device1.Connect` to establish the ACL connection
- **THEN** the task SHALL call `Device1.Pair` to pair
- **THEN** upon successful pairing, the task SHALL set `Trusted = true`
- **THEN** the task SHALL call `Device1.Connect` to establish profile connections
- **THEN** the device SHALL update its status to paired and connected in the UI

#### Scenario: Pairing agent handles passkey confirmation
- **WHEN** BlueZ requests passkey or PIN confirmation during pairing
- **THEN** the registered agent SHALL auto-accept the request
- **THEN** pairing SHALL proceed without blocking the UI

#### Scenario: Pairing fails
- **WHEN** any step in the Pair & Trust operation fails
- **THEN** the device row SHALL display `PairingStatus::Failed` with a red status strip
- **THEN** a detailed error message SHALL be logged to stderr
- **THEN** the device SHALL remain in its current state in the UI

### Requirement: Bluetooth state tracking and UI reactivity

The system SHALL maintain an in-memory `BluetoothState` model that the `BluetoothPage` entity observes via `sync_state()`. The page creates/updates/removes `BluetoothDeviceRow` entities to match the device list. Per-device D-Bus `PropertiesChanged` signals SHALL be monitored via the `monitor` module, with a 10-second full-list refresh as a fallback.

#### Scenario: Device property changes update the UI instantly
- **WHEN** a device's `Connected`, `Paired`, or `Name` property changes (via D-Bus signal)
- **THEN** the monitor SHALL emit the device address through the change channel
- **THEN** a quick single-device refresh SHALL update that device's row in the UI

#### Scenario: Fallback refresh catches missed changes
- **WHEN** no D-Bus signals arrive for 10 seconds
- **THEN** a full device list refresh SHALL run to catch any changes missed by signal monitoring

#### Scenario: Discovery status changes update the UI
- **WHEN** a scan starts or stops
- **THEN** the header button SHALL toggle between Scan and Stop
- **THEN** the device list SHALL update as devices are discovered
