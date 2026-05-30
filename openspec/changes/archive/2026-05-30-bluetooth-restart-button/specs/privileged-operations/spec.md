## ADDED Requirements

### Requirement: Bluetooth restart via pkexec

The system SHALL allow users to restart the Bluetooth service by executing `pkexec dev.toomosin.bludio.restart-bluetooth-service`. The system SHALL use `tokio::process::Command` to run pkexec asynchronously. The system SHALL NOT handle or store the user's password — authentication is delegated entirely to the system's PolicyKit agent.

#### Scenario: Successful restart

- **WHEN** the user clicks the restart Bluetooth button and authenticates successfully in the polkit dialog
- **THEN** pkexec SHALL execute `systemctl restart bluetooth`
- **THEN** the button SHALL return to its normal state after the command completes

#### Scenario: User cancels authentication

- **WHEN** the user clicks the restart Bluetooth button and cancels the polkit dialog
- **THEN** the system SHALL detect exit code 126
- **THEN** the system SHALL display an error message: "Authentication cancelled"

#### Scenario: Policy file not installed

- **WHEN** the user clicks the restart Bluetooth button and the policy file is not installed
- **THEN** pkexec SHALL exit with code 127
- **THEN** the system SHALL display an error message indicating the action is not authorized

### Requirement: pkexec error handling

The system SHALL handle all pkexec exit codes and map them to user-facing messages. The system SHALL NOT crash or hang on pkexec failure. The system SHALL display errors via the existing error banner on the Bluetooth page.

#### Scenario: pkexec binary not found

- **WHEN** pkexec is not installed on the system
- **THEN** the system SHALL catch the spawn error
- **THEN** the system SHALL display an error message: "pkexec not found — PolicyKit is required"

#### Scenario: pkexec returns unknown error

- **WHEN** pkexec exits with an unexpected exit code
- **THEN** the system SHALL display the stderr output as the error message

### Requirement: Restart button disabled during execution

The system SHALL prevent multiple simultaneous pkexec invocations. While a pkexec call is in-flight, the restart button SHALL be visually disabled and SHALL not respond to clicks.

#### Scenario: Button is disabled while command runs

- **WHEN** a pkexec call is in progress
- **THEN** the restart button SHALL be visually dimmed
- **THEN** clicking the button SHALL have no effect

#### Scenario: Button re-enables after command completes

- **WHEN** a pkexec call completes (success or failure)
- **THEN** the restart button SHALL return to its normal interactive state
