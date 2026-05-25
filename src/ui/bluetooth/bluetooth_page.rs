//! Bluetooth page entity: header (scan button) + error/loading states + device list.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<BluetoothPage> → Vec<Entity<BluetoothDeviceRow>>

use crate::bluetooth::BluetoothState;
use crate::ui::bluetooth::BluetoothPageCommand;
use crate::ui::bluetooth::device_row::BluetoothDeviceRow;
use crate::ui::components::page_header::page_header;
use crate::ui::{h_flex, v_flex};
use futures::channel::mpsc::UnboundedSender;
use gpui::{
    Context, CursorStyle, Entity, MouseButton, MouseUpEvent, Render, Window, div, prelude::*, px,
};

// ── Page entity ────────────────────────────────────────────────────────────

/// Owns a list of `BluetoothDeviceRow` entities and coordinates their state.
pub(crate) struct BluetoothPage {
    rows: Vec<Entity<BluetoothDeviceRow>>,
    discovering: bool,
    error: Option<String>,
    initialized: bool,
    cmd_tx: UnboundedSender<BluetoothPageCommand>,
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
                    .child(
                        // Scan / Stop button
                        div()
                            .id("scan-btn")
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(if discovering {
                                colors.danger
                            } else {
                                colors.accent
                            })
                            .cursor(CursorStyle::PointingHand)
                            .hover(|el| {
                                el.bg(if discovering {
                                    colors.danger_hover
                                } else {
                                    colors.accent_hover
                                })
                            })
                            .child(if discovering { "stop" } else { "scan" })
                            .on_mouse_up(MouseButton::Left, {
                                let cmd_tx = cmd_tx.clone();
                                move |_: &MouseUpEvent, _window, _app| {
                                    let _ = cmd_tx.unbounded_send(BluetoothPageCommand::ToggleScan);
                                }
                            }),
                    ),
            )
            // ── Error banner ──
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
                    div()
                        .id("device-list")
                        .flex_1()
                        .overflow_y_scroll()
                        .when(self.rows.is_empty(), |el| {
                            el.child(
                                h_flex()
                                    .justify_center()
                                    .h(px(200.0))
                                    .text_color(colors.text_secondary)
                                    .child("No devices. Press \"scan\" to discover."),
                            )
                        })
                        .children(self.rows.clone()),
                )
            })
    }
}
