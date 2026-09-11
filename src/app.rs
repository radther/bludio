//! Application entity: `Page` navigation, `BludioApp` state, render, and discovery.
//!
//! Follows Zed's pattern where the entry point (`main.rs`) is thin and the
//! app entity lives in its own module.

use crate::backend::audio::AudioState;
use crate::backend::audio::DeviceKind;
use crate::backend::audio::pulse::PaWakeup;
use crate::backend::bluetooth::BluetoothState;
use crate::backend::bluetooth::device::{
    DeviceRowAction, PairingStatus, devices_changed, execute_device_action, format_device_error,
    quick_device_status,
};
use crate::backend::bluetooth::monitor::MonitorEvent;
use crate::backend::subsystem::SubsystemStatus;

// ── Audio connection helper ────────────────────────────────────────────────

/// Channels and receiver from a PulseAudio thread spawn attempt.
struct AudioConnection {
    cmd_tx: tokio::sync::mpsc::UnboundedSender<crate::backend::audio::AudioCommand>,
    state_rx: tokio::sync::mpsc::UnboundedReceiver<AudioState>,
    wakeup_rx: std::sync::mpsc::Receiver<PaWakeup>,
}

impl AudioConnection {
    /// Spawn a PA thread and return the channels + wakeup receiver.
    /// The caller decides how to wait for the wakeup (blocking or timeout).
    ///
    /// If the caller drops the receivers, the PA thread will self-terminate
    /// on the next mainloop iteration (cmd_rx closed or state_tx send fails).
    fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (state_tx, state_rx) = tokio::sync::mpsc::unbounded_channel();
        let (wakeup_tx, wakeup_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            crate::backend::audio::pulse::run_pa_thread_from_channels(
                cmd_rx, &state_tx, &wakeup_tx,
            );
        });

        Self {
            cmd_tx,
            state_rx,
            wakeup_rx,
        }
    }
}
use crate::backend::settings::{settings, update_settings};
use crate::ui::common::icons;
use crate::ui::components::tab_bar::{Tab, TabAction, TabBar, TabBarEvent};
use crate::ui::pages::audio::audio_page::AudioPage;
use crate::ui::pages::bluetooth::BluetoothPageCommand;
use crate::ui::pages::bluetooth::bluetooth_page::BluetoothPage;
use crate::ui::pages::configuration::configuration_page::ConfigurationPage;
#[cfg(debug_assertions)]
use crate::ui::pages::dev_test::dev_test_page::DevTestPage;
use crate::ui::pages::settings::settings_page::{SettingsEvent, SettingsPage};
use crate::ui::{h_flex, v_flex};
use futures::{FutureExt, StreamExt};
use gpui::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Subscription, Window,
    prelude::*,
};

// ── Page navigation ───────────────────────────────────────────────────────

/// Available pages in the application.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    BluetoothDevices,
    AudioOutputs,
    AudioInputs,
    Configuration,
    Settings,
    #[cfg(debug_assertions)]
    DevTest,
}

/// Errors that can occur when running a command via `pkexec`.
#[derive(Debug, PartialEq)]
pub(crate) enum PkexecError {
    NotFound,
    AuthCancelled,
    NotAuthorized,
    Failed(i32, String),
}

impl std::fmt::Display for PkexecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "pkexec not found — PolicyKit is required"),
            Self::AuthCancelled => write!(f, "Authentication cancelled"),
            Self::NotAuthorized => {
                write!(f, "Not authorized — is the policy file installed?")
            }
            Self::Failed(code, stderr) => {
                write!(f, "pkexec failed with code {code}: {stderr}")
            }
        }
    }
}

// ── App state ──────────────────────────────────────────────────────────────

pub(crate) struct BludioApp {
    pub(crate) bt_state: BluetoothState,
    pub(crate) bt_agent: Option<crate::backend::bluetooth::agent::AgentHandle>,
    pub(crate) audio_state: AudioState,
    /// PA mainloop wakeup handle — `None` until the PA thread connects.
    pa_wakeup: Option<PaWakeup>,
    /// Self-contained page entities.
    audio_output_page: Entity<AudioPage>,
    audio_input_page: Entity<AudioPage>,
    configuration_page: Entity<ConfigurationPage>,
    /// Developer test page: self-contained entity.
    #[cfg(debug_assertions)]
    dev_test_page: Entity<DevTestPage>,
    /// Bluetooth device page: self-contained entity.
    pub(crate) bluetooth_page: Entity<BluetoothPage>,
    /// Settings page: self-contained entity.
    settings_page: Entity<SettingsPage>,
    _settings_page_sub: Subscription,
    /// Tab bar entity: owns tab selection visual state + animation.
    tab_bar: Entity<TabBar>,
    _tab_bar_sub: Subscription,
    _keystroke_sub: Subscription,
    active_page: Page,
    focus_handle: FocusHandle,
}

impl BludioApp {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // ── Audio init: spawn PA thread ──
        let conn = AudioConnection::spawn();
        // Non-blocking: if PA is already running the wakeup is ready
        // immediately; if not, the state loop will wait for it async.
        let pa_wakeup = conn.wakeup_rx.try_recv().ok();
        let audio_cmd_tx = conn.cmd_tx;
        let audio_state_rx = conn.state_rx;

        // If the wakeup wasn't immediately available, hand the receiver
        // to the state loop so it can wait asynchronously (no main-thread
        // block).
        let wakeup_rx_for_loop = if pa_wakeup.is_none() {
            Some(conn.wakeup_rx)
        } else {
            None
        };

        // ── Wire background subsystems ──
        Self::spawn_audio_state_loop(audio_state_rx, wakeup_rx_for_loop, window, cx);
        Self::spawn_bluetooth_init(window, cx);

        // ── Bluetooth command channel ──
        let (bt_cmd_tx, bt_cmd_rx) = futures::channel::mpsc::unbounded::<BluetoothPageCommand>();

        // ── Create page entities ──
        let bluetooth_page = cx.new(|cx| BluetoothPage::new(bt_cmd_tx, cx));
        let audio_output_page =
            cx.new(|cx| AudioPage::new(DeviceKind::Output, audio_cmd_tx.clone(), pa_wakeup, cx));
        let audio_input_page =
            cx.new(|cx| AudioPage::new(DeviceKind::Input, audio_cmd_tx.clone(), pa_wakeup, cx));
        let configuration_page =
            cx.new(|cx| ConfigurationPage::new(audio_cmd_tx.clone(), pa_wakeup, cx));
        #[cfg(debug_assertions)]
        let dev_test_page = cx.new(DevTestPage::new);
        let settings_page = cx.new(SettingsPage::new);
        let settings_page_sub = cx.subscribe(&settings_page, {
            move |this, _, event: &SettingsEvent, cx| {
                match event {
                    SettingsEvent::ModeChanged(mode) => {
                        update_settings(|s| s.theme_mode = *mode, cx);
                    }
                    SettingsEvent::LightThemeChanged(id) => {
                        update_settings(|s| s.light_theme_id = id.clone(), cx);
                    }
                    SettingsEvent::DarkThemeChanged(id) => {
                        update_settings(|s| s.dark_theme_id = id.clone(), cx);
                    }
                    SettingsEvent::DisableAnimationsChanged(value) => {
                        update_settings(|s| s.disable_animations = *value, cx);
                        crate::ui::common::accessibility::set_disable_animations(*value);
                    }
                    SettingsEvent::FontChanged(family) => {
                        update_settings(|s| s.font_family = family.clone(), cx);
                    }
                }
                this.settings_page
                    .update(cx, |page, cx| page.sync_dropdowns(cx));
            }
        });

        // ── Create tab bar entity and wire up events ──
        let tabs = vec![
            Tab {
                icon: icons::bluetooth,
                tooltip: "Bluetooth Devices",
            },
            Tab {
                icon: icons::audio_output,
                tooltip: "Output Devices",
            },
            Tab {
                icon: icons::audio_input,
                tooltip: "Input Devices",
            },
            Tab {
                icon: icons::audio_card,
                tooltip: "Configuration",
            },
            Tab {
                icon: icons::bolt,
                tooltip: "Settings",
            },
            #[cfg(debug_assertions)]
            Tab {
                icon: icons::text_field_test,
                tooltip: "Text Field Test",
            },
        ];
        let actions = vec![
            TabAction {
                icon: icons::refresh_ccw,
                tooltip: "Restart Audio Stack",
                action_id: "restart-audiostack".into(),
            },
            TabAction {
                icon: icons::power,
                tooltip: "Close",
                action_id: "close".into(),
            },
        ];
        let tab_bar = cx.new(|cx| TabBar::new(tabs, actions, 0, cx));
        let tab_bar_sub = cx.subscribe(&tab_bar, {
            move |this, _tb, event: &TabBarEvent, cx| match event {
                TabBarEvent::TabClicked(idx) => {
                    Self::switch_to_tab(this, *idx, cx);
                }
                TabBarEvent::ActionButtonClicked(action_id) => match action_id.as_str() {
                    "restart-audiostack" => {
                        Self::handle_restart_audiostack(this, cx);
                    }
                    "close" => {
                        cx.quit();
                    }
                    _ => {
                        eprintln!("[app] Unknown tab bar action: {action_id}");
                    }
                },
            }
        });

        // ── Keyboard shortcuts ──
        let keystroke_sub =
            cx.observe_keystrokes(move |this, event: &gpui::KeystrokeEvent, _window, cx| {
                if event.keystroke.modifiers.control {
                    let idx = match event.keystroke.key.as_str() {
                        "1" => Some(0),
                        "2" => Some(1),
                        "3" => Some(2),
                        "4" => Some(3),
                        "5" => Some(4),
                        #[cfg(debug_assertions)]
                        "6" => Some(5),
                        _ => None,
                    };
                    if let Some(idx) = idx {
                        Self::switch_to_tab(this, idx, cx);
                    }
                }
            });

        Self::spawn_bluetooth_command_handler(bt_cmd_rx, window, cx);

        Self {
            bt_state: BluetoothState::default(),
            bt_agent: None,
            audio_state: AudioState::default(),
            pa_wakeup,
            audio_output_page,
            audio_input_page,
            configuration_page,
            #[cfg(debug_assertions)]
            dev_test_page,
            bluetooth_page,
            settings_page,
            _settings_page_sub: settings_page_sub,
            tab_bar,
            _tab_bar_sub: tab_bar_sub,
            _keystroke_sub: keystroke_sub,
            active_page: Page::BluetoothDevices,
            focus_handle: cx.focus_handle(),
        }
    }

    // ── Audio state loop ───────────────────────────────────────────────

    fn spawn_audio_state_loop(
        audio_state_rx: tokio::sync::mpsc::UnboundedReceiver<AudioState>,
        wakeup_rx: Option<std::sync::mpsc::Receiver<PaWakeup>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.spawn_in(window, async move |this, cx| {
            let _tokio_guard = crate::TOKIO.enter();

            // ── Phase 0: wait for PA wakeup handle (async, no UI block) ──
            if let Some(wakeup_rx) = wakeup_rx {
                // Offload the blocking std::sync::mpsc::recv() to a
                // worker thread so we don't stall the background executor.
                let (tx, rx) = tokio::sync::oneshot::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(wakeup_rx.recv().ok());
                });
                // Wait for the wakeup (or channel close if PA thread died).
                if let Ok(Some(wakeup)) = rx.await {
                    let _ = this.update_in(cx, |this, window, cx| {
                        this.pa_wakeup = Some(wakeup);
                        this.audio_output_page.update(cx, |page, cx| {
                            page.set_wakeup(wakeup);
                            page.sync_rows(&this.audio_state, window, cx);
                        });
                        this.audio_input_page.update(cx, |page, cx| {
                            page.set_wakeup(wakeup);
                            page.sync_rows(&this.audio_state, window, cx);
                        });
                        this.configuration_page.update(cx, |page, cx| {
                            page.set_wakeup(wakeup);
                            page.sync_cards(&this.audio_state, cx);
                        });
                        cx.notify();
                    });
                }
            }

            // ── Phase 1: process audio state updates ──
            let mut rx = audio_state_rx;
            while let Some(state) = rx.recv().await {
                let is_disconnected =
                    matches!(state.subsystem_status, SubsystemStatus::Disconnected(_));
                let _ = this.update_in(cx, |this, window, cx| {
                    this.audio_state = state.clone();
                    this.audio_output_page
                        .update(cx, |page, cx| page.sync_rows(&state, window, cx));
                    this.audio_input_page
                        .update(cx, |page, cx| page.sync_rows(&state, window, cx));
                    this.configuration_page
                        .update(cx, |page, cx| page.sync_cards(&state, cx));
                    cx.notify();
                });
                // If PA thread sent a Disconnected state, stop reading and
                // trigger reconnection.
                if is_disconnected {
                    break;
                }
            }

            // Channel closed or received Disconnected — trigger reconnect.
            let _ = this.update_in(cx, |this, window, cx| {
                if !matches!(
                    this.audio_state.subsystem_status,
                    SubsystemStatus::Disconnected(_)
                ) {
                    this.audio_state.subsystem_status =
                        SubsystemStatus::Disconnected("PulseAudio connection lost".into());
                    this.audio_output_page.update(cx, |page, cx| {
                        page.sync_rows(&this.audio_state, window, cx);
                    });
                    this.audio_input_page.update(cx, |page, cx| {
                        page.sync_rows(&this.audio_state, window, cx);
                    });
                    this.configuration_page.update(cx, |page, cx| {
                        page.sync_cards(&this.audio_state, cx);
                    });
                    cx.notify();
                }
                Self::spawn_audio_reconnect(window, cx);
            });
        })
        .detach();
    }

    // ── Audio reconnect loop ───────────────────────────────────────────

    /// Attempt to reconnect to PulseAudio on a fixed 3-second interval.
    /// Spawns a new PA thread, wires channels, and restarts the state loop.
    fn spawn_audio_reconnect(window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let _tokio_guard = crate::TOKIO.enter();
            // Brief pause before first attempt.
            cx.background_executor()
                .timer(std::time::Duration::from_secs(1))
                .await;

            loop {
                // Check if already reconnected (another path may have succeeded).
                let already_connected = this
                    .read_with(cx, |app, _| {
                        app.audio_state.subsystem_status == SubsystemStatus::Connected
                    })
                    .unwrap_or(true);
                if already_connected {
                    return;
                }

                // Set Reconnecting status on pages.
                let _ = this.update_in(cx, |this, window, cx| {
                    this.audio_state.subsystem_status = SubsystemStatus::Reconnecting;
                    this.audio_output_page.update(cx, |page, cx| {
                        page.sync_rows(&this.audio_state, window, cx);
                    });
                    this.audio_input_page.update(cx, |page, cx| {
                        page.sync_rows(&this.audio_state, window, cx);
                    });
                    this.configuration_page.update(cx, |page, cx| {
                        page.sync_cards(&this.audio_state, cx);
                    });
                    cx.notify();
                });

                // Attempt to create a new PA thread.
                let conn = AudioConnection::spawn();

                // Wait up to 2s for the wakeup handle.
                let pa_wakeup = {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let wakeup_rx = conn.wakeup_rx;
                    std::thread::spawn(move || {
                        let result = wakeup_rx.recv().ok();
                        let _ = tx.send(result);
                    });
                    match tokio::time::timeout(std::time::Duration::from_secs(2), rx).await {
                        Ok(Ok(Some(wakeup))) => Some(wakeup),
                        _ => None,
                    }
                };

                if pa_wakeup.is_some() {
                    // Success — wire new channels into pages.
                    let audio_cmd_tx = conn.cmd_tx;
                    let audio_state_rx = conn.state_rx;
                    let _ = this.update_in(cx, |this, window, cx| {
                        this.pa_wakeup = pa_wakeup;
                        this.audio_output_page.update(cx, |page, _cx| {
                            page.update_cmd_tx(audio_cmd_tx.clone(), pa_wakeup);
                        });
                        this.audio_input_page.update(cx, |page, _cx| {
                            page.update_cmd_tx(audio_cmd_tx.clone(), pa_wakeup);
                        });
                        this.configuration_page.update(cx, |page, _cx| {
                            page.update_cmd_tx(audio_cmd_tx.clone(), pa_wakeup);
                        });

                        // Spawn new state loop with the new receiver.
                        // No wakeup_rx needed — wakeup is already available.
                        Self::spawn_audio_state_loop(audio_state_rx, None, window, cx);
                    });
                    return;
                }

                // Failed — wait 3s, retry.
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(3))
                    .await;
            }
        })
        .detach();
    }

    // ── Bluetooth init + monitor loop ──────────────────────────────────

    /// Connect to BlueZ, register agent, return (state, agent).
    /// Shared by initial connection and reconnection.
    async fn connect_bluetooth() -> Result<
        (
            BluetoothState,
            crate::backend::bluetooth::agent::AgentHandle,
        ),
        String,
    > {
        let state = BluetoothState::new().await?;
        // SAFETY: BluetoothState::new() only returns Ok after session is set.
        let session = state
            .session
            .as_ref()
            .ok_or("BluetoothState::new() returned Ok but session is None")?;
        let agent = crate::backend::bluetooth::agent::register_agent(session).await?;
        Ok((state, agent))
    }

    fn spawn_bluetooth_init(window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            // ── Init ──
            let rx = crate::tokio_task(async { Self::connect_bluetooth().await });
            let adapter = match rx.await {
                Ok(Ok((state, agent))) => {
                    let Some(adapter) = state.adapter.clone() else {
                        let msg = "BluetoothState::new() returned Ok but adapter is None";
                        eprintln!("[bluetooth] {msg}");
                        this.update_in(cx, |this, _window, cx| {
                            this.bt_state.subsystem_status =
                                SubsystemStatus::Disconnected(msg.into());
                            this.bluetooth_page
                                .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                            cx.notify();
                        })
                        .ok();
                        return;
                    };
                    this.update_in(cx, |this, _window, cx| {
                        this.bt_state = state;
                        this.bt_agent = Some(agent);
                        this.bluetooth_page
                            .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                        cx.notify();
                    })
                    .ok();
                    adapter
                }
                Ok(Err(e)) => {
                    this.update_in(cx, |this, _window, cx| {
                        this.bt_state.subsystem_status = SubsystemStatus::Disconnected(e);
                        this.bluetooth_page
                            .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                        cx.notify();
                    })
                    .ok();
                    return;
                }
                Err(_) => {
                    this.update_in(cx, |this, _window, cx| {
                        this.bt_state.subsystem_status = SubsystemStatus::Disconnected(
                            "Bluetooth initialization cancelled".into(),
                        );
                        this.bluetooth_page
                            .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };

            // ── Background monitoring (signals + fallback poll) ──
            let (monitor_tx, mut monitor_rx) = futures::channel::mpsc::unbounded::<MonitorEvent>();
            let monitor_adapter = adapter.clone();
            crate::TOKIO.spawn(async move {
                crate::backend::bluetooth::monitor::run_monitor(&monitor_adapter, monitor_tx).await;
            });

            // ── Session event watcher (AdapterAdded/Removed = BlueZ restart) ──
            // The session.events() stream is not easily Send-compatible.
            // Instead, we rely on:
            // 1. Adapter Powered property changes (monitor detects rfkill)
            // 2. Init failures during reconnect (catches BlueZ restarts)
            // 3. The monitor sync error (catches adapter disappearance)

            // ── Main event loop: process monitor events ──
            loop {
                // Skip device updates if not connected.
                let is_connected = this
                    .read_with(cx, |app, _| {
                        app.bt_state.subsystem_status == SubsystemStatus::Connected
                    })
                    .unwrap_or(false);
                if !is_connected {
                    // Wait a bit then check again.
                    cx.background_executor()
                        .timer(std::time::Duration::from_secs(1))
                        .await;
                    continue;
                }

                // Wait for a monitor event or a 10s fallback timer.
                let event: Option<MonitorEvent> = futures::select! {
                    evt = monitor_rx.next().fuse() => evt,
                    () = cx.background_executor()
                        .timer(std::time::Duration::from_secs(10)).fuse() => None,
                };

                match event {
                    Some(MonitorEvent::DeviceChanged(addr)) => {
                        let a = adapter.clone();
                        match crate::tokio_task(async move { quick_device_status(&a, addr).await })
                            .await
                        {
                            Ok(Some(device)) => {
                                let _ = this.update_in(cx, |this, _window, cx| {
                                    this.bt_state.upsert_device(device);
                                    this.bluetooth_page.update(cx, |page, cx| {
                                        page.sync_state(&this.bt_state, cx);
                                    });
                                    cx.notify();
                                });
                            }
                            Ok(None) => {
                                eprintln!(
                                    "[bluetooth] quick_device_status returned None for {addr}"
                                );
                            }
                            Err(_) => {
                                eprintln!(
                                    "[bluetooth] quick_device_status task cancelled for {addr}"
                                );
                            }
                        }
                    }
                    Some(MonitorEvent::AdapterPoweredOff) => {
                        let _ = this.update_in(cx, |this, window, cx| {
                            eprintln!("[bluetooth] Adapter powered off");
                            this.bt_state.subsystem_status = SubsystemStatus::Disconnected(
                                "Bluetooth adapter powered off".into(),
                            );
                            this.bluetooth_page
                                .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                            cx.notify();
                            Self::reconnect_bluetooth(this, window, cx);
                        });
                        return;
                    }
                    Some(MonitorEvent::AdapterPoweredOn) => {
                        // If we were disconnected, this is a reconnection signal.
                        // But if we're already connected, ignore.
                        let is_disconnected = this
                            .read_with(cx, |app, _| {
                                matches!(
                                    app.bt_state.subsystem_status,
                                    SubsystemStatus::Disconnected(_)
                                )
                            })
                            .unwrap_or(false);
                        if is_disconnected {
                            let _ = this.update_in(cx, |this, window, cx| {
                                eprintln!("[bluetooth] Adapter powered on, triggering reconnect");
                                Self::reconnect_bluetooth(this, window, cx);
                            });
                            return;
                        }
                    }
                    None => {
                        // 10s fallback: full list refresh.
                        let a = adapter.clone();
                        match crate::tokio_task(async move {
                            crate::backend::bluetooth::discovery::refresh_device_list(&a).await
                        })
                        .await
                        {
                            Ok(Some(fresh)) => {
                                let changed = this
                                    .read_with(cx, |app, _| {
                                        devices_changed(&app.bt_state.devices, &fresh)
                                    })
                                    .unwrap_or(true);
                                if changed {
                                    let _ = this.update_in(cx, |this, _window, cx| {
                                        this.bt_state.replace_devices(fresh);
                                        this.bluetooth_page.update(cx, |page, cx| {
                                            page.sync_state(&this.bt_state, cx);
                                        });
                                        cx.notify();
                                    });
                                }
                            }
                            Ok(None) => {
                                eprintln!("[bluetooth] refresh_device_list returned None");
                            }
                            Err(_) => {
                                eprintln!("[bluetooth] refresh_device_list task cancelled");
                            }
                        }
                    }
                }
            }
        })
        .detach();
    }

    // ── Bluetooth reconnect ────────────────────────────────────────────

    /// Retry loop for Bluetooth reconnection. Attempts `connect_bluetooth()`
    /// every 3 seconds until success, then spawns the monitor event loop.
    /// Must be called from within a spawn_in callback (has window access).
    fn reconnect_bluetooth(this: &mut Self, window: &mut Window, cx: &mut Context<Self>) {
        // Set Reconnecting status.
        this.bt_state.subsystem_status = SubsystemStatus::Reconnecting;
        this.bluetooth_page
            .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
        cx.notify();

        // Spawn the reconnection as a background task with retry loop.
        cx.spawn_in(window, async move |this, cx| {
            // Brief pause before first attempt.
            cx.background_executor()
                .timer(std::time::Duration::from_secs(1))
                .await;

            loop {
                // Attempt reconnection.
                let rx = crate::tokio_task(async { Self::connect_bluetooth().await });

                match rx.await {
                    Ok(Ok((state, agent))) => {
                        let _ = this.update_in(cx, |this, _window, cx| {
                            this.bt_state = state;
                            this.bt_agent = Some(agent);
                            this.bluetooth_page
                                .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                            cx.notify();

                            // Spawn new monitor + event loop.
                            Self::spawn_bluetooth_init(_window, cx);
                        });
                        return;
                    }
                    Ok(Err(e)) => {
                        eprintln!("[bluetooth] Reconnect failed: {e}, retrying in 3s");
                        let _ = this.update_in(cx, |this, _window, cx| {
                            this.bt_state.subsystem_status = SubsystemStatus::Disconnected(e);
                            this.bluetooth_page
                                .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                            cx.notify();
                        });
                    }
                    Err(_) => {
                        eprintln!("[bluetooth] Reconnect task cancelled");
                        return;
                    }
                }

                // Wait 3s before retry.
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(3))
                    .await;
            }
        })
        .detach();
    }

    // ── Post-action device refresh ─────────────────────────────────────

    /// Refresh device state after a user action: quick single-device status,
    /// then a delayed full list refresh to catch anything the D-Bus signal
    /// path might miss.
    async fn refresh_device_after_action(
        adapter: &bluer::Adapter,
        addr: bluer::Address,
        this: &gpui::WeakEntity<Self>,
        cx: &mut gpui::AsyncWindowContext,
    ) {
        // Phase 1: quick single-device status refresh
        let a = adapter.clone();
        match crate::tokio_task(async move {
            crate::backend::bluetooth::device::quick_device_status(&a, addr).await
        })
        .await
        {
            Ok(Some(device)) => {
                let _ = this.update_in(cx, |this, _window, cx| {
                    this.bt_state.upsert_device(device);
                    this.bluetooth_page
                        .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                    cx.notify();
                });
            }
            Ok(None) => {
                eprintln!("[bluetooth] quick_device_status returned None for {addr}");
            }
            Err(_) => {
                eprintln!("[bluetooth] quick_device_status task cancelled for {addr}");
            }
        }

        // Phase 2: full list refresh after a small delay
        let a = adapter.clone();
        match crate::tokio_task(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            crate::backend::bluetooth::discovery::refresh_device_list(&a).await
        })
        .await
        {
            Ok(Some(devices)) => {
                let _ = this.update_in(cx, |this, _window, cx| {
                    this.bt_state.replace_devices(devices);
                    this.bluetooth_page
                        .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                    cx.notify();
                });
            }
            Ok(None) => {
                eprintln!("[bluetooth] refresh_device_list returned None");
            }
            Err(_) => {
                eprintln!("[bluetooth] refresh_device_list task cancelled");
            }
        }
    }

    // ── Pair-and-trust sequence ───────────────────────────────────────

    /// Execute the full PairAndTrust flow: Connect → Pair → Trust+Reconnect.
    ///
    /// Each step runs on the Tokio runtime via `tokio_task`. Status updates
    /// are pushed to the UI through `push_status`. Errors are logged and
    /// surfaced as `PairingStatus::Failed` in the UI.
    async fn execute_pair_and_trust(
        adapter: &bluer::Adapter,
        addr: bluer::Address,
        this: &gpui::WeakEntity<Self>,
        cx: &mut gpui::AsyncWindowContext,
    ) {
        let mut push_status = |status: Option<PairingStatus>, banner_msg: Option<String>| {
            let _ = this.update_in(cx, |this, _window, cx| {
                if let Some(existing) = this.bt_state.devices.iter_mut().find(|d| d.address == addr)
                {
                    existing.pairing_status = status;
                }
                this.bluetooth_page.update(cx, |page, cx| {
                    page.sync_state(&this.bt_state, cx);
                    if let Some(msg) = banner_msg {
                        page.show_error(msg, cx);
                    }
                });
                cx.notify();
            });
        };

        // ── Step-by-step chain ──
        // Helper: map a tokio_task double-Result to (ok, status, banner_msg).
        let map_result =
            |action_name: &str,
             failure_label: &str,
             result: Result<Result<(), bluer::Error>, futures::channel::oneshot::Canceled>|
             -> (bool, Option<PairingStatus>, Option<String>) {
                match result {
                    Ok(Ok(())) => (true, None, None),
                    Ok(Err(e)) => {
                        let err_msg = format_device_error(action_name, addr, &e);
                        eprintln!("[bluetooth] Pairing failed for {addr}: {err_msg}");
                        (
                            false,
                            Some(PairingStatus::Failed(failure_label.into())),
                            Some(err_msg),
                        )
                    }
                    Err(_) => {
                        eprintln!(
                            "[bluetooth] Pairing failed for {addr}: {action_name} task cancelled"
                        );
                        (
                            false,
                            Some(PairingStatus::Failed("internal error".into())),
                            None,
                        )
                    }
                }
            };

        // 1. Connect
        let a2 = adapter.clone();
        let (ok, status, banner) = map_result(
            "connect",
            "connection failed",
            crate::tokio_task(async move {
                let device = a2.device(addr)?;
                device.connect().await
            })
            .await,
        );
        if let Some(s) = status {
            push_status(Some(s), banner);
        }

        // 2. Pair
        let ok = if ok {
            push_status(Some(PairingStatus::Pairing), None);
            let a2 = adapter.clone();
            let (ok, status, banner) = map_result(
                "pair",
                "pairing failed",
                crate::tokio_task(async move {
                    let device = a2.device(addr)?;
                    device.pair().await
                })
                .await,
            );
            if let Some(s) = status {
                push_status(Some(s), banner);
            }
            ok
        } else {
            false
        };

        // 3. Trust + reconnect
        let ok = if ok {
            push_status(Some(PairingStatus::Trusting), None);
            let a2 = adapter.clone();
            let (ok, status, banner) = map_result(
                "trust",
                "trust failed",
                crate::tokio_task(async move {
                    let device = a2.device(addr)?;
                    device.set_trusted(true).await?;
                    device.connect().await
                })
                .await,
            );
            if let Some(s) = status {
                push_status(Some(s), banner);
            }
            ok
        } else {
            false
        };

        if ok {
            push_status(None, None);
        }

        // Refresh device state (regardless of success/failure).
        Self::refresh_device_after_action(adapter, addr, this, cx).await;
    }

    // ── Tab switching ─────────────────────────────────────────────────

    fn switch_to_tab(this: &mut Self, idx: usize, cx: &mut Context<Self>) {
        let new_page = match idx {
            0 => Page::BluetoothDevices,
            1 => Page::AudioOutputs,
            2 => Page::AudioInputs,
            3 => Page::Configuration,
            4 => Page::Settings,
            #[cfg(debug_assertions)]
            5 => Page::DevTest,
            _ => return,
        };
        if this.active_page != new_page {
            this.active_page = new_page;
            this.tab_bar
                .update(cx, |tab_bar, cx| tab_bar.set_active_index(idx, cx));
            cx.notify();
        }
    }

    // ── Audio stack restart ────────────────────────────────────────────

    /// Handle the restart-audiostack action: run the full recovery sequence
    /// and surface per-step errors in the Bluetooth page banner.
    fn handle_restart_audiostack(_this: &mut Self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let results = vec![
                crate::tokio_task(async {
                    Self::run_command(&[
                        "systemctl",
                        "--user",
                        "restart",
                        "wireplumber",
                        "pipewire",
                        "pipewire-pulse",
                    ])
                    .await
                }),
                crate::tokio_task(async {
                    Self::run_pkexec(&["systemctl", "restart", "bluetooth"]).await
                }),
                crate::tokio_task(async {
                    Self::run_command(&["rfkill", "unblock", "bluetooth"]).await
                }),
            ];

            let mut errors = Vec::new();
            for rx in results {
                match rx.await {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => errors.push(e.to_string()),
                    Err(_) => errors.push("Restart task cancelled".into()),
                }
            }

            let _ = this.update(cx, |this, cx| {
                if !errors.is_empty() {
                    let msg = errors.join("; ");
                    this.bluetooth_page.update(cx, |page, cx| {
                        page.show_error(msg, cx);
                    });
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Execute a non-privileged command and return Ok(()) or a typed error.
    async fn run_command(command: &[&str]) -> Result<(), PkexecError> {
        let output = tokio::process::Command::new(command[0])
            .args(&command[1..])
            .output()
            .await;

        match output {
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(PkexecError::NotFound)
                } else {
                    Err(PkexecError::Failed(
                        -1,
                        format!("Failed to run {}: {e}", command[0]),
                    ))
                }
            }
            Ok(output) => match output.status.code() {
                Some(0) => Ok(()),
                code => {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    if stderr.is_empty() {
                        Err(PkexecError::Failed(
                            code.unwrap_or(-1),
                            format!("{} exited with non-zero status", command[0]),
                        ))
                    } else {
                        Err(PkexecError::Failed(code.unwrap_or(-1), stderr))
                    }
                }
            },
        }
    }

    /// Execute a command via pkexec and return Ok(()) or a typed error.
    async fn run_pkexec(command: &[&str]) -> Result<(), PkexecError> {
        let output = tokio::process::Command::new("pkexec")
            .args(command)
            .output()
            .await;

        match output {
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(PkexecError::NotFound)
                } else {
                    Err(PkexecError::Failed(
                        -1,
                        format!("Failed to run pkexec: {e}"),
                    ))
                }
            }
            Ok(output) => match output.status.code() {
                Some(0) => Ok(()),
                Some(126) => Err(PkexecError::AuthCancelled),
                Some(127) => Err(PkexecError::NotAuthorized),
                code => {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    if stderr.is_empty() {
                        Err(PkexecError::Failed(
                            code.unwrap_or(-1),
                            "pkexec exited with non-zero status".into(),
                        ))
                    } else {
                        Err(PkexecError::Failed(code.unwrap_or(-1), stderr))
                    }
                }
            },
        }
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
                let Some(adapter) = this
                    .read_with(cx, |app, _| app.bt_state.adapter.clone())
                    .unwrap_or(None) else { continue };
                match cmd {
                    BluetoothPageCommand::ToggleScan => {
                        let _ = this.update_in(cx, |this, window, cx| {
                            if this.bt_state.discovering {
                                this.bt_state.discovering = false;
                            } else {
                                this.bt_state.discovering = true;
                                cx.spawn_in(window, async move |this, cx| {
                                    crate::backend::bluetooth::discovery::run_discovery(this, cx).await;
                                })
                                .detach();
                            }
                            this.bluetooth_page
                                .update(cx, |page, cx| page.sync_state(&this.bt_state, cx));
                            cx.notify();
                        });
                    }
                    BluetoothPageCommand::DeviceAction { addr, action } => {
                        // Dispatch D-Bus work as a detached background task —
                        // never block the command loop awaiting BlueZ.
                        let a = adapter.clone();
                        let _ = this.update_in(cx, |this, window, cx| {
                            if action == DeviceRowAction::PairAndTrust {
                                eprintln!("[bluetooth] dispatching PairAndTrust for {addr}");

                                // Set initial status immediately (synchronous).
                                if let Some(existing) =
                                    this.bt_state.devices.iter_mut().find(|d| d.address == addr)
                                {
                                    existing.pairing_status =
                                        Some(PairingStatus::Connecting);
                                }
                                this.bluetooth_page.update(cx, |page, cx| {
                                    page.sync_state(&this.bt_state, cx);
                                });
                                cx.notify();

                                // Spawn the full chain as a background task.
                                let a = adapter.clone();
                                cx.spawn_in(window, async move |this, cx| {
                                    Self::execute_pair_and_trust(&a, addr, &this, cx).await;
                                })
                                .detach();
                            } else {
                                // For Connect, show a transient loading state
                                // on the row while the D-Bus call is in flight.
                                if action == DeviceRowAction::Connect {
                                    if let Some(existing) = this
                                        .bt_state
                                        .devices
                                        .iter_mut()
                                        .find(|d| d.address == addr)
                                    {
                                        existing.pairing_status =
                                            Some(PairingStatus::Connecting);
                                    }
                                    this.bluetooth_page.update(cx, |page, cx| {
                                        page.sync_state(&this.bt_state, cx);
                                    });
                                    cx.notify();
                                }

                                // Spawn simple action + post-action refresh.
                                let a2 = a.clone();
                                cx.spawn_in(window, async move |this, cx| {
                                    let a3 = a2.clone();
                                    let action_result = crate::tokio_task(async move {
                                        execute_device_action(&a3, addr, action).await
                                    })
                                    .await;

                                    match action_result {
                                        Ok(Ok(())) => {
                                            // Success — no banner needed.
                                        }
                                        Ok(Err(e)) => {
                                            eprintln!(
                                                "[bluetooth] {action:?} for {addr}: {e}"
                                            );
                                            let _ = this.update_in(cx, |this, _window, cx| {
                                                this.bluetooth_page.update(cx, |page, cx| {
                                                    page.show_error(e, cx);
                                                });
                                            });
                                        }
                                        Err(_) => {
                                            eprintln!(
                                                "[bluetooth] {action:?} for {addr}: tokio task cancelled"
                                            );
                                        }
                                    }

                                    Self::refresh_device_after_action(&a2, addr, &this, cx).await;
                                })
                                .detach();
                            }
                        });
                    }
                }
            }
        })
        .detach();
    }
}

impl Focusable for BludioApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BludioApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_page = self.active_page;

        let theme = crate::ui::theme::theme(cx);
        let colors = &theme.colors;

        h_flex()
            .size_full()
            .items_stretch()
            .font_family(settings(cx).font_family.clone())
            .bg(colors.background)
            .text_color(colors.text)
            // ── Left tab bar ──
            .child(self.tab_bar.clone())
            // ── Right content area ──
            .child(
                v_flex()
                    .flex_1()
                    .child(match active_page {
                        Page::BluetoothDevices => self.bluetooth_page.clone().into_any_element(),
                        Page::AudioOutputs => self.audio_output_page.clone().into_any_element(),
                        Page::AudioInputs => self.audio_input_page.clone().into_any_element(),
                        Page::Configuration => self.configuration_page.clone().into_any_element(),
                        Page::Settings => self.settings_page.clone().into_any_element(),
                        #[cfg(debug_assertions)]
                        Page::DevTest => self.dev_test_page.clone().into_any_element(),
                    })
                    .into_any_element(),
            )
    }
}
