## ADDED Requirements

### Requirement: Script detects host architecture
The system SHALL detect the host CPU architecture via `uname -m` and select the corresponding release tarball.

#### Scenario: x86_64 detection
- **WHEN** the install script runs on an x86_64 system
- **THEN** it downloads the `linux-x86_64` tarball

#### Scenario: aarch64 detection
- **WHEN** the install script runs on an aarch64 system
- **THEN** it downloads the `linux-aarch64` tarball

#### Scenario: Unsupported architecture
- **WHEN** the install script runs on an unsupported architecture (e.g., `i386`)
- **THEN** it prints a clear error message and exits with a non-zero code

### Requirement: Script detects installation privileges
The system SHALL determine whether it is running with root privileges and choose between system-wide and user-local installation paths.

#### Scenario: Root installation
- **WHEN** the script is run as root (or via `sudo`)
- **THEN** it installs the binary to `/usr/local/bin/`, the PolicyKit policy to `/usr/share/polkit-1/actions/`, and the `.desktop` file to `/usr/share/applications/`

#### Scenario: User installation
- **WHEN** the script is run as a non-root user
- **THEN** it installs the binary to `~/.local/bin/`, the `.desktop` file to `~/.local/share/applications/`, and skips the PolicyKit policy with a warning

### Requirement: Script downloads and extracts tarball
The system SHALL download the correct release tarball from GitHub Releases, verify it exists, and extract it to a temporary directory.

#### Scenario: Successful download
- **WHEN** the script constructs the download URL from the detected architecture and latest release tag
- **THEN** it downloads the tarball via HTTPS and extracts it without error

#### Scenario: Download failure
- **WHEN** the tarball download fails (e.g., 404 or network error)
- **THEN** the script prints the error, cleans up temporary files, and exits with a non-zero code

### Requirement: Script installs binary
The system SHALL copy the extracted `bludio` binary to the target bin directory and make it executable.

#### Scenario: Binary installed
- **WHEN** the script copies `bludio` to the target directory
- **THEN** the binary exists at the target path with executable permissions

### Requirement: Script installs PolicyKit policy when privileged
The system SHALL copy the PolicyKit policy file to `/usr/share/polkit-1/actions/` only when running with root privileges.

#### Scenario: Policy installed with root
- **WHEN** the script runs as root
- **THEN** `dev.toomosin.bludio.policy` is copied to `/usr/share/polkit-1/actions/`

#### Scenario: Policy skipped without root
- **WHEN** the script runs as a non-root user
- **THEN** it skips the policy installation and prints a warning that privileged Bluetooth operations will be unavailable

### Requirement: Script installs .desktop file
The system SHALL copy the `.desktop` file to the appropriate applications directory for the detected install mode.

#### Scenario: Desktop file installed
- **WHEN** the script completes the installation
- **THEN** the `bludio.desktop` file exists in the target applications directory

### Requirement: Script handles PATH for user installs
The system SHALL warn the user if `~/.local/bin` is not on their PATH and provide instructions to add it.

#### Scenario: PATH check
- **WHEN** the script performs a user-local install
- **THEN** it checks if the target bin directory is in `PATH` and prints a warning with shell-specific instructions if not

### Requirement: Script is executable and POSIX-compliant
The system SHALL be a POSIX shell script (`/bin/sh`) that requires no dependencies beyond standard POSIX utilities (`curl`, `tar`, `uname`, `id`, `chmod`) and does not require `bash`.

#### Scenario: POSIX execution
- **WHEN** the script is run with `/bin/sh`
- **THEN** it executes successfully without bash-specific syntax

### Requirement: README documents the install command
The system SHALL include installation instructions in `README.md` covering both `curl | bash` and `curl | sudo bash` usage, plus manual download instructions.

#### Scenario: README instructions
- **WHEN** a user reads the README
- **THEN** they find a clear "Install" section with the `curl` command, the `sudo` variant, and manual steps
