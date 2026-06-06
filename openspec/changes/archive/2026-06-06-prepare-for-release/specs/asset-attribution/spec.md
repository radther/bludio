## ADDED Requirements

### Requirement: Font licenses bundled
The repository SHALL contain the full SIL Open Font License 1.1 text for every bundled OFL font (Noto Sans and OpenDyslexic).

#### Scenario: Noto Sans license is present
- **WHEN** a user inspects the `licenses/` directory
- **THEN** a file named `LICENSE-NOTO-SANS` contains the full SIL OFL 1.1 text and copyright notice

#### Scenario: OpenDyslexic license is present
- **WHEN** a user inspects the `licenses/` directory
- **THEN** a file named `LICENSE-OPENDYSLEXIC` contains the full SIL OFL 1.1 text and copyright notice

### Requirement: Icon license bundled
The repository SHALL contain the full ISC license text for the Lucide icons distributed in the `icons/` directory.

#### Scenario: Lucide license is present
- **WHEN** a user inspects the `licenses/` directory
- **THEN** a file named `LICENSE-LUCIDE` contains the full ISC license text and copyright notice

### Requirement: Third-party notices index all assets
The `THIRD_PARTY_NOTICES.md` file SHALL contain a section listing all bundled assets (fonts and icons), their authors, and the path to their corresponding license file.

#### Scenario: Asset attribution is complete
- **WHEN** a user opens `THIRD_PARTY_NOTICES.md`
- **THEN** it contains an "Assets" section
- **AND** that section lists Noto Sans, OpenDyslexic, and Lucide with author, license, and license file path
