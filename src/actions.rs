use crate::app::BludioApp;
use crate::bluetooth::{self, device, properties};
use crate::ui::bluetooth_page::Action;
use gpui::{AsyncApp, WeakEntity};

/// Execute a device action (connect / disconnect / forget / pair+trust)
/// with three phases:
/// 1. Run the action on Tokio
/// 2. Quick single-device status update for instant UI feedback
/// 3. Full list refresh after BlueZ settles (delayed 500ms)
pub(crate) async fn execute(
    adapter: bluer::Adapter,
    addr: bluer::Address,
    action: Action,
    this: WeakEntity<BludioApp>,
    cx: &mut AsyncApp,
) {
    // ── 1. Execute on Tokio ──
    let a = adapter.clone();
    let _ = crate::tokio_task(async move {
        match action {
            Action::Connect => {
                if let Ok(device) = a.device(addr) {
                    let _ = device.connect().await;
                }
            }
            Action::Disconnect => {
                if let Ok(device) = a.device(addr) {
                    let _ = device.disconnect().await;
                }
            }
            Action::Forget => {
                let _ = a.remove_device(addr).await;
            }
            Action::PairAndTrust => {
                if let Ok(device) = a.device(addr) {
                    let _ = device.pair().await;
                    let _ = device.set_trusted(true).await;
                    let _ = device.connect().await;
                }
            }
        }
    })
    .await;

    // ── 2. Quick status update ──
    let a = adapter.clone();
    let quick = crate::tokio_task(async move { quick_device_status(&a, addr).await });
    if let Ok(Some(device)) = quick.await {
        let _ = this.update(cx, |this, cx| {
            this.bt_state.upsert_device(device);
            cx.notify();
        });
    } else if action == Action::Forget {
        let _ = this.update(cx, |this, cx| {
            this.bt_state.remove_device(addr);
            cx.notify();
        });
    }

    // ── 3. Full list refresh (delayed) ──
    let a = adapter;
    if let Ok(Some(devices)) = crate::tokio_task(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        bluetooth::discovery::refresh_device_list(&a).await
    })
    .await
    {
        let _ = this.update(cx, |this, cx| {
            this.bt_state.replace_devices(devices);
            cx.notify();
        });
    }
}

/// Minimal status fetch for immediate UI updates — no filtering.
pub(crate) async fn quick_device_status(
    adapter: &bluer::Adapter,
    addr: bluer::Address,
) -> Option<device::BluetoothDevice> {
    let device = adapter.device(addr).ok()?;
    let props =
        properties::fetch_properties(&device, properties::PropertyTimeouts::default()).await;
    properties::build_device(addr, &props, false, false)
}
