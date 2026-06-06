## Why

Bludio currently requires users to install Rust and use `cargo install --git` to obtain the app. This is a significant barrier for end users who are not Rust developers. We need a frictionless, single-command installation path that works on any modern Linux system without requiring the Rust toolchain or external package managers like Flatpak.

## What Changes

- Add a GitHub Actions workflow that builds release binaries for `x86_64` and `aarch64` Linux on every version tag push.
- Build inside an `ubuntu:22.04` container to target glibc 2.35, ensuring compatibility with Ubuntu 22.04+, Debian 11+, Fedora 35+, and all newer distros.
- Produce architecture-specific tarballs containing the stripped binary, PolicyKit policy file, and a `.desktop` entry.
- Publish release artifacts to GitHub Releases automatically.
- Create an `install.sh` script that detects architecture, downloads the correct tarball, installs the binary and PolicyKit policy (when run with privileges), and sets up desktop integration.
- Update `README.md` with the new installation instructions.

## Capabilities

### New Capabilities
- `github-release-binary`: Build, package, and publish precompiled Linux binaries via GitHub Actions and GitHub Releases.
- `install-script`: Single-command `curl | bash` installer that handles architecture detection, download, binary placement, PolicyKit policy installation, and `.desktop` registration.

### Modified Capabilities
<!-- No existing spec-level requirements are changing. This is a pure packaging/distribution change with no runtime behavior modifications. -->

## Impact

- New files: `.github/workflows/release.yml`, `install.sh`, `bludio.desktop`.
- Modified files: `README.md`.
- No runtime code changes; this is a CI/CD and distribution change.
