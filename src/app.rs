//! Application entity: `Page` navigation, `BludioApp` state, render, and discovery.
//!
//! Follows Zed's pattern where the entry point (`main.rs`) is thin and the
//! app entity lives in its own module.

use crate::bluetooth::BluetoothState;
use crate::ui;
use gpui::{App, Context, FocusHandle, Focusable, IntoElement, Render, Window, hsla, prelude::*};
use ui::{h_flex, icons, tab_bar, v_flex};

// ── Page navigation ───────────────────────────────────────────────────────

/// Available pages in the application.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    BluetoothDevices,
    Page2,
}

// ── App state ──────────────────────────────────────────────────────────────

pub(crate) struct BludioApp {
    pub(crate) bt_state: BluetoothState,
    pub(crate) bt_agent: Option<crate::bluetooth::agent::AgentHandle>,
    pub(crate) initialized: bool,
    active_page: Page,
    _focus_handle: FocusHandle,
}

impl BludioApp {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            // ── Init ──
            let rx = crate::tokio_task(async {
                let state = BluetoothState::new().await?;
                let agent =
                    crate::bluetooth::agent::register_agent(state.session.as_ref().unwrap()).await;
                Ok::<_, String>((state, agent))
            });
            let adapter = match rx.await {
                Ok(Ok((state, agent))) => {
                    let adapter = state.adapter.clone().unwrap();
                    this.update(cx, |this, cx| {
                        this.bt_state = state;
                        this.initialized = true;
                        this.bt_agent = agent.ok();
                        cx.notify();
                    })
                    .ok();
                    adapter
                }
                Ok(Err(e)) => {
                    this.update(cx, |this, cx| {
                        this.bt_state.error = Some(e);
                        cx.notify();
                    })
                    .ok();
                    return;
                }
                Err(_) => {
                    this.update(cx, |this, cx| {
                        this.bt_state.error = Some("Bluetooth initialization cancelled".into());
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };

            // ── Background monitoring (signals + fallback poll) ──
            let (monitor_tx, mut change_rx) = tokio::sync::mpsc::unbounded_channel();
            let monitor_adapter = adapter.clone();
            crate::TOKIO.spawn(async move {
                crate::bluetooth::monitor::run_monitor(&monitor_adapter, monitor_tx).await;
            });

            // Enter Tokio context so mpsc::recv works on this thread.
            let _tokio_guard = crate::TOKIO.enter();

            loop {
                let discovering = this
                    .read_with(cx, |app, _| app.bt_state.discovering)
                    .unwrap_or(false);
                if discovering {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(1000))
                        .await;
                    continue;
                }

                // Wait for a D-Bus signal or a 10s fallback timer.
                let signal_addr: Option<bluer::Address> = tokio::select! {
                    Some(addr) = change_rx.recv() => Some(addr),
                    _ = cx.background_executor()
                        .timer(std::time::Duration::from_secs(10)) => None,
                };

                let a = adapter.clone();
                if let Some(addr) = signal_addr {
                    // Single-device quick refresh from D-Bus signal.
                    if let Ok(Some(device)) = crate::tokio_task(async move {
                        crate::actions::quick_device_status(&a, addr).await
                    })
                    .await
                    {
                        let _ = this.update(cx, |this, cx| {
                            this.bt_state.upsert_device(device);
                            cx.notify();
                        });
                    }
                } else {
                    // 10s fallback: full list refresh.
                    if let Ok(Some(fresh)) = crate::tokio_task(async move {
                        crate::bluetooth::discovery::refresh_device_list(&a).await
                    })
                    .await
                    {
                        let changed = this
                            .read_with(cx, |app, _| devices_changed(&app.bt_state.devices, &fresh))
                            .unwrap_or(true);
                        if changed {
                            let _ = this.update(cx, |this, cx| {
                                this.bt_state.replace_devices(fresh);
                                cx.notify();
                            });
                        }
                    }
                }
            }
        })
        .detach();

        Self {
            bt_state: BluetoothState::default(),
            bt_agent: None,
            initialized: false,
            active_page: Page::BluetoothDevices,
            _focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for BludioApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self._focus_handle.clone()
    }
}

impl Render for BludioApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_page = self.active_page;

        let bg = hsla(0.0, 0.0, 0.08, 1.0);
        let text = hsla(0.0, 0.0, 0.95, 1.0);
        let text_secondary = hsla(0.0, 0.0, 0.6, 1.0);

        let this = cx.weak_entity();

        let tabs = [
            tab_bar::Tab {
                icon: icons::bluetooth,
                tooltip: "Bluetooth Devices",
            },
            tab_bar::Tab {
                icon: icons::page_placeholder,
                tooltip: "Page 2",
            },
        ];
        let active_tab_index = match active_page {
            Page::BluetoothDevices => 0,
            Page::Page2 => 1,
        };

        h_flex()
            .size_full()
            .items_stretch()
            .bg(bg)
            .text_color(text)
            // ── Left tab bar ──
            .child(tab_bar::tab_bar_view(&tabs, active_tab_index, {
                let this = this.clone();
                move |idx: usize, _window: &mut Window, app: &mut gpui::App| {
                    if let Some(this) = this.upgrade() {
                        let new_page = match idx {
                            0 => Page::BluetoothDevices,
                            _ => Page::Page2,
                        };
                        this.update(app, |this, cx| {
                            if this.active_page != new_page {
                                this.active_page = new_page;
                                cx.notify();
                            }
                        });
                    }
                }
            }))
            // ── Right content area ──
            .child(
                v_flex()
                    .flex_1()
                    .child(match active_page {
                        Page::BluetoothDevices => {
                            ui::bluetooth_page::bluetooth_page_view(&self.bt_state, cx)
                                .into_any_element()
                        }
                        Page::Page2 => h_flex()
                            .flex_1()
                            .justify_center()
                            .text_color(text_secondary)
                            .child("Page 2")
                            .into_any_element(),
                    })
                    .into_any_element(),
            )
    }
}

// ── Discovery orchestration ────────────────────────────────────────────────

pub(crate) async fn run_discovery(this: gpui::WeakEntity<BludioApp>, cx: &mut gpui::AsyncApp) {
    let _tokio_guard = crate::TOKIO.enter();

    let adapter = match this.read_with(cx, |app, _| app.bt_state.adapter.clone()) {
        Ok(Some(a)) => a,
        _ => return,
    };

    let mut rx = crate::bluetooth::discovery::start_scan(&adapter);

    while let Some(event) = rx.recv().await {
        match event {
            crate::bluetooth::discovery::DiscoveryEvent::DeviceAdded(device) => {
                let _ = this.update(cx, |this, cx| {
                    this.bt_state.upsert_device(device);
                    cx.notify();
                });
                if !this
                    .read_with(cx, |app, _| app.bt_state.discovering)
                    .unwrap_or(false)
                {
                    break;
                }
            }
            crate::bluetooth::discovery::DiscoveryEvent::Done => break,
        }
    }

    let a = adapter;
    if let Ok(Some(devices)) =
        crate::tokio_task(async move { crate::bluetooth::discovery::refresh_device_list(&a).await })
            .await
    {
        let _ = this.update(cx, |this, cx| {
            this.bt_state.replace_devices(devices);
            this.bt_state.discovering = false;
            cx.notify();
        });
        return;
    }

    let _ = this.update(cx, |this, cx| {
        this.bt_state.discovering = false;
        cx.notify();
    });
}

// ── Utilities ──────────────────────────────────────────────────────────────

fn devices_changed(
    old: &[crate::bluetooth::device::BluetoothDevice],
    new: &[crate::bluetooth::device::BluetoothDevice],
) -> bool {
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
