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
    let props = crate::backend::bluetooth::properties::fetch_properties(
        &device,
        crate::backend::bluetooth::properties::PropertyTimeouts::default(),
    )
    .await;
    crate::backend::bluetooth::properties::build_device(addr, &props, false, false)
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

// ── Error formatting ─────────────────────────────────────────────────────

/// Map BlueZ D-Bus errors to human-readable messages.
pub(crate) fn format_device_error(
    action: &str,
    addr: bluer::Address,
    error: &bluer::Error,
) -> String {
    let raw = error.to_string();
    let friendly = if raw.contains("NotAvailable") {
        "Device is out of range or not responding"
    } else if raw.contains("NotReady") {
        "Bluetooth adapter is not ready"
    } else if raw.contains("AlreadyConnected") {
        "Device is already connected"
    } else if raw.contains("NotConnected") {
        "Device is not connected"
    } else if raw.contains("InProgress") {
        "Another operation is in progress"
    } else if raw.contains("AuthenticationTimeout") {
        "Pairing timed out — try again"
    } else if raw.contains("AuthenticationRejected") {
        "Pairing was rejected by the device"
    } else if raw.contains("AuthenticationCanceled") {
        "Pairing was canceled"
    } else if raw.contains("ConnectionAttemptFailed") {
        "Connection attempt failed — device may be busy"
    } else {
        "Operation failed"
    };
    format!("{friendly} ({action} {addr}): {raw}")
}

// ── Device actions ────────────────────────────────────────────────────────

/// Execute a device action (connect / disconnect / forget / pair+trust)
/// on the current async runtime.
///
/// Returns `Ok(())` on success, or `Err(String)` with a human-readable
/// error message on failure.
pub(crate) async fn execute_device_action(
    adapter: &bluer::Adapter,
    addr: bluer::Address,
    action: DeviceRowAction,
) -> Result<(), String> {
    match action {
        DeviceRowAction::Connect => {
            let device = adapter
                .device(addr)
                .map_err(|e| format_device_error("connect", addr, &e))?;
            device
                .connect()
                .await
                .map_err(|e| format_device_error("connect", addr, &e))?;
            Ok(())
        }
        DeviceRowAction::Disconnect => {
            let device = adapter
                .device(addr)
                .map_err(|e| format_device_error("disconnect", addr, &e))?;
            device
                .disconnect()
                .await
                .map_err(|e| format_device_error("disconnect", addr, &e))?;
            Ok(())
        }
        DeviceRowAction::Forget => {
            adapter
                .remove_device(addr)
                .await
                .map_err(|e| format_device_error("forget", addr, &e))?;
            Ok(())
        }
        // PairAndTrust is handled stepwise in the command handler
        // (app.rs) with per-step UI status updates.
        DeviceRowAction::PairAndTrust => {
            // Unreachable — see command handler in app.rs.
            // Kept to satisfy exhaustive match.
            Ok(())
        }
    }
}
