//! Background D-Bus property-change monitoring via per-device event listeners
//! and adapter-level property monitoring.

use std::collections::HashMap;

use bluer::{Adapter, AdapterEvent, AdapterProperty, DeviceProperty};
use futures::{StreamExt, channel::mpsc};
use tokio::task::JoinHandle;

/// Event from the monitor subsystem.
pub(crate) enum MonitorEvent {
    /// A device property changed. Contains the device address.
    DeviceChanged(bluer::Address),
    /// The adapter was powered off (e.g., rfkill block).
    AdapterPoweredOff,
    /// The adapter was powered on (e.g., rfkill unblock).
    AdapterPoweredOn,
}

/// Start background D-Bus property-change monitoring.
///
/// Spawns per-device event listeners that emit `DeviceChanged` whenever
/// `Connected`, `Paired`, `Name`, or `Alias` changes.
/// Also monitors adapter-level properties (specifically `Powered`) to
/// detect rfkill block/unblock events reactively.
/// Re-syncs the listener set every 5 seconds to catch new/removed devices.
pub(crate) async fn run_monitor(adapter: &Adapter, tx: mpsc::UnboundedSender<MonitorEvent>) {
    let mut handles: HashMap<bluer::Address, JoinHandle<()>> = HashMap::new();
    let a = adapter.clone();

    // Spawn adapter-level event listener for Powered changes.
    let adapter_tx = tx.clone();
    let adapter_clone = adapter.clone();
    let _adapter_listener = tokio::spawn(async move {
        let mut events = match adapter_clone.events().await {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[monitor] Failed to get adapter events: {e}");
                return;
            }
        };
        while let Some(event) = events.next().await {
            if let AdapterEvent::PropertyChanged(AdapterProperty::Powered(powered)) = event {
                let result = if powered {
                    adapter_tx.unbounded_send(MonitorEvent::AdapterPoweredOn)
                } else {
                    adapter_tx.unbounded_send(MonitorEvent::AdapterPoweredOff)
                };
                if result.is_err() {
                    eprintln!("[monitor] Adapter event send failed, receiver dropped");
                    return;
                }
            }
        }
    });

    loop {
        match a.device_addresses().await {
            Ok(addresses) => {
                handles.retain(|addr, _| addresses.contains(addr));

                for addr in addresses {
                    if handles.contains_key(&addr) {
                        continue;
                    }
                    let tx = tx.clone();
                    let Ok(device) = a.device(addr) else { continue };
                    handles.insert(
                        addr,
                        tokio::spawn(async move {
                            let Ok(mut events) = device.events().await else {
                                return;
                            };
                            while let Some(evt) = events.next().await {
                                let bluer::DeviceEvent::PropertyChanged(prop) = evt;
                                match prop {
                                    DeviceProperty::Connected(_)
                                    | DeviceProperty::Paired(_)
                                    | DeviceProperty::Name(_)
                                    | DeviceProperty::Alias(_) => {
                                        let _ =
                                            tx.unbounded_send(MonitorEvent::DeviceChanged(addr));
                                    }
                                    _ => {}
                                }
                            }
                        }),
                    );
                }
            }
            Err(e) => {
                eprintln!("[monitor] Sync error: {e}");
                // If we can't even list devices, the adapter may be gone.
                // The adapter listener will catch Powered changes.
                // BlueZ restarts are detected by the reconnect retry loop
                // (init failure → retry).
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
