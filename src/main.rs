mod actions;
mod bluetooth;
mod device_list;
mod icons;
mod tab_bar;
mod tooltip;

use std::sync::LazyLock;

use bluetooth::BluetoothState;
use futures::channel::oneshot;
use gpui::{
    App, Bounds, Context, CursorStyle, FocusHandle, Focusable, FontWeight, IntoElement,
    MouseButton, MouseUpEvent, Render, SharedString, Window, WindowBounds, WindowOptions, div,
    hsla, prelude::*, px, rgba, size,
};
use gpui_platform_gpui_unofficial::application;

// ── Tokio bridge ───────────────────────────────────────────────────────────

/// Global Tokio runtime for all BlueZ D-Bus operations.
///
/// gpui-unofficial's executor is not Tokio-based, but `bluer` requires a
/// Tokio 1.x reactor. We spin up a dedicated runtime and route all Bluetooth
/// work through `tokio_task()`.
static TOKIO: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"));

pub(crate) fn tokio_task<F, T>(f: F) -> oneshot::Receiver<T>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    TOKIO.spawn(async move {
        let _ = tx.send(f.await);
    });
    rx
}

// ── Page navigation ───────────────────────────────────────────────────────

/// Available pages in the application.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    BluetoothDevices,
    Page2,
}

// ── App state ──────────────────────────────────────────────────────────────

pub(crate) struct BludioApp {
    bt_state: BluetoothState,
    bt_agent: Option<bluetooth::agent::AgentHandle>,
    initialized: bool,
    active_page: Page,
    _focus_handle: FocusHandle,
}

impl BludioApp {
    fn new(cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            // ── Init ──
            let rx = tokio_task(async {
                let state = BluetoothState::new().await?;
                let agent = bluetooth::agent::register_agent(
                    state.session.as_ref().unwrap(),
                )
                .await;
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
                        this.bt_state.error =
                            Some("Bluetooth initialization cancelled".into());
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };

            // ── Background monitoring (signals + fallback poll) ──
            let (monitor_tx, mut change_rx) =
                tokio::sync::mpsc::unbounded_channel();
            let monitor_adapter = adapter.clone();
            TOKIO.spawn(async move {
                bluetooth::monitor::run_monitor(&monitor_adapter, monitor_tx).await;
            });

            // Enter Tokio context so mpsc::recv works on this thread.
            let _tokio_guard = TOKIO.enter();

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
                    if let Ok(Some(device)) =
                        tokio_task(async move { crate::actions::quick_device_status(&a, addr).await }).await
                    {
                        let _ = this.update(cx, |this, cx| {
                            this.bt_state.upsert_device(device);
                            cx.notify();
                        });
                    }
                } else {
                    // 10s fallback: full list refresh.
                    if let Ok(Some(fresh)) = tokio_task(async move {
                        bluetooth::discovery::refresh_device_list(&a).await
                    })
                    .await
                    {
                        let changed = this
                            .read_with(cx, |app, _| {
                                devices_changed(&app.bt_state.devices, &fresh)
                            })
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
        let discovering = self.bt_state.discovering;
        let error = self.bt_state.error.clone();
        let initialized = self.initialized;
        let active_page = self.active_page;

        let bg = hsla(0.0, 0.0, 0.08, 1.0);
        let surface = hsla(0.0, 0.0, 0.14, 1.0);
        let text = hsla(0.0, 0.0, 0.95, 1.0);
        let text_secondary = hsla(0.0, 0.0, 0.6, 1.0);
        let accent = hsla(210.0 / 360.0, 0.7, 0.55, 1.0);
        let accent_hover = hsla(210.0 / 360.0, 0.7, 0.45, 1.0);
        let danger = hsla(0.0, 0.7, 0.55, 1.0);
        let danger_hover = hsla(0.0, 0.7, 0.45, 1.0);

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

        div()
            .size_full()
            .flex()
            .flex_row()
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
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(match active_page {
                        Page::BluetoothDevices => {
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                // ── Bluetooth header ──
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .px_4()
                                        .py_2()
                                        .bg(surface)
                                        .border_b_1()
                                        .border_color(hsla(0.0, 0.0, 0.25, 1.0))
                                        .child(
                                            div().font_weight(FontWeight::BOLD).child("bluetooth"),
                                        )
                                        .child(scan_button(
                                            discovering,
                                            danger,
                                            accent,
                                            danger_hover,
                                            accent_hover,
                                            cx,
                                        )),
                                )
                                // ── Error / Loading / Device list ──
                                .when_some(error, |el, err| el.child(error_banner(err)))
                                .when(
                                    !initialized && self.bt_state.error.is_none(),
                                    |el| el.child(loading_indicator(text_secondary)),
                                )
                                .when(initialized, |el| {
                                    el.child(device_list::device_list_view(
                                        &self.bt_state,
                                        cx,
                                    ))
                                })
                                .into_any_element()
                        }
                        Page::Page2 => div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(text_secondary)
                            .child("Page 2")
                            .into_any_element(),
                    })
                    .into_any_element(),
            )
    }
}

// ── Header sub-components ──────────────────────────────────────────────────

fn scan_button(
    discovering: bool,
    danger: gpui::Hsla,
    accent: gpui::Hsla,
    danger_hover: gpui::Hsla,
    accent_hover: gpui::Hsla,
    cx: &mut Context<BludioApp>,
) -> impl IntoElement {
    div()
        .id("scan-btn")
        .px_3()
        .py_1()
        .rounded_md()
        .bg(if discovering { danger } else { accent })
        .cursor(CursorStyle::PointingHand)
        .hover(|el| el.bg(if discovering { danger_hover } else { accent_hover }))
        .child(if discovering { "stop" } else { "scan" })
        .on_mouse_up(MouseButton::Left, cx.listener(
            |this, _: &MouseUpEvent, _window, cx| {
                if this.bt_state.discovering {
                    this.bt_state.discovering = false;
                } else {
                    this.bt_state.discovering = true;
                    cx.spawn(async move |this, cx| {
                        run_discovery(this, cx).await;
                    })
                    .detach();
                }
                cx.notify();
            },
        ))
}

fn error_banner(err: String) -> impl IntoElement {
    div()
        .px_4()
        .py_2()
        .bg(rgba(0xff3c3c33))
        .text_color(hsla(0.0, 0.8, 0.75, 1.0))
        .child(SharedString::from(format!("Error: {}", err)))
}

fn loading_indicator(text_secondary: gpui::Hsla) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .flex_1()
        .text_color(text_secondary)
        .child("Connecting to Bluetooth...")
}

// ── Discovery orchestration ────────────────────────────────────────────────

async fn run_discovery(this: gpui::WeakEntity<BludioApp>, cx: &mut gpui::AsyncApp) {
    let _tokio_guard = TOKIO.enter();

    let adapter = match this.read_with(cx, |app, _| app.bt_state.adapter.clone()) {
        Ok(Some(a)) => a,
        _ => return,
    };

    let mut rx = bluetooth::discovery::start_scan(&adapter);

    while let Some(event) = rx.recv().await {
        match event {
            bluetooth::discovery::DiscoveryEvent::DeviceAdded(device) => {
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
            bluetooth::discovery::DiscoveryEvent::Done => break,
        }
    }

    let a = adapter;
    if let Ok(Some(devices)) =
        tokio_task(async move { bluetooth::discovery::refresh_device_list(&a).await }).await
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
    old: &[bluetooth::device::BluetoothDevice],
    new: &[bluetooth::device::BluetoothDevice],
) -> bool {
    if old.len() != new.len() {
        return true;
    }
    for (a, b) in old.iter().zip(new.iter()) {
        if a.address != b.address
            || a.paired != b.paired
            || a.connected != b.connected
        {
            return true;
        }
    }
    false
}

// ── Entry point ────────────────────────────────────────────────────────────

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                app_id: Some("com.bludio.app".to_string()),
                ..Default::default()
            },
            |_window, cx| cx.new(BludioApp::new),
        )
        .unwrap();

        cx.activate(true);
    });
}
