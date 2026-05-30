## MODIFIED Requirements

### Requirement: Application displays a scrollable list of Bluetooth devices

The system SHALL query the BlueZ D-Bus service on startup and render a vertically scrollable list of known Bluetooth devices (paired and recently discovered). Each list entry SHALL display the device name, MAC address, current connection/pairing status, and a colored status strip on the left edge. Unnamed devices SHALL be hidden by default, matching blueman's default filter. When the Bluetooth subsystem is disconnected or reconnecting, the page SHALL display the corresponding status state instead of the device list.

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
- **THEN** the application SHALL display the error message in the page content area (not the error banner)
- **THEN** the application SHALL continue running without crashing

#### Scenario: Bluetooth initializing
- **WHEN** Bluetooth is initializing and no adapter is available yet
- **THEN** the Bluetooth subsystem status SHALL be `Connecting`
- **THEN** the page SHALL display a "Connecting to Bluetooth..." message in the content area via a `match` on `SubsystemStatus::Connecting`

#### Scenario: Bluetooth disconnected after runtime failure
- **WHEN** the Bluetooth subsystem status transitions to `Disconnected` after initial connection
- **THEN** the page SHALL display the error message in the existing error display area
- **THEN** the device list SHALL NOT be rendered
- **THEN** the scan button SHALL be visually disabled and non-interactive

#### Scenario: Bluetooth reconnecting
- **WHEN** the Bluetooth subsystem status is `Reconnecting`
- **THEN** the page SHALL display "Reconnecting to Bluetooth..." in the content area
- **THEN** the device list SHALL NOT be rendered
- **THEN** the scan button SHALL be visually disabled and non-interactive

#### Scenario: Bluetooth reconnected successfully
- **WHEN** the Bluetooth subsystem status transitions back to `Connected`
- **THEN** the page SHALL resume displaying the device list with fresh data
- **THEN** the scan button SHALL become interactive again

### Requirement: Bluetooth state tracking and UI reactivity

The system SHALL maintain an in-memory `BluetoothState` model that the `BluetoothPage` entity observes via `sync_state()`. The page creates/updates/removes `BluetoothDeviceRow` entities to match the device list. Per-device D-Bus `PropertiesChanged` signals SHALL be monitored via the `monitor` module, with a 10-second full-list refresh as a fallback. When the Bluetooth subsystem disconnects, the monitor loop SHALL terminate and be re-spawned upon reconnection.

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

#### Scenario: Monitor terminates on BlueZ disconnect
- **WHEN** the BlueZ service becomes unavailable
- **THEN** the per-device D-Bus event listeners SHALL terminate (naturally, as the D-Bus objects disappear)
- **THEN** the monitor task SHALL exit
- **THEN** no further device updates SHALL be processed until reconnection

#### Scenario: Monitor re-spawned after reconnection
- **WHEN** the Bluetooth subsystem successfully reconnects
- **THEN** a new monitor task SHALL be spawned with the new adapter handle
- **THEN** D-Bus property monitoring SHALL resume for all current devices

#### Scenario: Adapter powered off via rfkill
- **WHEN** the adapter's `Powered` D-Bus property changes to `false` (e.g., `rfkill block bluetooth`)
- **THEN** the monitor SHALL detect this via the adapter-level property listener
- **THEN** the Bluetooth subsystem status SHALL become `Disconnected`
- **THEN** the Bluetooth page SHALL display the disconnected state in the page content area

#### Scenario: Adapter powered on via rfkill
- **WHEN** the adapter's `Powered` D-Bus property changes to `true` (e.g., `rfkill unblock bluetooth`) while status is `Disconnected`
- **THEN** the monitor SHALL detect this and trigger reconnection
- **THEN** the system SHALL reinitialize the adapter and resume monitoring
