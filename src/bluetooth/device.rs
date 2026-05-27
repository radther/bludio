//! BluetoothDevice data type, sorting, actions, and pairing status model.

use bluer::Address;
use std::fmt;

// ── Pairing status ────────────────────────────────────────────────────────

/// Status of an active pairing/connection operation shown on the device row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PairingStatus {
    /// Establishing the ACL connection.
    Connecting,
    /// Performing the pairing handshake (authentication via agent).
    Pairing,
    /// Setting the device as trusted.
    Trusting,
    /// The operation failed with the given reason.
    Failed(String),
}

impl fmt::Display for PairingStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PairingStatus::Connecting => write!(f, "connecting…"),
            PairingStatus::Pairing => write!(f, "pairing…"),
            PairingStatus::Trusting => write!(f, "trusting…"),
            PairingStatus::Failed(_) => write!(f, "failed to pair"),
        }
    }
}

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
    /// Active pairing operation status — `None` when idle.
    pub pairing_status: Option<PairingStatus>,
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
/// `pairing_status` is ignored — it's a transient display field that
/// shouldn't block UI updates.
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
// TODO: return Result<(), String> so callers can surface errors in the UI.
// Currently errors are only logged to stderr — the user sees no feedback.
pub(crate) async fn execute_device_action(
    adapter: &bluer::Adapter,
    addr: bluer::Address,
    action: DeviceRowAction,
) {
    match action {
        DeviceRowAction::Connect => {
            if let Ok(device) = adapter.device(addr)
                && let Err(e) = device.connect().await
            {
                eprintln!("[bluetooth] Connect failed for {addr}: {e}");
            }
        }
        DeviceRowAction::Disconnect => {
            if let Ok(device) = adapter.device(addr)
                && let Err(e) = device.disconnect().await
            {
                eprintln!("[bluetooth] Disconnect failed for {addr}: {e}");
            }
        }
        DeviceRowAction::Forget => {
            if let Err(e) = adapter.remove_device(addr).await {
                eprintln!("[bluetooth] Forget failed for {addr}: {e}");
            }
        }
        // PairAndTrust is handled stepwise in the command handler
        // (app.rs) with per-step UI status updates.
        DeviceRowAction::PairAndTrust => {
            // Unreachable — see command handler in app.rs.
            // Kept to satisfy exhaustive match.
        }
    }
}
