## Why

Bludio is approaching an Alpha release and must be made publicly installable and distributable. Before publishing, we need to: select a project license compatible with all dependencies, ensure third-party assets (fonts, icons) include required attribution and license files, and provide a working installation path that also places the PolicyKit policy file in the correct system location. Without these, the project cannot be shared, installed, or legally distributed.

## What Changes

- Add a top-level `LICENSE` file choosing the MIT license for Bludio itself.
- Audit all direct and transitive dependency licenses for MIT compatibility.
- Create a `THIRD_PARTY_NOTICES.md` file documenting dependency and asset licenses.
- Include full license texts and copyright notices for bundled assets:
  - Noto Sans fonts (SIL Open Font License 1.1)
  - OpenDyslexic fonts (SIL Open Font License 1.1)
  - Lucide icons (ISC License)
- Make the app installable directly from the GitHub URL via `cargo install --git <repo-url>` by:
  - Adding a `build.rs` that copies the PolicyKit `.policy` file into the appropriate system directory during install.
  - Updating `Cargo.toml` with metadata required for crates.io publishing (description, license, repository, keywords, categories).
- Update `Cargo.toml` `description` field to reflect the actual application purpose.

## Capabilities

### New Capabilities
- `release-license`: Select MIT as the project license, verify dependency license compatibility, and establish `LICENSE` and `THIRD_PARTY_NOTICES.md`.
- `asset-attribution`: Bundle required license files and copyright notices for all third-party fonts and icons distributed with the application.
- `cargo-install`: Enable installation via `cargo install`, including automatic placement of the PolicyKit policy file.

### Modified Capabilities
<!-- No existing spec-level requirements are changing. -->

## Impact

- `Cargo.toml`: Updated package metadata and new `build.rs` integration.
- New files: `LICENSE`, `THIRD_PARTY_NOTICES.md`, `build.rs`.
- New directories: `licenses/` (for full third-party license texts), `THIRD_PARTY_NOTICES.md` at repo root.
- `policy/`: The existing `dev.toomosin.bludio.policy` file path may be referenced in `build.rs`; no content changes to the policy itself.
- No runtime code changes; this is a packaging and legal-readiness change.
