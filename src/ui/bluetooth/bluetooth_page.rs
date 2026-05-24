//! Bluetooth page entity: header (scan button) + error/loading states + device list.
//!
//! Ownership chain:
//!   BludioApp → Entity<BluetoothPage> → Vec<Entity<BluetoothDeviceRow>>

use crate::bluetooth::BluetoothState;
use crate::ui::bluetooth::BluetoothPageCommand;
use crate::ui::bluetooth::device_row::BluetoothDeviceRow;
use crate::ui::{h_flex, v_flex};
use futures::channel::mpsc::UnboundedSender;
use gpui::{
    Context, CursorStyle, Entity, FontWeight, MouseButton, MouseUpEvent, Render, Window, div, hsla,
    prelude::*, px,
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
    pub(crate) fn sync_state(
        &mut self,
        state: &BluetoothState,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.discovering = state.discovering;
        self.error = state.error.clone();
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let surface = hsla(0.0, 0.0, 0.14, 1.0);
        let text_secondary = hsla(0.0, 0.0, 0.6, 1.0);
        let accent = hsla(210.0 / 360.0, 0.7, 0.55, 1.0);
        let accent_hover = hsla(210.0 / 360.0, 0.7, 0.45, 1.0);
        let danger = hsla(0.0, 0.7, 0.55, 1.0);
        let danger_hover = hsla(0.0, 0.7, 0.45, 1.0);

        let discovering = self.discovering;
        let initialized = self.initialized;

        let cmd_tx = self.cmd_tx.clone();

        v_flex()
            .flex_1()
            // ── Bluetooth header ──
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .bg(surface)
                    .border_b_1()
                    .border_color(hsla(0.0, 0.0, 0.25, 1.0))
                    .child(div().font_weight(FontWeight::BOLD).child("bluetooth"))
                    .child(
                        // Scan / Stop button
                        div()
                            .id("scan-btn")
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(if discovering { danger } else { accent })
                            .cursor(CursorStyle::PointingHand)
                            .hover(|el| {
                                el.bg(if discovering {
                                    danger_hover
                                } else {
                                    accent_hover
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
                        .bg(gpui::rgba(0xff3c3c33))
                        .text_color(hsla(0.0, 0.8, 0.75, 1.0))
                        .child(gpui::SharedString::from(format!("Error: {}", err))),
                )
            })
            // ── Loading indicator ──
            .when(!initialized && self.error.is_none(), |el| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_1()
                        .text_color(text_secondary)
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
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .h(px(200.0))
                                    .text_color(text_secondary)
                                    .child("No devices. Press \"scan\" to discover."),
                            )
                        })
                        .children(self.rows.clone()),
                )
            })
    }
}
