## ADDED Requirements

### Requirement: Release workflow triggers on version tags
The system SHALL execute a GitHub Actions workflow whenever a git tag matching `v*.*.*` is pushed to the repository.

#### Scenario: Tag push triggers build
- **WHEN** a maintainer pushes a tag `v0.1.0` to the repository
- **THEN** the release workflow starts within 60 seconds

### Requirement: Workflow builds for x86_64 and aarch64
The system SHALL produce a release binary for both `x86_64` and `aarch64` architectures.

#### Scenario: x86_64 build succeeds
- **WHEN** the release workflow runs the x86_64 job
- **THEN** it completes without error and produces a tarball artifact

#### Scenario: aarch64 build succeeds
- **WHEN** the release workflow runs the aarch64 job
- **THEN** it completes without error and produces a tarball artifact

### Requirement: Build uses ubuntu:22.04 container
The system SHALL compile the release binary inside an `ubuntu:22.04` Docker container to ensure the resulting binary links against glibc 2.35.

#### Scenario: Container-based build
- **WHEN** the workflow executes the build step
- **THEN** it runs inside `ubuntu:22.04` with all required development packages installed

### Requirement: Binary is stripped before packaging
The system SHALL run `strip` on the release binary before inclusion in the tarball to minimize size.

#### Scenario: Stripped binary size
- **WHEN** the build completes
- **THEN** the binary has been stripped and the tarball contains only the stripped binary

### Requirement: Tarball contains required files
The system SHALL package each release as a tarball containing the binary, the PolicyKit policy file, and a `.desktop` file.

#### Scenario: Tarball contents
- **WHEN** the workflow creates the release tarball
- **THEN** the tarball contains `bludio`, `dev.toomosin.bludio.policy`, and `bludio.desktop`

### Requirement: Release is published to GitHub Releases
The system SHALL create a GitHub Release and attach both architecture tarballs to it.

#### Scenario: Release published
- **WHEN** both build jobs complete successfully
- **THEN** a GitHub Release named `v0.1.0` exists with both tarballs attached

### Requirement: Cargo.lock is respected
The system SHALL use `--locked` when building to ensure reproducible builds using the exact dependency versions in `Cargo.lock`.

#### Scenario: Locked build
- **WHEN** the workflow runs `cargo build --release`
- **THEN** it passes the `--locked` flag
