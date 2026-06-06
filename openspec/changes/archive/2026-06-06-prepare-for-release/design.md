## Context

Bludio is a Rust GUI application built on `gpui-unofficial` (Zed's GPUI framework) that manages Bluetooth and PulseAudio devices via D-Bus. It is nearing an Alpha release and currently lacks:
- A project-wide open-source license
- A dependency license audit
- Bundled license files for third-party fonts and icons
- A working `cargo install` path that places the required PolicyKit policy file

The project currently has a `policy/dev.toomosin.bludio.policy` file that defines a privileged action for restarting the Bluetooth service, but there is no automated mechanism to install this file into `/usr/share/polkit-1/actions/` when a user runs `cargo install`.

## Goals / Non-Goals

**Goals:**
- Choose and apply an MIT license to Bludio itself.
- Audit all direct dependency licenses and confirm MIT compatibility.
- Bundle full license texts and copyright notices for all third-party assets (Noto Sans, OpenDyslexic, Lucide icons) in the repository.
- Enable `cargo install --git <repo-url>` to build and install the binary directly from GitHub without manual cloning, and copy the PolicyKit policy file to the system directory when running with appropriate privileges.
- Update `Cargo.toml` with crates.io-ready metadata.

**Non-Goals:**
- Creating `.deb`, `.rpm`, or Flatpak/Snap packages (out of scope for Alpha).
- Setting up CI/CD pipelines or automated releases.
- Creating a project website or documentation site.
- Modifying any runtime application behavior.

## Decisions

### MIT for Bludio itself
- **Rationale**: MIT is maximally permissive, aligns with the user's preference, and is compatible with every dependency in the tree.
- **Compatibility check**:
  | Dependency | License | MIT Compatible? |
  |------------|---------|-----------------|
  | `gpui-unofficial` | Apache-2.0 | Yes |
  | `gpui-platform-gpui-unofficial` | Apache-2.0 | Yes |
  | `bluer` | BSD-2-Clause | Yes |
  | `tokio` | MIT | Yes |
  | `futures` | MIT / Apache-2.0 | Yes |
  | `libpulse-binding` | MIT / Apache-2.0 | Yes |
  | `libpulse-sys` | MIT / Apache-2.0 | Yes |
  | `serde`, `serde_json` | MIT / Apache-2.0 | Yes |
  | `unicode-segmentation` | MIT / Apache-2.0 | Yes |
- No GPL, LGPL, or copyleft dependencies in the direct tree. The underlying PulseAudio system library is LGPL, but we only link to it dynamically at runtime; the Rust bindings themselves are MIT/Apache-2.0.

### Bundle full license texts in a `licenses/` directory
- **Rationale**: OFL (fonts) and ISC (icons) both require preserving copyright notices and license text when redistributing the assets. A central `licenses/` directory is cleaner than scattering files, and `THIRD_PARTY_NOTICES.md` at the root can index them.
- **Alternative considered**: Embedding notices only in metadata. Rejected because binary font metadata is hard for end users to discover; explicit text files are clearer.

### Use `build.rs` for PolicyKit policy installation
- **Rationale**: `cargo install --git` has no built-in mechanism for installing auxiliary system files. A `build.rs` script can detect when running under `cargo install` (via `CARGO_TARGET_DIR` or profile) and copy the `.policy` file to the appropriate `datadir` path (`/usr/share/polkit-1/actions/` or `$PREFIX/share/polkit-1/actions/`). The policy file is read from the source tree via `CARGO_MANIFEST_DIR`, which works identically for `--git` and `--path` installs.
- **Alternative considered**: A separate `install.sh` script. Rejected because `cargo install --git` is the standard Rust workflow and users expect it to work standalone without manual cloning.
- **Caveat**: Writing to `/usr/share/` requires root. The `build.rs` will gracefully skip the copy if the target directory is not writable, and print a message telling the user to manually copy the file or re-run with `sudo`.

## Risks / Trade-offs

- **[Risk]** `build.rs` running during normal `cargo build` may attempt to write system files.  
  → **Mitigation**: Only attempt the copy when `CARGO_INSTALL_ROOT` or a `POLICY_INSTALL_DIR` env var is present, or when the profile is `release` and the target path is under a system prefix. Default `cargo build` does nothing.

- **[Risk]** `cargo install --git` from a non-root user will not install the policy file, and the app will lack the privileged action.  
  → **Mitigation**: Print a clear warning during build and include manual installation instructions in `README.md` and `THIRD_PARTY_NOTICES.md`.

- **[Risk]** The `build.rs` approach is platform-specific to Linux/Polkit.  
  → **Mitigation**: Acceptable for now; the app is already Linux-only (BlueZ/PulseAudio). Guard the copy with `cfg(target_os = "linux")`.

## Migration Plan

N/A — this is a new-project readiness change, not a migration of existing deployed code.

## Open Questions

1. Should we also support an environment variable (e.g. `BLUDIO_POLICY_DIR`) so packagers can override the install path? → *Decision: Yes, implement in `build.rs`.*
2. Do we need to handle the case where the policy file is already present (overwrite vs skip)? → *Decision: Overwrite if newer, otherwise skip.*
