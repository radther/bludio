## MODIFIED Requirements

### Requirement: Bluetooth restart via pkexec
The system SHALL allow users to restart the full audio stack by executing a sequence of commands via `tokio::process::Command`. The sequence SHALL be: (1) `systemctl --user restart wireplumber pipewire pipewire-pulse`, (2) `pkexec systemctl restart bluetooth`, (3) `rfkill unblock bluetooth`. The system SHALL NOT handle or store the user's password — authentication is delegated entirely to the system's PolicyKit agent for the pkexec step only. The system SHALL collect per-step failures and surface them to the user.

#### Scenario: Successful full stack restart
- **WHEN** the user clicks the restart audio stack button and authenticates successfully in the polkit dialog
- **THEN** the system SHALL execute `systemctl --user restart wireplumber pipewire pipewire-pulse`
- **THEN** the system SHALL execute `pkexec systemctl restart bluetooth`
- **THEN** the system SHALL execute `rfkill unblock bluetooth`
- **THEN** the button SHALL return to its normal state after all commands complete

#### Scenario: User cancels authentication
- **WHEN** the user clicks the restart audio stack button and cancels the polkit dialog
- **THEN** the system SHALL detect exit code 126
- **THEN** the system SHALL display an error message: "Authentication cancelled"

#### Scenario: Policy file not installed
- **WHEN** the user clicks the restart audio stack button and the policy file is not installed
- **THEN** pkexec SHALL exit with code 127
- **THEN** the system SHALL display an error message indicating the action is not authorized

#### Scenario: One step fails but others continue
- **WHEN** the user clicks the restart audio stack button
- **THEN** if the `rfkill` command fails (e.g. binary not found), the system SHALL continue execution
- **THEN** the system SHALL surface the `rfkill` failure in the error banner alongside any other failures
- **THEN** the remaining commands SHALL still execute

## ADDED Requirements

### Requirement: Audio stack restart includes user services
The system SHALL restart the PipeWire user services (`wireplumber`, `pipewire`, `pipewire-pulse`) as part of the audio stack restart sequence. This command SHALL run without `pkexec` because `--user` services are owned by the current session.

#### Scenario: User services restart without elevation
- **WHEN** the audio stack restart sequence begins
- **THEN** the first command SHALL be `systemctl --user restart wireplumber pipewire pipewire-pulse`
- **THEN** this command SHALL run without pkexec elevation
- **THEN** on success, the system SHALL proceed to the Bluetooth restart step
