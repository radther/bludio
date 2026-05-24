use super::device;
use super::properties::{self, PropertyTimeouts};
use bluer::Adapter;
use futures::StreamExt;
use futures::channel::mpsc;
use gpui::{AsyncWindowContext, WeakEntity};

// ── Discovery events ───────────────────────────────────────────────────────

pub(crate) enum DiscoveryEvent {
    DeviceAdded(device::BluetoothDevice),
    Done,
}

// ── Run a scan ─────────────────────────────────────────────────────────────

/// Start device discovery on the Tokio runtime, stream discovered devices
/// back through a futures mpsc channel, and stop after 30s or when the
/// channel receiver drops.
pub(crate) fn start_scan(adapter: &Adapter) -> mpsc::UnboundedReceiver<DiscoveryEvent> {
    let (tx, rx) = mpsc::unbounded();
    let a = adapter.clone();

    crate::TOKIO.spawn(async move {
        let mut events = match a.discover_devices().await {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Discovery error: {}", e);
                let _ = tx.unbounded_send(DiscoveryEvent::Done);
                return;
            }
        };

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);

        loop {
            if tokio::time::Instant::now() >= deadline {
                let _ = tx.unbounded_send(DiscoveryEvent::Done);
                break;
            }

            match tokio::time::timeout(std::time::Duration::from_millis(250), events.next()).await {
                Ok(Some(bluer::AdapterEvent::DeviceAdded(addr))) => {
                    if let Some(info) = fetch_device_info(&a, addr).await {
                        let _ = tx.unbounded_send(DiscoveryEvent::DeviceAdded(info));
                    }
                }
                Ok(Some(bluer::AdapterEvent::DeviceRemoved(_))) => {}
                Ok(None) => {
                    let _ = tx.unbounded_send(DiscoveryEvent::Done);
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
    let props = properties::fetch_properties(&device, PropertyTimeouts::SCAN).await;
    properties::build_device(addr, &props, true, true)
}

// ── Full list refresh ──────────────────────────────────────────────────────

/// Full device list refresh: enumerate all known devices, filter by RSSI
/// (in-range only, unless paired) and name (blueman default), then sort.
pub(crate) async fn refresh_device_list(adapter: &Adapter) -> Option<Vec<device::BluetoothDevice>> {
    let addresses = adapter.device_addresses().await.ok()?;
    let mut devices = Vec::new();

    for addr in addresses {
        let device = adapter.device(addr).ok()?;
        let props = properties::fetch_properties(&device, PropertyTimeouts::default()).await;

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

// ── Discovery orchestration ────────────────────────────────────────────────

/// Run a device discovery session, pushing updates to the app entity.
/// Stops when `discovering` flag is cleared or the scan completes.
pub(crate) async fn run_discovery(
    this: WeakEntity<crate::app::BludioApp>,
    cx: &mut AsyncWindowContext,
) {
    let adapter = match this.read_with(cx, |app, _| app.bt_state.adapter.clone()) {
        Ok(Some(a)) => a,
        _ => return,
    };

    let mut rx = start_scan(&adapter);

    while let Some(event) = rx.next().await {
        match event {
            DiscoveryEvent::DeviceAdded(device) => {
                let _ = this.update_in(cx, |this, window, cx| {
                    this.bt_state.upsert_device(device);
                    this.bluetooth_page
                        .update(cx, |page, cx| page.sync_state(&this.bt_state, window, cx));
                    cx.notify();
                });
                if !this
                    .read_with(cx, |app, _| app.bt_state.discovering)
                    .unwrap_or(false)
                {
                    break;
                }
            }
            DiscoveryEvent::Done => break,
        }
    }

    let a = adapter;
    if let Ok(Some(devices)) = crate::tokio_task(async move { refresh_device_list(&a).await }).await
    {
        let _ = this.update_in(cx, |this, window, cx| {
            this.bt_state.replace_devices(devices);
            this.bt_state.discovering = false;
            this.bluetooth_page
                .update(cx, |page, cx| page.sync_state(&this.bt_state, window, cx));
            cx.notify();
        });
        return;
    }

    let _ = this.update_in(cx, |this, window, cx| {
        this.bt_state.discovering = false;
        this.bluetooth_page
            .update(cx, |page, cx| page.sync_state(&this.bt_state, window, cx));
        cx.notify();
    });
}
