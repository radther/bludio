## 1. GitHub Actions Release Workflow

- [x] 1.1 Create `.github/workflows/release.yml` with `on.push.tags: [ 'v*.*.*' ]` trigger
- [x] 1.2 Add `build-x86_64` job using `ubuntu-latest` runner with `ubuntu:22.04` container
- [x] 1.3 Install build dependencies in the container: `build-essential`, `pkg-config`, `libxcb1-dev`, `libxkbcommon-dev`, `libxkbcommon-x11-dev`, `libdbus-1-dev`, `libpulse-dev`, `libwayland-dev`, `libvulkan-dev`, `vulkan-validationlayers-dev`, `libx11-dev`
- [x] 1.4 Install Rust toolchain (`rustup`) inside the container matching the `rust-toolchain.toml` version (1.95.0)
- [x] 1.5 Build release binary with `cargo build --release --locked`
- [x] 1.6 Strip the binary and create tarball `bludio-${VERSION}-linux-x86_64.tar.gz` containing `bludio`, `dev.toomosin.bludio.policy`, and `bludio.desktop`
- [x] 1.7 Upload tarball as workflow artifact
- [x] 1.8 Add `build-aarch64` job using `ubuntu-24.04-arm` runner with identical container and steps
- [x] 1.9 Add `release` job that depends on both builds, downloads artifacts, and creates GitHub Release with `gh release create` (or `softprops/action-gh-release`)

## 2. Desktop Integration File

- [x] 2.1 Create `bludio.desktop` at repository root with `Name=Bludio`, `Exec=bludio`, `Type=Application`, `Categories=AudioVideo;Audio;Settings;`, and `Icon=bluetooth`
- [x] 2.2 Verify the `.desktop` file passes `desktop-file-validate` (if available)

## 3. Install Script

- [x] 3.1 Create `install.sh` as a POSIX-compliant `/bin/sh` script
- [x] 3.2 Detect architecture via `uname -m` with mapping: `x86_64` → `x86_64`, `aarch64` → `aarch64`, else error
- [x] 3.3 Fetch latest release tag via GitHub API: `curl -s https://api.github.com/repos/toomosin/bludio/releases/latest | grep '"tag_name":' | sed ...`
- [x] 3.4 Construct download URL: `https://github.com/toomosin/bludio/releases/download/${TAG}/bludio-${TAG}-linux-${ARCH}.tar.gz`
- [x] 3.5 Download tarball to a temporary directory and verify it exists
- [x] 3.6 Detect root: `if [ "$(id -u)" -eq 0 ]` → system install; else → user install
- [x] 3.7 System install path: copy binary to `/usr/local/bin/`, `.desktop` to `/usr/share/applications/`, policy to `/usr/share/polkit-1/actions/`
- [x] 3.8 User install path: copy binary to `~/.local/bin/` (create if missing), `.desktop` to `~/.local/share/applications/` (create if missing), skip policy with warning
- [x] 3.9 After user install, check if `~/.local/bin` is in `PATH`; if not, print shell-specific export instructions
- [x] 3.10 Clean up temporary files on success and on error (use `trap`)
- [x] 3.11 Print summary on completion: what was installed, where, and any warnings

## 4. Documentation

- [x] 4.1 Add an "Install (Binary)" section to `README.md` with the `curl | bash` command
- [x] 4.2 Add the `curl | sudo bash` variant for system-wide install with PolicyKit
- [x] 4.3 Add manual download instructions for users who prefer not to pipe to bash
- [x] 4.4 Document supported distros (Ubuntu 22.04+, Debian 11+, Fedora 35+, Arch, etc.)
- [x] 4.5 Document fallback: `cargo install --git` for older distros or unsupported architectures

## 5. Verification

- [x] 5.1 Test `cargo build --release --locked` in an `ubuntu:22.04` container locally (or via Actions test run)
- [x] 5.2 Verify the built binary runs on the host system after container build
- [x] 5.3 Test `install.sh` locally in a fresh environment (e.g., container or VM)
- [x] 5.4 Verify user install places files correctly and prints PATH warning when needed
- [x] 5.5 Verify system install places PolicyKit policy correctly
- [x] 5.6 Verify the `.desktop` file appears in the app launcher after install
- [x] 5.7 Tag a test release (`v0.1.0-test.1`) and confirm the workflow completes end-to-end
- [x] 5.8 Download and extract the published tarball, verify all three files are present
