use bluer::Address;

// ── Action types ────────────────────────────────────────────────────────────

/// Action types for device buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeviceRowAction {
    Connect,
    Disconnect,
    Forget,
    PairAndTrust,
}

// ── Bluetooth device ───────────────────────────────────────────────────────

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

// ── Device actions ────────────────────────────────────────────────────────

/// Minimal status fetch for immediate UI updates — no filtering.
pub(crate) async fn quick_device_status(
    adapter: &bluer::Adapter,
    addr: bluer::Address,
) -> Option<BluetoothDevice> {
    let device = adapter.device(addr).ok()?;
    let props = crate::bluetooth::properties::fetch_properties(
        &device,
        crate::bluetooth::properties::PropertyTimeouts::default(),
    )
    .await;
    crate::bluetooth::properties::build_device(addr, &props, false, false)
}

/// Compare two device lists — returns `true` if display-relevant fields
/// (count, address, paired, connected) differ.
pub(crate) fn devices_changed(old: &[BluetoothDevice], new: &[BluetoothDevice]) -> bool {
    if old.len() != new.len() {
        return true;
    }
    for (a, b) in old.iter().zip(new.iter()) {
        if a.address != b.address || a.paired != b.paired || a.connected != b.connected {
            return true;
        }
    }
    false
}

/// Execute a device action (connect / disconnect / forget / pair+trust)
/// on the current async runtime.
pub(crate) async fn execute_device_action(
    adapter: &bluer::Adapter,
    addr: bluer::Address,
    action: DeviceRowAction,
) {
    match action {
        DeviceRowAction::Connect => {
            if let Ok(device) = adapter.device(addr) {
                let _ = device.connect().await;
            }
        }
        DeviceRowAction::Disconnect => {
            if let Ok(device) = adapter.device(addr) {
                let _ = device.disconnect().await;
            }
        }
        DeviceRowAction::Forget => {
            let _ = adapter.remove_device(addr).await;
        }
        DeviceRowAction::PairAndTrust => {
            if let Ok(device) = adapter.device(addr) {
                let _ = device.pair().await;
                let _ = device.set_trusted(true).await;
                let _ = device.connect().await;
            }
        }
    }
}
