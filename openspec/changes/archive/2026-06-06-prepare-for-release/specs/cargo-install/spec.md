## ADDED Requirements

### Requirement: Application installable via cargo install
The project SHALL be installable using `cargo install --git <repo-url>` without requiring the user to clone the repository manually, producing a runnable `bludio` binary.

#### Scenario: Install from GitHub URL succeeds
- **WHEN** a user runs `cargo install --git https://github.com/toomosin/bludio`
- **THEN** the repository is fetched, compiled, and the binary is installed to the Cargo bin directory
- **AND** running `bludio --version` (or equivalent) exits successfully

### Requirement: PolicyKit file installed during cargo install
A `build.rs` script SHALL copy `policy/dev.toomosin.bludio.policy` to the system PolicyKit actions directory when the install target prefix is writable.

#### Scenario: System install with privileges
- **WHEN** `cargo install --git <repo-url>` is run as root (or with `sudo`) on Linux
- **THEN** the policy file is copied to `/usr/share/polkit-1/actions/dev.toomosin.bludio.policy`

#### Scenario: User install without privileges
- **WHEN** `cargo install --git <repo-url>` is run as a non-root user
- **THEN** the build completes without error
- **AND** a warning is printed instructing the user to manually copy the policy file or re-run with appropriate privileges

### Requirement: Policy install path is configurable
The `build.rs` script SHALL respect an optional `BLUDIO_POLICY_DIR` environment variable to override the default PolicyKit actions directory.

#### Scenario: Custom policy directory
- **WHEN** `BLUDIO_POLICY_DIR=/custom/path cargo install --git <repo-url>` is executed
- **THEN** the policy file is copied to `/custom/path/dev.toomosin.bludio.policy`

### Requirement: Cargo.toml metadata is publication-ready
The `Cargo.toml` SHALL contain a meaningful `description`, `license`, `repository`, `keywords`, and `categories` suitable for crates.io.

#### Scenario: Metadata is complete
- **WHEN** `cargo metadata --format-version 1` is executed
- **THEN** the package contains non-empty `description`, `license`, `repository`, `keywords`, and `categories` fields
