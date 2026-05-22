use bluer::Address;

/// Simplified view of a Bluetooth device for UI display.
#[derive(Clone, Debug)]
pub struct BluetoothDevice {
    /// The Bluetooth MAC address.
    pub address: Address,
    /// Display name (alias if set, otherwise device name, otherwise MAC).
    pub display_name: String,
    /// Whether the device is paired.
    pub paired: bool,
    /// Whether the device is currently connected.
    pub connected: bool,
    /// Whether the device is trusted (auto-connect allowed).
    #[allow(dead_code)]
    pub trusted: bool,
}

/// Sort devices for display: paired first, then connected, then
/// alphabetically by display name (case-insensitive).
pub fn sort_devices(devices: &mut [BluetoothDevice]) {
    devices.sort_by(|a, b| {
        b.paired
            .cmp(&a.paired)
            .then(b.connected.cmp(&a.connected))
            .then_with(|| {
                a.display_name
                    .to_lowercase()
                    .cmp(&b.display_name.to_lowercase())
            })
    });
}
