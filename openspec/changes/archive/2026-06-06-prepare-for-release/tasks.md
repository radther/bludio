## 1. License & Legal Foundation

- [x] 1.1 Create `LICENSE` file with MIT license text and current year / copyright holder
- [x] 1.2 Create `THIRD_PARTY_NOTICES.md` with dependency license audit table
- [x] 1.3 Update `Cargo.toml` with `license = "MIT"`, `repository`, meaningful `description`, `keywords`, and `categories`

## 2. Asset Attribution

- [x] 2.1 Create `licenses/` directory at repository root
- [x] 2.2 Add `licenses/LICENSE-NOTO-SANS` with full SIL OFL 1.1 text and Noto Project copyright
- [x] 2.3 Add `licenses/LICENSE-OPENDYSLEXIC` with full SIL OFL 1.1 text and Abbie Gonzalez copyright
- [x] 2.4 Add `licenses/LICENSE-LUCIDE` with full ISC license text and Lucide copyright
- [x] 2.5 Update `THIRD_PARTY_NOTICES.md` with an "Assets" section indexing all fonts and icons with authors, licenses, and license file paths

## 3. Cargo Install & PolicyKit Integration

- [x] 3.1 Create `build.rs` that detects the PolicyKit actions directory (respecting `BLUDIO_POLICY_DIR` override and system prefix)
- [x] 3.2 Implement policy file copy in `build.rs` for Linux builds when the target directory is writable
- [x] 3.3 Implement graceful fallback in `build.rs` that prints a warning and skips the copy when running without privileges
- [x] 3.4 Verify `cargo install --git <repo-url>` compiles the binary successfully (from a temp directory, no local clone)
- [x] 3.5 Test policy file installation with `sudo cargo install --git <repo-url>` on a Linux system
- [x] 3.6 Verify non-root `cargo install --git <repo-url>` completes without error and shows the manual-install warning

## 4. Final Verification

- [x] 4.1 Run `cargo metadata` and confirm all required package fields are present and valid
- [x] 4.2 Confirm all new files (`LICENSE`, `THIRD_PARTY_NOTICES.md`, `build.rs`, `licenses/*`) are tracked by git
- [x] 4.3 Run `cargo clippy` and `cargo test` to ensure the new `build.rs` does not break existing builds
