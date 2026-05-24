//! Application entity: `Page` navigation, `BludioApp` state, render, and discovery.
//!
//! Follows Zed's pattern where the entry point (`main.rs`) is thin and the
//! app entity lives in its own module.

use crate::audio::AudioState;
use crate::audio::DeviceKind;
use crate::bluetooth::BluetoothState;
use crate::bluetooth::device::{devices_changed, execute_device_action, quick_device_status};
use crate::ui::audio::audio_page::AudioPage;
use crate::ui::bluetooth::BluetoothPageCommand;
use crate::ui::bluetooth::bluetooth_page::BluetoothPage;
use crate::ui::dev_test_page::DevTestPage;
use crate::ui::icons;
use crate::ui::tab_bar;
use crate::ui::{h_flex, v_flex};
use futures::{FutureExt, StreamExt};
use gpui::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window, hsla, prelude::*,
};

// ── Page navigation ───────────────────────────────────────────────────────

/// Available pages in the application.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    BluetoothDevices,
    AudioOutputs,
    AudioInputs,
    DevTest,
}

// ── App state ──────────────────────────────────────────────────────────────

pub(crate) struct BludioApp {
    pub(crate) bt_state: BluetoothState,
    pub(crate) bt_agent: Option<crate::bluetooth::agent::AgentHandle>,
    pub(crate) audio_state: AudioState,
    /// Self-contained page entities.
    audio_output_page: Entity<AudioPage>,
    audio_input_page: Entity<AudioPage>,
    /// Developer test page: self-contained entity.
    dev_test_page: Entity<DevTestPage>,
    /// Bluetooth device page: self-contained entity.
    pub(crate) bluetooth_page: Entity<BluetoothPage>,
    active_page: Page,
    _focus_handle: FocusHandle,
}

impl BludioApp {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // ── Audio init: spawn PA thread ──
        let (audio_cmd_tx, audio_cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (audio_state_tx, audio_state_rx) = tokio::sync::mpsc::unbounded_channel();
        let (wakeup_tx, wakeup_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            crate::audio::pulse::run_pa_thread_from_channels(
                audio_cmd_rx,
                audio_state_tx,
                wakeup_tx,
            );
        });

        // Receive the wakeup handle (block briefly, PA thread sends it immediately).
        let pa_wakeup = wakeup_rx.recv().ok();
        let pa_init_error = pa_wakeup
            .is_none()
            .then(|| "PulseAudio failed to initialize".to_string());

        // ── Wire background subsystems ──
        Self::spawn_audio_state_loop(audio_state_rx, window, cx);
        Self::spawn_bluetooth_init(window, cx);

        // ── Bluetooth command channel ──
        let (bt_cmd_tx, bt_cmd_rx) = futures::channel::mpsc::unbounded::<BluetoothPageCommand>();

        // ── Create page entities ──
        let bluetooth_page = cx.new(|cx| BluetoothPage::new(bt_cmd_tx, cx));
        let audio_output_page =
            cx.new(|cx| AudioPage::new(DeviceKind::Output, audio_cmd_tx.clone(), pa_wakeup, cx));
        let audio_input_page =
            cx.new(|cx| AudioPage::new(DeviceKind::Input, audio_cmd_tx.clone(), pa_wakeup, cx));
        let dev_test_page = cx.new(DevTestPage::new);

        Self::spawn_bluetooth_command_handler(bt_cmd_rx, window, cx);

        let mut audio_state = AudioState::default();
        if let Some(err) = pa_init_error {
            audio_state.error = Some(err);
        }

        Self {
            bt_state: BluetoothState::default(),
            bt_agent: None,
            audio_state,
            audio_output_page,
            audio_input_page,
            dev_test_page,
            bluetooth_page,
            active_page: Page::BluetoothDevices,
            _focus_handle: cx.focus_handle(),
        }
    }

    // ── Audio state loop ───────────────────────────────────────────────

    fn spawn_audio_state_loop(
        audio_state_rx: tokio::sync::mpsc::UnboundedReceiver<AudioState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.spawn_in(window, async move |this, cx| {
            let _tokio_guard = crate::TOKIO.enter();
            let mut rx = audio_state_rx;
            while let Some(state) = rx.recv().await {
                let _ = this.update_in(cx, |this, window, cx| {
                    this.audio_state = state.clone();
                    this.audio_output_page
                        .update(cx, |page, cx| page.sync_rows(&state, window, cx));
                    this.audio_input_page
                        .update(cx, |page, cx| page.sync_rows(&state, window, cx));
                    cx.notify();
                });
            }
        })
        .detach();
    }

    // ── Bluetooth init + monitor loop ──────────────────────────────────

    fn spawn_bluetooth_init(window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
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
                    this.update_in(cx, |this, window, cx| {
                        this.bt_state = state;
                        this.bt_agent = agent.ok();
                        this.bluetooth_page
                            .update(cx, |page, cx| page.sync_state(&this.bt_state, window, cx));
                        cx.notify();
                    })
                    .ok();
                    adapter
                }
                Ok(Err(e)) => {
                    this.update_in(cx, |this, window, cx| {
                        this.bt_state.error = Some(e);
                        this.bluetooth_page
                            .update(cx, |page, cx| page.sync_state(&this.bt_state, window, cx));
                        cx.notify();
                    })
                    .ok();
                    return;
                }
                Err(_) => {
                    this.update_in(cx, |this, window, cx| {
                        this.bt_state.error = Some("Bluetooth initialization cancelled".into());
                        this.bluetooth_page
                            .update(cx, |page, cx| page.sync_state(&this.bt_state, window, cx));
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };

            // ── Background monitoring (signals + fallback poll) ──
            let (monitor_tx, mut change_rx) = futures::channel::mpsc::unbounded::<bluer::Address>();
            let monitor_adapter = adapter.clone();
            crate::TOKIO.spawn(async move {
                crate::bluetooth::monitor::run_monitor(&monitor_adapter, monitor_tx).await;
            });

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
                let signal_addr: Option<bluer::Address> = futures::select! {
                    addr = change_rx.next().fuse() => addr,
                    _ = cx.background_executor()
                        .timer(std::time::Duration::from_secs(10)).fuse() => None,
                };

                let a = adapter.clone();
                if let Some(addr) = signal_addr {
                    // Single-device quick refresh from D-Bus signal.
                    if let Ok(Some(device)) =
                        crate::tokio_task(async move { quick_device_status(&a, addr).await }).await
                    {
                        let _ = this.update_in(cx, |this, window, cx| {
                            this.bt_state.upsert_device(device);
                            this.bluetooth_page.update(cx, |page, cx| {
                                page.sync_state(&this.bt_state, window, cx);
                            });
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
                            let _ = this.update_in(cx, |this, window, cx| {
                                this.bt_state.replace_devices(fresh);
                                this.bluetooth_page.update(cx, |page, cx| {
                                    page.sync_state(&this.bt_state, window, cx);
                                });
                                cx.notify();
                            });
                        }
                    }
                }
            }
        })
        .detach();
    }

    // ── Bluetooth command handler ───────────────────────────────────────

    fn spawn_bluetooth_command_handler(
        bt_cmd_rx: futures::channel::mpsc::UnboundedReceiver<BluetoothPageCommand>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.spawn_in(window, async move |this, cx| {
            let mut bt_cmd_rx = bt_cmd_rx;
            while let Some(cmd) = bt_cmd_rx.next().await {
                let adapter = match this
                    .read_with(cx, |app, _| app.bt_state.adapter.clone())
                    .unwrap_or(None)
                {
                    Some(a) => a,
                    None => continue,
                };
                match cmd {
                    BluetoothPageCommand::ToggleScan => {
                        let _ = this.update_in(cx, |this, window, cx| {
                            if this.bt_state.discovering {
                                this.bt_state.discovering = false;
                            } else {
                                this.bt_state.discovering = true;
                                cx.spawn_in(window, async move |this, cx| {
                                    crate::bluetooth::discovery::run_discovery(this, cx).await;
                                })
                                .detach();
                            }
                            this.bluetooth_page
                                .update(cx, |page, cx| page.sync_state(&this.bt_state, window, cx));
                            cx.notify();
                        });
                    }
                    BluetoothPageCommand::DeviceAction { addr, action } => {
                        // Route D-Bus operations through the Tokio runtime.
                        let a = adapter.clone();
                        let _ = crate::tokio_task(async move {
                            execute_device_action(&a, addr, action).await;
                        })
                        .await;
                        // Quick status update (also via Tokio).
                        let a = adapter.clone();
                        if let Ok(Some(device)) =
                            crate::tokio_task(async move { quick_device_status(&a, addr).await })
                                .await
                        {
                            let _ = this.update_in(cx, |this, window, cx| {
                                this.bt_state.upsert_device(device);
                                this.bluetooth_page.update(cx, |page, cx| {
                                    page.sync_state(&this.bt_state, window, cx);
                                });
                                cx.notify();
                            });
                        }
                        // Full list refresh (delayed)
                        let a = adapter.clone();
                        if let Ok(Some(devices)) = crate::tokio_task(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            crate::bluetooth::discovery::refresh_device_list(&a).await
                        })
                        .await
                        {
                            let _ = this.update_in(cx, |this, window, cx| {
                                this.bt_state.replace_devices(devices);
                                this.bluetooth_page.update(cx, |page, cx| {
                                    page.sync_state(&this.bt_state, window, cx);
                                });
                                cx.notify();
                            });
                        }
                    }
                }
            }
        })
        .detach();
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

        let this = cx.weak_entity();

        let tabs = [
            tab_bar::Tab {
                icon: icons::bluetooth,
                tooltip: "Bluetooth Devices",
            },
            tab_bar::Tab {
                icon: icons::audio_output,
                tooltip: "Output Devices",
            },
            tab_bar::Tab {
                icon: icons::audio_input,
                tooltip: "Input Devices",
            },
            tab_bar::Tab {
                icon: icons::text_field_test,
                tooltip: "Text Field Test",
            },
        ];
        let active_tab_index = match active_page {
            Page::BluetoothDevices => 0,
            Page::AudioOutputs => 1,
            Page::AudioInputs => 2,
            Page::DevTest => 3,
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
                            1 => Page::AudioOutputs,
                            2 => Page::AudioInputs,
                            3 => Page::DevTest,
                            _ => Page::DevTest,
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
                        Page::BluetoothDevices => self.bluetooth_page.clone().into_any_element(),
                        Page::AudioOutputs => self.audio_output_page.clone().into_any_element(),
                        Page::AudioInputs => self.audio_input_page.clone().into_any_element(),
                        Page::DevTest => self.dev_test_page.clone().into_any_element(),
                    })
                    .into_any_element(),
            )
    }
}
