# bluetooth-device-management

## Purpose

Manage Bluetooth devices via the BlueZ D-Bus interface. Provides a scrollable UI list of known devices with per-device action buttons (connect, disconnect, forget, pair+trust), real-time device scanning with name resolution, and reactive state tracking via D-Bus property-change signals.

## Requirements

### Requirement: Application displays a scrollable list of Bluetooth devices

The system SHALL query the BlueZ D-Bus service on startup and render a vertically scrollable list of known Bluetooth devices (paired and recently discovered). Each list entry SHALL display the device name (or alias), MAC address, and current connection status. Unnamed devices SHALL be hidden by default, matching blueman's default filter.

#### Scenario: No devices known
- **WHEN** no Bluetooth devices are paired and no devices have been discovered
- **THEN** the list area SHALL display a "No devices" placeholder

#### Scenario: Multiple devices present
- **WHEN** one or more Bluetooth devices are known
- **THEN** the list SHALL display each device's name, MAC address, and connection status
- **THEN** the list SHALL be vertically scrollable when content exceeds the visible area
- **THEN** devices SHALL be sorted: paired first, then connected, then alphabetically

#### Scenario: BlueZ D-Bus service unavailable
- **WHEN** the BlueZ D-Bus service is not running or not accessible
- **THEN** the application SHALL display an error message indicating Bluetooth is unavailable
- **THEN** the application SHALL continue running without crashing

### Requirement: Device action buttons per list entry

The system SHALL render Connect, Disconnect, and Forget action buttons for each device in the list. Each button SHALL trigger the corresponding BlueZ D-Bus operation asynchronously without blocking the UI.

#### Scenario: Connect to a paired but disconnected device
- **WHEN** user clicks the Connect button on a paired, disconnected device
- **THEN** the system SHALL call `Device1.Connect` over D-Bus
- **THEN** the connection status SHALL update in the UI upon success

#### Scenario: Disconnect from a connected device
- **WHEN** user clicks the Disconnect button on a connected device
- **THEN** the system SHALL call `Device1.Disconnect` over D-Bus
- **THEN** the connection status SHALL update to disconnected in the UI

#### Scenario: Forget a paired device
- **WHEN** user clicks the Forget button on any paired device
- **THEN** the system SHALL call `Adapter1.RemoveDevice` with the device's object path
- **THEN** the device SHALL be removed from the list after successful removal

### Requirement: Scan for nearby Bluetooth devices

The system SHALL provide a Scan button that initiates device discovery via `Adapter1.StartDiscovery`. During scanning, newly discovered devices SHALL appear in the UI list in real time as they are reported by BlueZ.

#### Scenario: Start scanning
- **WHEN** user clicks the Scan button
- **THEN** the system SHALL call `StartDiscovery` on the default adapter
- **THEN** the Scan button SHALL change to a Stop button while discovery is active

#### Scenario: Device discovered during scan
- **WHEN** a new Bluetooth device is detected during an active scan
- **THEN** the device SHALL appear in the list with its resolved name and address
- **THEN** the device SHALL remain in the list after scanning stops

#### Scenario: Stop scanning
- **WHEN** user clicks the Stop button during an active scan
- **THEN** the system SHALL stop the discovery stream (dropping the stream automatically stops `StartDiscovery`)
- **THEN** the button SHALL revert to Scan
- **THEN** a final full device list refresh SHALL run to resolve any outstanding device names

#### Scenario: Scan timeout
- **WHEN** discovery has been active for 30 seconds without manual stop
- **THEN** the system SHALL automatically stop discovery
- **THEN** the button SHALL revert to Scan

### Requirement: Pair, trust, and connect a discovered device

The system SHALL provide a Pair & Trust button for each discovered (unpaired) device. Clicking this button SHALL call `Device1.Pair`, set `Trusted = true`, and then call `Device1.Connect` to establish profile connections. A pairing agent SHALL be registered once at startup to handle passkey/confirmation dialogs.

#### Scenario: Pair, trust, and connect a new device
- **WHEN** user clicks the Pair & Trust button on an unpaired device
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
- **THEN** the device SHALL remain in its current state in the UI
- **THEN** the failure SHALL be logged and the application SHALL continue running

### Requirement: Bluetooth state tracking and UI reactivity

The system SHALL maintain an in-memory model of Bluetooth state (device list, discovery status) that the UI observes. Any change to this state SHALL automatically trigger a re-render of the affected UI components. Per-device D-Bus `PropertiesChanged` signals SHALL be monitored for instant connect/disconnect/rename updates, with a 10-second full-list refresh as a fallback.

#### Scenario: Device property changes update the UI instantly
- **WHEN** a device's `Connected`, `Paired`, or `Name` property changes (via D-Bus signal)
- **THEN** the UI SHALL reflect the change within one frame (no polling delay)

#### Scenario: Fallback refresh catches missed changes
- **WHEN** no D-Bus signals arrive for 10 seconds
- **THEN** a full device list refresh SHALL run to catch any changes missed by signal monitoring

#### Scenario: Discovery status changes update the UI
- **WHEN** a scan starts or stops
- **THEN** the header button SHALL toggle between Scan and Stop
- **THEN** the device list SHALL update as devices are discovered
