//! Bluetooth device row entity.
//!
//! Each row displays one Bluetooth device (name, address, status) and
//! conditional action buttons (Connect, Disconnect, Forget, Pair & Trust).
//! Actions are sent as `BluetoothPageCommand`s through a channel — the row
//! never spawns async tasks directly.

use crate::ui::StyledExt;
use bluer::Address;
use futures::channel::mpsc::UnboundedSender;
use gpui::{
    Context, CursorStyle, MouseButton, MouseUpEvent, Render, SharedString, Window, div, prelude::*,
};

use super::BluetoothPageCommand;
use crate::bluetooth::device::{DeviceRowAction, PairingStatus};
use crate::ui::{h_flex, v_flex};

// ── Row entity ─────────────────────────────────────────────────────────────

/// A single Bluetooth device row in the list.
///
/// Unlike `AudioDeviceRow`, this row currently has no interactive sub-entities
/// (`TextField`, Dropdown, etc.) — only simple action buttons with `on_mouse_up`.
/// The `Focusable` impl is omitted intentionally; it will be added when
/// interactive children are introduced.
pub(crate) struct BluetoothDeviceRow {
    pub(crate) address: Address,
    display_name: String,
    paired: bool,
    pub(crate) connected: bool,
    pairing_status: Option<PairingStatus>,
    cmd_tx: UnboundedSender<BluetoothPageCommand>,
}

impl BluetoothDeviceRow {
    pub(crate) fn new(
        address: Address,
        display_name: String,
        paired: bool,
        connected: bool,
        pairing_status: Option<PairingStatus>,
        cmd_tx: UnboundedSender<BluetoothPageCommand>,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            address,
            display_name,
            paired,
            connected,
            pairing_status,
            cmd_tx,
        }
    }

    /// Update the device state from a `BluetoothDevice`.
    pub(crate) fn update_from_device(
        &mut self,
        device: &crate::bluetooth::device::BluetoothDevice,
    ) {
        self.display_name.clone_from(&device.display_name);
        self.paired = device.paired;
        self.connected = device.connected;
        self.pairing_status.clone_from(&device.pairing_status);
    }
}

// ── Render ─────────────────────────────────────────────────────────────────

impl Render for BluetoothDeviceRow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let addr = self.address;
        let addr_str = addr.to_string();
        let name = self.display_name.clone();
        let paired = self.paired;
        let connected = self.connected;

        let colors = &crate::ui::theme::theme(cx).colors;
        let text_styles = &crate::ui::theme::theme(cx).text_styles;

        let pairing_status = self.pairing_status.as_ref();

        let cmd_tx = self.cmd_tx.clone();

        h_flex()
            .justify_between()
            .id(SharedString::from(format!("device-{addr}")))
            .px_4()
            .border_b_1()
            .border_color(colors.border_subtle)
            .hover(|el| el.bg(colors.hover_overlay))
            .child(
                // ── Device info ──
                v_flex()
                    .child(
                        div()
                            .styled(text_styles.body)
                            .child(SharedString::from(name)),
                    )
                    .child(
                        div()
                            .styled(text_styles.caption)
                            .text_color(colors.text_secondary)
                            .child(SharedString::from(addr_str)),
                    )
                    .child(div().styled(text_styles.caption).mt_0p5().child(
                        if let Some(ref status) = pairing_status {
                            let (dot_color, label): (gpui::Hsla, String) = match status {
                                PairingStatus::Connecting => (colors.accent, status.to_string()),
                                PairingStatus::Pairing => (colors.warning, status.to_string()),
                                PairingStatus::Trusting => (colors.success, status.to_string()),
                                PairingStatus::Failed(_) => (colors.danger, status.to_string()),
                            };
                            div().text_color(dot_color).child(format!("● {label}"))
                        } else if connected {
                            div().text_color(colors.success).child("● connected")
                        } else if paired {
                            div().text_color(colors.text_secondary).child("paired")
                        } else {
                            div().text_color(colors.text_secondary).child("discovered")
                        },
                    )),
            )
            .child(
                // ── Action buttons ──
                h_flex().gap_1().map(move |mut btn_row| {
                    if paired && !connected {
                        btn_row = btn_row.child(action_btn(
                            "Connect",
                            colors.accent,
                            colors.accent_hover,
                            addr,
                            DeviceRowAction::Connect,
                            &cmd_tx,
                            text_styles,
                        ));
                    }
                    if connected {
                        btn_row = btn_row.child(action_btn(
                            "Disconnect",
                            colors.danger,
                            colors.danger_hover,
                            addr,
                            DeviceRowAction::Disconnect,
                            &cmd_tx,
                            text_styles,
                        ));
                    }
                    if paired {
                        btn_row = btn_row.child(action_btn(
                            "Forget",
                            colors.danger,
                            colors.danger_hover,
                            addr,
                            DeviceRowAction::Forget,
                            &cmd_tx,
                            text_styles,
                        ));
                    }
                    if !paired && pairing_status.is_none() {
                        btn_row = btn_row.child(action_btn(
                            "Pair & Trust",
                            colors.accent,
                            colors.accent_hover,
                            addr,
                            DeviceRowAction::PairAndTrust,
                            &cmd_tx,
                            text_styles,
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
    text_styles: &crate::ui::theme::TextStyleSet,
) -> gpui::Stateful<gpui::Div> {
    let cmd_tx = cmd_tx.clone();
    div()
        .id(SharedString::from(format!("btn-{addr}-{action:?}")))
        .px_2()
        .py_1()
        .rounded_sm()
        .styled(text_styles.caption)
        .bg(bg)
        .cursor(CursorStyle::PointingHand)
        .hover(move |el| el.bg(hover_bg))
        .child(SharedString::from(label.to_string()))
        .on_mouse_up(MouseButton::Left, move |_: &MouseUpEvent, _window, _app| {
            let _ = cmd_tx.unbounded_send(BluetoothPageCommand::DeviceAction { addr, action });
        })
}
