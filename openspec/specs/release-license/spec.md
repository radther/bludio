## ADDED Requirements

### Requirement: Project uses MIT license
The repository SHALL contain a top-level `LICENSE` file containing the full text of the MIT license with the current year and copyright holder.

#### Scenario: License file exists
- **WHEN** a user views the repository root
- **THEN** a `LICENSE` file is present containing the MIT license text

### Requirement: Dependency license audit documented
The repository SHALL contain a `THIRD_PARTY_NOTICES.md` file listing every direct dependency, its license, and a statement confirming MIT compatibility.

#### Scenario: Audit is readable
- **WHEN** a user opens `THIRD_PARTY_NOTICES.md`
- **THEN** it contains a table or list of all direct dependencies with their SPDX license identifiers
- **AND** each entry confirms the license is compatible with MIT

### Requirement: Cargo.toml declares MIT license
The `Cargo.toml` package metadata SHALL include `license = "MIT"` and a `repository` URL.

#### Scenario: Package metadata is valid
- **WHEN** `cargo metadata` is executed
- **THEN** the package `license` field equals `"MIT"`
- **AND** the `repository` field is a valid HTTPS URL
