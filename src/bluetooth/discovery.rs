use super::device;
use super::properties::{self, PropertyTimeouts};
use bluer::Adapter;

// ── Discovery events ───────────────────────────────────────────────────────

pub(crate) enum DiscoveryEvent {
    DeviceAdded(device::BluetoothDevice),
    Done,
}

// ── Run a scan ─────────────────────────────────────────────────────────────

/// Start device discovery on the Tokio runtime, stream discovered devices
/// back through an mpsc channel, and stop after 30s or when the channel
/// receiver drops.
pub(crate) fn start_scan(
    adapter: &Adapter,
) -> tokio::sync::mpsc::UnboundedReceiver<DiscoveryEvent> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let a = adapter.clone();

    tokio::spawn(async move {
        let mut events = match a.discover_devices().await {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Discovery error: {}", e);
                let _ = tx.send(DiscoveryEvent::Done);
                return;
            }
        };

        use futures::StreamExt;
        let deadline =
            tokio::time::Instant::now() + std::time::Duration::from_secs(30);

        loop {
            if tokio::time::Instant::now() >= deadline {
                let _ = tx.send(DiscoveryEvent::Done);
                break;
            }

            match tokio::time::timeout(
                std::time::Duration::from_millis(250),
                events.next(),
            )
            .await
            {
                Ok(Some(bluer::AdapterEvent::DeviceAdded(addr))) => {
                    if let Some(info) = fetch_device_info(&a, addr).await {
                        let _ = tx.send(DiscoveryEvent::DeviceAdded(info));
                    }
                }
                Ok(Some(bluer::AdapterEvent::DeviceRemoved(_))) => {}
                Ok(None) => {
                    let _ = tx.send(DiscoveryEvent::Done);
                    break;
                }
                Err(_timeout) => {}
                _ => {}
            }
        }
    });

    rx
}

// ── Device info for scan ───────────────────────────────────────────────────

/// Fetch device info with RSSI + name filtering for scan results.
/// Returns `None` for devices not in range or without a resolved name.
async fn fetch_device_info(
    adapter: &Adapter,
    addr: bluer::Address,
) -> Option<device::BluetoothDevice> {
    let device = adapter.device(addr).ok()?;
    let props =
        properties::fetch_properties(&device, PropertyTimeouts::SCAN).await;
    properties::build_device(addr, &props, true, true)
}

// ── Full list refresh ──────────────────────────────────────────────────────

/// Full device list refresh: enumerate all known devices, filter by RSSI
/// (in-range only, unless paired) and name (blueman default), then sort.
pub(crate) async fn refresh_device_list(
    adapter: &Adapter,
) -> Option<Vec<device::BluetoothDevice>> {
    let addresses = adapter.device_addresses().await.ok()?;
    let mut devices = Vec::new();

    for addr in addresses {
        let device = adapter.device(addr).ok()?;
        let props =
            properties::fetch_properties(&device, PropertyTimeouts::default()).await;

        if !props.paired && props.rssi.is_none() {
            continue;
        }

        let Some(dev) = properties::build_device(addr, &props, true, false) else {
            continue;
        };
        devices.push(dev);
    }

    device::sort_devices(&mut devices);
    Some(devices)
}
