## Context

Bludio is a Rust GUI application using `gpui-unofficial` and `bluer` that manages Bluetooth and PulseAudio devices. The project has reached Alpha readiness with a working `cargo install --git` path (via the `prepare-for-release` change). However, requiring the Rust toolchain is a significant barrier for end users.

The current binary is a 39MB dynamically-linked ELF (31MB stripped) that depends on system libraries: X11/XCB, xkbcommon, D-Bus, PulseAudio, glibc, and GPU/Vulkan drivers. These are standard on any Linux desktop and are not bundled.

## Goals / Non-Goals

**Goals:**
- Provide a single-command installation path (`curl | bash`) that does not require Rust.
- Build release binaries for `x86_64` and `aarch64` via GitHub Actions on every version tag.
- Target glibc 2.35 (Ubuntu 22.04 baseline) for broad compatibility.
- Package each release as a tarball containing the binary, PolicyKit policy, and `.desktop` entry.
- Publish tarballs to GitHub Releases automatically.
- Handle both system-wide (with root) and user-only (without root) installations.

**Non-Goals:**
- Flatpak, Snap, .deb, .rpm, or AUR packaging.
- Windows or macOS builds.
- Auto-update mechanism within the app.
- Code signing or notarization.

## Decisions

### Build on `ubuntu:22.04` instead of manylinux
- **Rationale**: `manylinux_2_28` containers ship old GCC and system packages that may fail to compile gpui-unofficial's dependencies (wgpu, wayland-rs, x11rb). Ubuntu 22.04 provides glibc 2.35 with modern enough build tools. The compatibility trade-off is acceptable: Ubuntu 22.04+ (2022), Debian 11+ (2021), Fedora 35+ (2021), and all rolling distros are covered. Users on older LTS (Ubuntu 20.04, RHEL 8) are a small and shrinking segment of desktop Linux users.
- **Alternative considered**: `manylinux_2_28` for broader compatibility. Rejected due to uncertainty about whether gpui-unofficial's C++ dependencies (Vulkan, XCB) compile with the older toolchain.

### GitHub-hosted runners for both architectures
- **Rationale**: GitHub provides free `ubuntu-latest` (x86_64) and `ubuntu-24.04-arm` (aarch64) runners for public repositories. No self-hosted infrastructure needed.
- **Container strategy**: Run the build inside `ubuntu:22.04` Docker containers on both runners to ensure identical environments regardless of the host runner's OS version.

### Release tarball structure
- **Rationale**: A single tarball per architecture is the simplest distribution format. It includes everything needed: binary, policy file, and `.desktop` entry. The install script unpacks it and places files correctly.
- **Structure**:
  ```
  bludio-0.1.0-linux-x86_64/
  ├── bludio
  ├── dev.toomosin.bludio.policy
  └── bludio.desktop
  ```

### Install script with privilege detection
- **Rationale**: The PolicyKit policy file must reside in `/usr/share/polkit-1/actions/` (root-only). The script detects if it has root and chooses between system-wide and user-only installation. This matches the current `cargo install` behavior (works without root, but warns about missing privileges).
- **User install path**: Binary to `~/.local/bin/`, `.desktop` to `~/.local/share/applications/`. No PolicyKit policy — app runs but privileged Bluetooth restart is unavailable.
- **System install path**: Binary to `/usr/local/bin/`, `.desktop` system-wide, PolicyKit policy to `/usr/share/polkit-1/actions/`.

### `curl | bash` from raw GitHub URL
- **Rationale**: The install script lives in the repository root and is served via `raw.githubusercontent.com`. This is the standard pattern used by rustup, nvm, and many other projects. Users trust it because the script is auditable (visible in the repo).
- **Alternative considered**: Dedicated download domain. Rejected as unnecessary overhead for an Alpha release.

## Risks / Trade-offs

- **[Risk]** Building on Ubuntu 22.04 (glibc 2.35) excludes Ubuntu 20.04 LTS and RHEL 8 users.
  → **Mitigation**: Document that these users should use `cargo install --git` instead. The Rust toolchain path remains available.
- **[Risk]** GitHub Actions free runners have queue time and 6-hour job limits.
  → **Mitigation**: The build is expected to take ~10-15 minutes. Well within limits. Document that first-time builds compile all dependencies.
- **[Risk]** `curl | bash` feels unsafe to some users.
  → **Mitigation**: Provide manual download instructions in README. The script is short, readable, and in the public repo.
- **[Risk]** Aarch64 build may fail or be slow on GitHub's ARM runners.
  → **Mitigation**: Start with x86_64 only if aarch64 proves problematic. The workflow design makes it easy to add aarch64 later.
- **[Risk]** The `.desktop` file needs an icon path that works for both system and user installs.
  → **Mitigation**: Use a relative icon name (`Icon=bludio`) and rely on the icon being installed in the standard hicolor theme path. For Alpha, the app may not ship a custom icon, so a generic Bluetooth icon (`Icon=bluetooth`) is acceptable.
