//! Bluetooth device row entity.
//!
//! Each row displays one Bluetooth device (name, address, status) and
//! conditional action buttons (Connect, Disconnect, Forget, Pair & Trust).
//! Actions are sent as `BluetoothPageCommand`s through a channel — the row
//! never spawns async tasks directly.

use bluer::Address;
use futures::channel::mpsc::UnboundedSender;
use gpui::{
    Context, CursorStyle, FontWeight, MouseButton, MouseUpEvent, Render, SharedString, Window, div,
    hsla, prelude::*, px,
};

use super::BluetoothPageCommand;
use crate::bluetooth::device::DeviceRowAction;
use crate::ui::{h_flex, v_flex};

// ── Row entity ─────────────────────────────────────────────────────────────

/// A single Bluetooth device row in the list.
///
/// Unlike `AudioDeviceRow`, this row currently has no interactive sub-entities
/// (TextField, Dropdown, etc.) — only simple action buttons with `on_mouse_up`.
/// The `Focusable` impl is omitted intentionally; it will be added when
/// interactive children are introduced.
pub(crate) struct BluetoothDeviceRow {
    pub(crate) address: Address,
    display_name: String,
    paired: bool,
    connected: bool,
    cmd_tx: UnboundedSender<BluetoothPageCommand>,
}

impl BluetoothDeviceRow {
    pub(crate) fn new(
        address: Address,
        display_name: String,
        paired: bool,
        connected: bool,
        cmd_tx: UnboundedSender<BluetoothPageCommand>,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            address,
            display_name,
            paired,
            connected,
            cmd_tx,
        }
    }

    /// Update the device state from a `BluetoothDevice`.
    pub(crate) fn update_from_device(
        &mut self,
        device: &crate::bluetooth::device::BluetoothDevice,
    ) {
        self.display_name = device.display_name.clone();
        self.paired = device.paired;
        self.connected = device.connected;
    }
}

// ── Render ─────────────────────────────────────────────────────────────────

const ROW_HEIGHT: f32 = 64.0;

impl Render for BluetoothDeviceRow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let addr = self.address;
        let addr_str = addr.to_string();
        let name = self.display_name.clone();
        let paired = self.paired;
        let connected = self.connected;

        let text_secondary = hsla(0.0, 0.0, 0.6, 1.0);
        let accent = hsla(210.0 / 360.0, 0.7, 0.55, 1.0);
        let accent_hover = hsla(210.0 / 360.0, 0.7, 0.45, 1.0);
        let danger = hsla(0.0, 0.7, 0.55, 1.0);
        let danger_hover = hsla(0.0, 0.7, 0.45, 1.0);
        let success = hsla(140.0 / 360.0, 0.6, 0.5, 1.0);

        let cmd_tx = self.cmd_tx.clone();

        h_flex()
            .justify_between()
            .id(SharedString::from(format!("device-{addr}")))
            .px_4()
            .h(px(ROW_HEIGHT))
            .border_b_1()
            .border_color(hsla(0.0, 0.0, 0.20, 1.0))
            .hover(|el| el.bg(hsla(0.0, 0.0, 1.0, 0.04)))
            .child(
                // ── Device info ──
                v_flex()
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .child(SharedString::from(name)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_secondary)
                            .child(SharedString::from(addr_str)),
                    )
                    .child(div().text_xs().mt_0p5().child(if connected {
                        div().text_color(success).child("● connected")
                    } else if paired {
                        div().text_color(text_secondary).child("paired")
                    } else {
                        div().text_color(text_secondary).child("discovered")
                    })),
            )
            .child(
                // ── Action buttons ──
                h_flex().gap_1().map(move |mut btn_row| {
                    if paired && !connected {
                        btn_row = btn_row.child(action_btn(
                            "Connect",
                            accent,
                            accent_hover,
                            addr,
                            DeviceRowAction::Connect,
                            &cmd_tx,
                        ));
                    }
                    if connected {
                        btn_row = btn_row.child(action_btn(
                            "Disconnect",
                            danger,
                            danger_hover,
                            addr,
                            DeviceRowAction::Disconnect,
                            &cmd_tx,
                        ));
                    }
                    if paired {
                        btn_row = btn_row.child(action_btn(
                            "Forget",
                            danger,
                            danger_hover,
                            addr,
                            DeviceRowAction::Forget,
                            &cmd_tx,
                        ));
                    }
                    if !paired {
                        btn_row = btn_row.child(action_btn(
                            "Pair & Trust",
                            accent,
                            accent_hover,
                            addr,
                            DeviceRowAction::PairAndTrust,
                            &cmd_tx,
                        ));
                    }
                    btn_row
                }),
            )
    }
}

// ── Button builder ─────────────────────────────────────────────────────────

/// Build a single action button. Clicking sends a `BluetoothPageCommand`
/// through the channel (no async tasks spawned here).
fn action_btn(
    label: &str,
    bg: gpui::Hsla,
    hover_bg: gpui::Hsla,
    addr: Address,
    action: DeviceRowAction,
    cmd_tx: &UnboundedSender<BluetoothPageCommand>,
) -> gpui::Stateful<gpui::Div> {
    let cmd_tx = cmd_tx.clone();
    let action_clone = action;
    div()
        .id(SharedString::from(format!("btn-{addr}-{action:?}")))
        .px_2()
        .py_1()
        .rounded_sm()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .bg(bg)
        .cursor(CursorStyle::PointingHand)
        .hover(move |el| el.bg(hover_bg))
        .child(SharedString::from(label.to_string()))
        .on_mouse_up(MouseButton::Left, move |_: &MouseUpEvent, _window, _app| {
            let _ = cmd_tx.unbounded_send(BluetoothPageCommand::DeviceAction {
                addr,
                action: action_clone,
            });
        })
}
