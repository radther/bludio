//! Bluetooth state management: session, adapter, device list, blueman-parity filtering.

pub mod agent;
pub mod device;
pub mod discovery;
pub(crate) mod monitor;
pub(crate) mod properties;

use crate::subsystem::SubsystemStatus;
use device::BluetoothDevice;

/// Holds the Bluetooth session and adapter state shared with the UI.
#[derive(Clone)]
pub struct BluetoothState {
    /// The `BlueZ` session connection (None until initialized).
    pub session: Option<bluer::Session>,
    /// Handle to the default Bluetooth adapter (None until initialized).
    pub adapter: Option<bluer::Adapter>,
    /// Whether a device discovery scan is currently active.
    pub discovering: bool,
    /// List of known Bluetooth devices (sorted: paired → connected → alpha).
    pub devices: Vec<BluetoothDevice>,
    /// Unified subsystem health status.
    pub subsystem_status: SubsystemStatus,
}

impl Default for BluetoothState {
    fn default() -> Self {
        Self {
            session: None,
            adapter: None,
            discovering: false,
            devices: Vec::new(),
            subsystem_status: SubsystemStatus::Connecting,
        }
    }
}

impl BluetoothState {
    /// Connect to `BlueZ`, get the default adapter, power on, and enumerate
    /// currently paired devices.
    pub async fn new() -> Result<Self, String> {
        let session = bluer::Session::new()
            .await
            .map_err(|e| format!("Failed to connect to BlueZ: {e}"))?;

        let adapter = session
            .default_adapter()
            .await
            .map_err(|e| format!("No Bluetooth adapter found: {e}"))?;

        adapter
            .set_powered(true)
            .await
            .map_err(|e| format!("Failed to power on adapter: {e}"))?;

        adapter
            .set_pairable(true)
            .await
            .map_err(|e| format!("Failed to set pairable: {e}"))?;

        let mut state = Self {
            session: Some(session),
            adapter: Some(adapter),
            discovering: false,
            devices: Vec::new(),
            subsystem_status: SubsystemStatus::Connected,
        };

        state.list_devices().await?;

        Ok(state)
    }

    /// Re-enumerate all known devices from the adapter.
    ///
    /// Fetches properties for all devices concurrently via `join_all`.
    /// Since each device's D-Bus calls are also concurrent (`fetch_properties`
    /// uses `tokio::join!`), the total wait time is bounded by the slowest
    /// single property call across all devices rather than the sum of all.
    pub async fn list_devices(&mut self) -> Result<(), String> {
        let adapter = self
            .adapter
            .as_ref()
            .ok_or_else(|| "BluetoothState not initialized".to_string())?;

        let addresses = adapter
            .device_addresses()
            .await
            .map_err(|e| format!("Failed to list devices: {e}"))?;

        // Fire all device-info futures concurrently.
        let results: Vec<Result<BluetoothDevice, String>> = futures::future::join_all(
            addresses
                .iter()
                .map(|&addr| build_device_info(adapter, addr)),
        )
        .await;

        let mut devices = Vec::new();
        for result in results {
            match result {
                Ok(d) => devices.push(d),
                Err(e) => eprintln!("Skipping device: {e}"),
            }
        }
        device::sort_devices(&mut devices);
        self.devices = devices;
        Ok(())
    }

    /// Replace the entire device list (keeps it sorted).
    /// Clears any stale `pairing_status` — a full refresh means
    /// no operation is still in progress.
    pub fn replace_devices(&mut self, mut devices: Vec<BluetoothDevice>) {
        for d in &mut devices {
            d.pairing_status = None;
        }
        self.devices = devices;
        device::sort_devices(&mut self.devices);
    }

    /// Insert or update a device, then re-sort.
    /// Preserves an existing `pairing_status` if the incoming device
    /// has `None` — single-device refreshes should not clear the indicator.
    pub fn upsert_device(&mut self, mut device: BluetoothDevice) {
        if let Some(existing) = self
            .devices
            .iter_mut()
            .find(|d| d.address == device.address)
        {
            // Preserve existing pairing status; prefer the incoming one if set.
            let new_status = device
                .pairing_status
                .take()
                .or_else(|| existing.pairing_status.take());
            *existing = device;
            existing.pairing_status = new_status;
        } else {
            self.devices.push(device);
        }
        device::sort_devices(&mut self.devices);
    }

    /// Remove a device by address (no re-sort needed).
    // Part of the public BluetoothState API — used when devices are explicitly removed.
    #[allow(dead_code)]
    pub fn remove_device(&mut self, addr: bluer::Address) {
        self.devices.retain(|d| d.address != addr);
    }
}

/// Build a `BluetoothDevice` from an address. Returns `Err` for devices
/// that should be hidden (unnamed + unpaired).
async fn build_device_info(
    adapter: &bluer::Adapter,
    address: bluer::Address,
) -> Result<BluetoothDevice, String> {
    let device = adapter
        .device(address)
        .map_err(|e| format!("Device error: {e}"))?;

    let props =
        properties::fetch_properties(&device, properties::PropertyTimeouts::default()).await;
    properties::build_device(address, &props, true, false).ok_or_else(|| String::from("unnamed"))
}

/// Resolve a human-readable display name for a Bluetooth device.
///
/// Logic (matching blueman's default):
/// 1. Broadcast name (D-Bus `Name` property) → use it if present
/// 2. Alias that differs from the raw MAC address → use alias
/// 3. Paired device with no name → fall back to MAC address
/// 4. Unpaired device with no name → return None (hide it)
///
/// The MAC comparison normalises dash/colon separators and is case-insensitive,
/// matching blueman's `alias.replace("-", ":") == address` check.
pub fn resolve_display_name(
    name: Option<String>,
    alias: Option<String>,
    paired: bool,
    addr_str: &str,
) -> Option<String> {
    if let Some(n) = name.filter(|n| !n.is_empty()) {
        Some(n)
    } else {
        let alias_matches_mac = alias
            .as_ref()
            .is_some_and(|a| a.replace('-', ":").to_lowercase() == addr_str.to_lowercase());
        if alias_matches_mac {
            if paired { Some(addr_str.into()) } else { None }
        } else {
            match alias {
                Some(a) if !a.is_empty() => Some(a),
                _ if paired => Some(addr_str.into()),
                _ => None,
            }
        }
    }
}
