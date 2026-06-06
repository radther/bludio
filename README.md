# Bludio

A GPUI Bluetooth and audio device manager for Linux.

## Install

```bash
cargo install --git https://github.com/radther/bludio
```

Or, with `sudo` to also install the PolicyKit policy file for privileged Bluetooth operations (restarting bluetooth):

```bash
sudo cargo install --git https://github.com/radther/bludio
```

> If `sudo cargo` fails with "command not found", your `cargo` is in your user PATH but not root's. Use this instead:
> ```bash
> sudo -E env "PATH=$PATH" cargo install --git https://github.com/radther/bludio
> ```

## PolicyKit

If you installed without root privileges, the PolicyKit policy file was not copied to the system directory automatically. To enable privileged Bluetooth operations, manually copy it:

```bash
sudo cp policy/dev.toomosin.bludio.policy /usr/share/polkit-1/actions/
```

Or re-run the install with `sudo`:

```bash
sudo cargo install --git https://github.com/radther/bludio
```

## System Requirements

Bludio requires the following system services to be running. These are **not** installed by `cargo install`:

- **BlueZ / `bluetoothd`** — D-Bus Bluetooth daemon (bluer talks to this)
- **PulseAudio** — Audio backend for device management
- **PolicyKit / Polkit** — Required for the privileged policy file to function
- **GPU with Vulkan support** — gpui-unofficial is GPU-accelerated via Vulkan
- **X11 or Wayland** — Display server (both are supported)

## Note on Pairing

Pairing requests are automatically accepted without user confirmation. This works well with devices that don't require authentication — most headphones, keyboards, speakers, etc. Devices that require a PIN or passkey to pair are not currently supported.

## Third-Party Notices

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for dependency and asset license information.
