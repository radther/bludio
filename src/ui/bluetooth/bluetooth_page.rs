//! Bluetooth page entity: header (scan button) + error/loading states + device list.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<BluetoothPage> → Vec<Entity<BluetoothDeviceRow>>

use std::time::Duration;

use crate::bluetooth::BluetoothState;
use crate::ui::bluetooth::BluetoothPageCommand;
use crate::ui::bluetooth::device_row::BluetoothDeviceRow;
use crate::ui::components::error_banner::error_banner;
use crate::ui::components::page_header::page_header;
use crate::ui::icons;
use crate::ui::{h_flex, v_flex};
use futures::channel::mpsc::UnboundedSender;
use gpui::{ClickEvent, Context, CursorStyle, Entity, Render, Window, div, prelude::*, px};

// ── Page entity ────────────────────────────────────────────────────────────

/// Owns a list of `BluetoothDeviceRow` entities and coordinates their state.
pub(crate) struct BluetoothPage {
    rows: Vec<Entity<BluetoothDeviceRow>>,
    discovering: bool,
    error: Option<String>,
    initialized: bool,
    cmd_tx: UnboundedSender<BluetoothPageCommand>,
    /// Error banner text — `None` when banner is hidden.
    error_banner_text: Option<String>,
    /// Generation counter for banner animation (incremented on each new error).
    error_banner_generation: u64,
}

impl BluetoothPage {
    /// Create a new page entity with the given command channel.
    pub(crate) fn new(
        cmd_tx: UnboundedSender<BluetoothPageCommand>,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            rows: Vec::new(),
            discovering: false,
            error: None,
            initialized: false,
            cmd_tx,
            error_banner_text: None,
            error_banner_generation: 0,
        }
    }

    /// Sync rows with the latest Bluetooth state.
    /// Creates/updates/removes rows to match the device list, and caches
    /// the discovering/error/initialized flags.
    pub(crate) fn sync_state(&mut self, state: &BluetoothState, cx: &mut Context<Self>) {
        self.discovering = state.discovering;
        self.error.clone_from(&state.error);
        self.initialized = state.adapter.is_some();

        let cmd_tx = self.cmd_tx.clone();

        for device in &state.devices {
            if let Some(row) = self
                .rows
                .iter()
                .find(|r| r.read(cx).address == device.address)
            {
                row.update(cx, |row, _cx| {
                    row.update_from_device(device);
                });
            } else {
                let row = cx.new(|row_cx| {
                    BluetoothDeviceRow::new(
                        device.address,
                        device.display_name.clone(),
                        device.paired,
                        device.connected,
                        device.pairing_status.clone(),
                        cmd_tx.clone(),
                        row_cx,
                    )
                });
                self.rows.push(row);
            }
        }
        self.rows.retain(|r| {
            state
                .devices
                .iter()
                .any(|d| d.address == r.read(cx).address)
        });
    }

    /// Show an error message in the banner. Increments generation to restart animation.
    /// Spawns a timer to auto-dismiss after 5 seconds.
    pub(crate) fn show_error(&mut self, msg: String, cx: &mut Context<Self>) {
        self.error_banner_text = Some(msg);
        self.error_banner_generation += 1;
        cx.notify();

        // Auto-dismiss after 5 seconds.
        let entity = cx.entity().clone();
        cx.spawn(async move |_, cx| {
            cx.background_executor().timer(Duration::from_secs(5)).await;
            entity.update(cx, |page, cx| {
                page.clear_error(cx);
            });
        })
        .detach();
    }

    /// Clear the error banner.
    pub(crate) fn clear_error(&mut self, cx: &mut Context<Self>) {
        self.error_banner_text = None;
        cx.notify();
    }
}

impl Render for BluetoothPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::ui::theme::theme(cx);
        let colors = &theme.colors;
        let text_styles = &theme.text_styles;

        let discovering = self.discovering;
        let initialized = self.initialized;
        let connected_count = self.rows.iter().filter(|r| r.read(cx).connected).count();

        let cmd_tx = self.cmd_tx.clone();

        // Capture banner state for the render closure.
        let banner_text = self.error_banner_text.clone();
        let banner_generation = self.error_banner_generation;

        v_flex()
            .flex_1()
            // ── Bluetooth header ──
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .child(page_header(
                        "Bluetooth",
                        format!(
                            "{} device{}, {} connected",
                            self.rows.len(),
                            if self.rows.len() == 1 { "" } else { "s" },
                            connected_count,
                        ),
                        colors,
                        text_styles,
                    ))
                    .child({
                        let (btn_bg, btn_hover, icon_color) = if discovering {
                            (colors.danger, colors.danger, colors.text_colored_button)
                        } else {
                            (
                                colors.element_background,
                                colors.element_hover,
                                colors.text_secondary,
                            )
                        };
                        h_flex()
                            .id("scan-btn")
                            .justify_center()
                            .w(px(48.0))
                            .h(px(48.0))
                            .rounded_lg()
                            .bg(btn_bg)
                            .cursor(CursorStyle::PointingHand)
                            .hover(move |el| el.bg(btn_hover))
                            .child(
                                icons::bluetooth()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .text_color(icon_color),
                            )
                            .on_click({
                                let cmd_tx = cmd_tx.clone();
                                move |_: &ClickEvent, _window, _app| {
                                    let _ = cmd_tx.unbounded_send(BluetoothPageCommand::ToggleScan);
                                }
                            })
                    }),
            )
            // ── Bluetooth initialization error ──
            .when_some(self.error.clone(), |el, err| {
                el.child(
                    div()
                        .px_4()
                        .py_2()
                        .bg(colors.error_background)
                        .text_color(colors.danger)
                        .child(gpui::SharedString::from(format!("Error: {err}"))),
                )
            })
            // ── Loading indicator ──
            .when(!initialized && self.error.is_none(), |el| {
                el.child(
                    h_flex()
                        .justify_center()
                        .flex_1()
                        .text_color(colors.text_secondary)
                        .child("Connecting to Bluetooth..."),
                )
            })
            // ── Device list ──
            .when(initialized, |el| {
                el.child(
                    v_flex()
                        .gap_2()
                        .id("device-list")
                        .flex_1()
                        .overflow_y_scroll()
                        .when(self.rows.is_empty(), |el| {
                            el.child(
                                h_flex()
                                    .justify_center()
                                    .text_color(colors.text_secondary)
                                    .child("No devices. Press \"scan\" to discover."),
                            )
                        })
                        .children(self.rows.clone()),
                )
            })
            // ── Action error banner (slides up from bottom) ──
            .when_some(banner_text, |el, text| {
                let entity = cx.entity().clone();
                el.child(error_banner(
                    &text,
                    banner_generation,
                    colors,
                    text_styles,
                    move |_, _window, cx| {
                        entity.update(cx, |page, cx| {
                            page.clear_error(cx);
                        });
                    },
                ))
            })
    }
}
