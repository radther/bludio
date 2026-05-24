use bluer::Device;
use std::time::Duration;

use super::device::BluetoothDevice;

/// Timeout configuration for `fetch_properties`.
#[derive(Clone, Copy)]
pub(crate) struct PropertyTimeouts {
    /// Timeout for alias / name resolution (SDP / EIR lookup).
    pub name_resolution: Duration,
    /// Timeout for paired / connected / trusted queries.
    pub status: Duration,
    /// Timeout for RSSI query.
    pub rssi: Duration,
}

impl Default for PropertyTimeouts {
    fn default() -> Self {
        Self {
            name_resolution: Duration::from_millis(500),
            status: Duration::from_millis(500),
            rssi: Duration::from_secs(1),
        }
    }
}

impl PropertyTimeouts {
    /// Longer timeouts for scan — `BlueZ` needs time for SDP lookups.
    pub(crate) const SCAN: Self = Self {
        name_resolution: Duration::from_millis(1500),
        status: Duration::from_millis(500),
        rssi: Duration::from_secs(1),
    };
}

/// Raw device properties fetched from `BlueZ`.
pub(crate) struct DeviceProperties {
    pub alias: Option<String>,
    pub name: Option<String>,
    pub paired: bool,
    pub connected: bool,
    pub trusted: bool,
    pub rssi: Option<i16>,
}

/// Fetch all device properties with per-category timeouts.
pub(crate) async fn fetch_properties(
    device: &Device,
    timeouts: PropertyTimeouts,
) -> DeviceProperties {
    let alias = tokio::time::timeout(timeouts.name_resolution, device.alias())
        .await
        .ok()
        .and_then(std::result::Result::ok);

    let name = tokio::time::timeout(timeouts.name_resolution, device.name())
        .await
        .ok()
        .and_then(std::result::Result::ok)
        .flatten();

    let paired = tokio::time::timeout(timeouts.status, device.is_paired())
        .await
        .ok()
        .and_then(std::result::Result::ok)
        .unwrap_or(false);

    let connected = tokio::time::timeout(timeouts.status, device.is_connected())
        .await
        .ok()
        .and_then(std::result::Result::ok)
        .unwrap_or(false);

    let trusted = tokio::time::timeout(timeouts.status, device.is_trusted())
        .await
        .ok()
        .and_then(std::result::Result::ok)
        .unwrap_or(false);

    let rssi = tokio::time::timeout(timeouts.rssi, device.rssi())
        .await
        .ok()
        .and_then(std::result::Result::ok)
        .flatten();

    DeviceProperties {
        alias,
        name,
        paired,
        connected,
        trusted,
        rssi,
    }
}

/// Build a `BluetoothDevice` from properties, applying name/RSSI filtering.
/// Returns `None` if the device should be hidden.
pub(crate) fn build_device(
    address: bluer::Address,
    props: &DeviceProperties,
    require_name: bool,
    require_rssi: bool,
) -> Option<BluetoothDevice> {
    if require_rssi && props.rssi.is_none() && !props.paired {
        return None;
    }

    let addr_str = address.to_string();
    let display_name = match super::resolve_display_name(
        props.name.clone(),
        props.alias.clone(),
        props.paired,
        &addr_str,
    ) {
        Some(n) => n,
        None if require_name => return None,
        None => addr_str,
    };

    Some(BluetoothDevice {
        address,
        display_name,
        paired: props.paired,
        connected: props.connected,
        trusted: props.trusted,
        pairing_status: None,
    })
}
