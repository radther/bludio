# Bludio

A GPUI Bluetooth and audio device manager for Linux.

## Install

### Binary (recommended)

The fastest way to install Bludio — no Rust toolchain required.

```bash
curl -fsSL https://raw.githubusercontent.com/radther/bludio/main/install.sh | bash
```

For full functionality including privileged Bluetooth operations, run with `sudo`:

```bash
curl -fsSL https://raw.githubusercontent.com/radther/bludio/main/install.sh | sudo bash
```

> The install script detects your architecture, downloads the latest release, and sets up the app. Without `sudo`, the PolicyKit policy won't be installed and the Bluetooth restart feature will be unavailable.

### Manual download

If you prefer not to pipe to bash, download the latest release tarball for your architecture from [GitHub Releases](https://github.com/radther/bludio/releases), extract it, and copy the files to the appropriate locations:

```bash
# Example for v0.1.0 on x86_64
tar xzf bludio-v0.1.0-linux-x86_64.tar.gz
sudo cp bludio-v0.1.0-linux-x86_64/bludio /usr/local/bin/
sudo cp bludio-v0.1.0-linux-x86_64/bludio.desktop /usr/share/applications/
sudo cp bludio-v0.1.0-linux-x86_64/dev.toomosin.bludio.policy /usr/share/polkit-1/actions/
```

### Build from source

Requires the Rust toolchain:

```bash
cargo install --git https://github.com/radther/bludio
```

Or with `sudo` to also install the PolicyKit policy:

```bash
sudo cargo install --git https://github.com/radther/bludio
```

> If `sudo cargo` fails with "command not found", your `cargo` is in your user PATH but not root's. Use this instead:
> ```bash
> sudo -E env "PATH=$PATH" cargo install --git https://github.com/radther/bludio
> ```

## Supported Systems

Prebuilt binaries are compiled on Ubuntu 22.04 (glibc 2.35) and are compatible with:

- Ubuntu 22.04 LTS and newer
- Debian 11 (Bullseye) and newer
- Fedora 35 and newer
- Arch Linux and other rolling distributions

Users on older systems (Ubuntu 20.04 LTS, RHEL 8, etc.) should build from source with `cargo install --git`.

## System Requirements

Bludio requires the following system services to be running:

- **BlueZ / `bluetoothd`** — D-Bus Bluetooth daemon (bluer talks to this)
- **PulseAudio or PipeWire-Pulse** — Audio backend for device management
- **PolicyKit / Polkit** — Required for the privileged policy file to function
- **GPU with Vulkan support** — gpui-unofficial is GPU-accelerated via Vulkan
- **X11 or Wayland** — Display server (both are supported)

## Note on Pairing

Pairing requests are automatically accepted without user confirmation. This works well with devices that don't require authentication — most headphones, keyboards, speakers, etc. Devices that require a PIN or passkey may not pair, though some devices that support PIN entry can choose to skip authentication and pair without verification.

## Third-Party Notices

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for dependency and asset license information.
