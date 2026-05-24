use std::collections::HashMap;

use bluer::Adapter;
use futures::channel::mpsc;
use tokio::task::JoinHandle;

/// Start background D-Bus property-change monitoring.
///
/// Spawns per-device event listeners that send the device address through
/// `tx` whenever `Connected`, `Paired`, `Name`, or `Alias` changes.
/// Re-syncs the listener set every 5 seconds to catch new/removed devices.
pub(crate) async fn run_monitor(adapter: &Adapter, tx: mpsc::UnboundedSender<bluer::Address>) {
    let mut handles: HashMap<bluer::Address, JoinHandle<()>> = HashMap::new();
    let a = adapter.clone();

    loop {
        match a.device_addresses().await {
            Ok(addresses) => {
                handles.retain(|addr, _| addresses.contains(addr));

                for addr in addresses {
                    if handles.contains_key(&addr) {
                        continue;
                    }
                    let tx = tx.clone();
                    let device = match a.device(addr) {
                        Ok(d) => d,
                        Err(_) => continue,
                    };
                    handles.insert(
                        addr,
                        tokio::spawn(async move {
                            let Ok(mut events) = device.events().await else {
                                return;
                            };
                            use futures::StreamExt;
                            while let Some(evt) = events.next().await {
                                let bluer::DeviceEvent::PropertyChanged(prop) = evt;
                                use bluer::DeviceProperty;
                                match prop {
                                    DeviceProperty::Connected(_)
                                    | DeviceProperty::Paired(_)
                                    | DeviceProperty::Name(_)
                                    | DeviceProperty::Alias(_) => {
                                        let _ = tx.unbounded_send(addr);
                                    }
                                    _ => {}
                                }
                            }
                        }),
                    );
                }
            }
            Err(e) => {
                eprintln!("Monitor sync error: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
