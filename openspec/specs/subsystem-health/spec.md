# subsystem-health

## Purpose

Provide unified subsystem status tracking, disconnect detection, and automatic reconnection for backend subsystems (Bluetooth, Audio). Ensures the UI reacts appropriately to subsystem unavailability and recovery.

## Requirements

### Requirement: Unified subsystem status model

The system SHALL maintain a `SubsystemStatus` enum for each backend subsystem (Bluetooth, Audio) that represents the current connection state. The enum SHALL have four variants: `Connecting` (initial startup), `Connected` (operational), `Disconnected(String)` (with a user-facing error message), and `Reconnecting` (attempting recovery after a disconnect). This status SHALL be the single source of truth for whether a subsystem is operational. Pages SHALL render based on a single `match` on this enum — no boolean flags or `.is_none()` checks.

#### Scenario: Subsystem is connecting for the first time

- **WHEN** the application starts and a subsystem has not yet established its connection
- **THEN** its `SubsystemStatus` SHALL be `Connecting`
- **THEN** the corresponding page SHALL display a "Connecting to ..." message in the content area

#### Scenario: Subsystem is connected and operational

- **WHEN** a subsystem (Bluetooth or Audio) is initialized and responding
- **THEN** its `SubsystemStatus` SHALL be `Connected`

#### Scenario: Subsystem becomes unavailable

- **WHEN** a subsystem's underlying service (BlueZ or PulseAudio) becomes unavailable
- **THEN** its `SubsystemStatus` SHALL transition to `Disconnected(message)` with a human-readable error description
- **THEN** the corresponding page entity SHALL be notified to update its display

#### Scenario: Subsystem is attempting to reconnect

- **WHEN** a disconnected subsystem begins automatic reconnection
- **THEN** its `SubsystemStatus` SHALL transition to `Reconnecting`
- **THEN** the corresponding page entity SHALL be notified to update its display

#### Scenario: Subsystem reconnects successfully

- **WHEN** a reconnecting subsystem re-establishes its connection
- **THEN** its `SubsystemStatus` SHALL transition to `Connected`
- **THEN** the corresponding page entity SHALL receive fresh state data
- **THEN** the page SHALL resume displaying the device list

### Requirement: Bluetooth disconnect detection via D-Bus NameOwnerChanged

The system SHALL subscribe to D-Bus `NameOwnerChanged` signals for `org.bluez` on the session bus to detect when the BlueZ service becomes unavailable or returns. When the signal indicates BlueZ has lost its owner, the system SHALL set the Bluetooth subsystem status to `Disconnected`. When the signal indicates BlueZ has regained an owner, the system SHALL set the status to `Reconnecting` and begin reconnection.

#### Scenario: BlueZ service stops

- **WHEN** the BlueZ D-Bus service stops (e.g., `systemctl restart bluetooth`)
- **THEN** the D-Bus `NameOwnerChanged` signal SHALL be received with an empty new owner
- **THEN** the Bluetooth subsystem status SHALL become `Disconnected("Bluetooth service unavailable")`
- **THEN** the Bluetooth page SHALL display the disconnected state

#### Scenario: BlueZ service restarts

- **WHEN** the BlueZ D-Bus service becomes available again after a restart
- **THEN** the D-Bus `NameOwnerChanged` signal SHALL be received with a new owner
- **THEN** the Bluetooth subsystem status SHALL become `Reconnecting`
- **THEN** the system SHALL attempt to reinitialize `BluetoothState` (new session, adapter, agent)

#### Scenario: Bluetooth reconnection succeeds

- **WHEN** the reinitialization of `BluetoothState` completes successfully
- **THEN** the Bluetooth subsystem status SHALL become `Connected`
- **THEN** the Bluetooth page SHALL display the refreshed device list
- **THEN** the D-Bus property monitor SHALL be re-spawned with the new adapter

### Requirement: Audio disconnect detection via channel closure and context state

The system SHALL detect PulseAudio disconnection by observing the closure of the `audio_state_rx` channel (which indicates the PA thread has exited) or by the PA thread itself detecting `ContextState::Failed` or `ContextState::Terminated` on the PulseAudio context.

#### Scenario: PulseAudio process exits

- **WHEN** the PulseAudio or PipeWire process exits
- **THEN** the PA mainloop SHALL terminate
- **THEN** the PA thread SHALL exit, dropping the `AudioState` sender
- **THEN** `spawn_audio_state_loop` SHALL detect the channel closure (`rx.recv()` returns `None`)
- **THEN** the Audio subsystem status SHALL become `Disconnected("PulseAudio unavailable")`

#### Scenario: PA context enters failed state

- **WHEN** the PulseAudio context transitions to `Failed` or `Terminated` during the event loop
- **THEN** the PA thread SHALL send a final `AudioState` with `connected: false` and an error message
- **THEN** the PA thread SHALL exit
- **THEN** the Audio subsystem status SHALL become `Disconnected`

### Requirement: Audio reconnection via short fixed-interval retry

When the Audio subsystem status is `Disconnected`, the system SHALL spawn a reconnection task that attempts to create a new PulseAudio thread on a fixed 3-second interval. There SHALL be no exponential backoff — the interval remains constant to ensure fast recovery when the subsystem restarts.

#### Scenario: PulseAudio becomes available after restart

- **WHEN** PulseAudio is restarted and the reconnect task attempts connection
- **THEN** the new PA thread SHALL successfully initialize
- **THEN** the wakeup handle SHALL be received
- **THEN** the Audio subsystem status SHALL become `Connected`
- **THEN** fresh channels SHALL be wired to all audio page entities

#### Scenario: PulseAudio remains unavailable

- **WHEN** the reconnect attempt fails (PA not yet available)
- **THEN** the task SHALL wait 3 seconds and retry
- **THEN** the task SHALL continue retrying every 3 seconds until successful
- **THEN** the Audio subsystem status SHALL remain `Disconnected` between attempts

#### Scenario: Fast recovery after PipeWire restart

- **WHEN** PipeWire restarts (typically completes in 1–2 seconds)
- **THEN** the reconnect task SHALL detect availability on the next attempt (within 3 seconds)
- **THEN** the total downtime from the user's perspective SHALL be under 5 seconds

### Requirement: Bluetooth adapter property monitoring for rfkill detection

The system SHALL monitor adapter-level D-Bus property changes, specifically `Powered`, on the default Bluetooth adapter. When the adapter's `Powered` property changes, the system SHALL update the Bluetooth subsystem status accordingly. This enables reactive detection of rfkill block/unblock events without waiting for the periodic monitor re-sync.

#### Scenario: Adapter powered off (rfkill block)

- **WHEN** the adapter's `Powered` property changes to `false`
- **THEN** the Bluetooth subsystem status SHALL become `Disconnected("Bluetooth adapter powered off")`
- **THEN** the Bluetooth page SHALL display the disconnected state in the page content area
- **THEN** the per-device monitor listeners SHALL be cleaned up

#### Scenario: Adapter powered on (rfkill unblock)

- **WHEN** the adapter's `Powered` property changes to `true` while the subsystem status is `Disconnected`
- **THEN** the Bluetooth subsystem status SHALL become `Reconnecting`
- **THEN** the system SHALL attempt to reinitialize the adapter (set pairable, enumerate devices)
- **THEN** on success, the subsystem status SHALL become `Connected` and the monitor SHALL resume

### Requirement: UI displays disconnected and reconnecting states

Each page (Bluetooth, Audio Output, Audio Input, Configuration) SHALL render appropriate visual states when its subsystem is disconnected or reconnecting. These states SHALL use the page-level content area (the same region used for "Connecting to..." loading states), NOT the error banner component. The error banner is reserved for transient device-level action errors while the subsystem is connected.

#### Scenario: Bluetooth page shows disconnected state

- **WHEN** the Bluetooth subsystem status is `Disconnected`
- **THEN** the Bluetooth page SHALL display the error message in the page content area (not the error banner)
- **THEN** the device list SHALL NOT be displayed
- **THEN** the scan button SHALL be disabled

#### Scenario: Bluetooth page shows reconnecting state

- **WHEN** the Bluetooth subsystem status is `Reconnecting`
- **THEN** the Bluetooth page SHALL display "Reconnecting to Bluetooth..." in the page content area
- **THEN** the device list SHALL NOT be displayed
- **THEN** the scan button SHALL be disabled

#### Scenario: Audio page shows disconnected state

- **WHEN** the Audio subsystem status is `Disconnected`
- **THEN** the audio page SHALL display the error message in the page content area (not the error banner)
- **THEN** the device list SHALL NOT be displayed
- **THEN** volume sliders and action buttons SHALL NOT be rendered

#### Scenario: Audio page shows reconnecting state

- **WHEN** the Audio subsystem status is `Reconnecting`
- **THEN** the audio page SHALL display "Reconnecting to PulseAudio..." in the page content area
- **THEN** the device list SHALL NOT be displayed
