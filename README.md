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

## Third-Party Notices

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for dependency and asset license information.
