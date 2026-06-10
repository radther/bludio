//! Bluetooth device row entity.
//!
//! Each row displays one Bluetooth device (name, address, status) and
//! conditional action buttons (Connect, Disconnect, Forget, Pair & Trust).
//! Actions are sent as `BluetoothPageCommand`s through a channel — the row
//! never spawns async tasks directly.

use crate::backend::bluetooth::device::{DeviceRowAction, PairingStatus};
use crate::ui::StyledExt;
use crate::ui::components::button::action_btn;
use crate::ui::components::loading_bar::loading_bar;
use crate::ui::components::status_strip::status_strip;
use crate::ui::pages::bluetooth::BluetoothPageCommand;
use crate::ui::theme::text_styles;
use crate::ui::{h_flex, v_flex};
use bluer::Address;
use futures::channel::mpsc::UnboundedSender;
use gpui::{Context, Render, SharedString, Window, div, prelude::*};

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
        device: &crate::backend::bluetooth::device::BluetoothDevice,
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
        let text_styles = text_styles(cx);

        let pairing_status = self.pairing_status.as_ref();

        let status_color = if let Some(ref status) = pairing_status {
            match status {
                PairingStatus::Connecting => colors.bluetooth_accent,
                PairingStatus::Pairing => colors.warning,
                PairingStatus::Trusting => colors.success,
                PairingStatus::Failed(_) => colors.danger,
            }
        } else if connected {
            colors.success
        } else {
            colors.text_secondary
        };

        let is_connecting = matches!(
            pairing_status,
            Some(PairingStatus::Connecting)
                | Some(PairingStatus::Pairing)
                | Some(PairingStatus::Trusting)
        );

        let cmd_tx = self.cmd_tx.clone();

        h_flex()
            .items_stretch()
            .id(SharedString::from(format!("device-{addr}")))
            .hover(|el| el.bg(colors.background_hover))
            .child(status_strip(status_color))
            .child(
                v_flex()
                    .flex_1()
                    .child(
                        h_flex()
                            .justify_between()
                            .px_4()
                            .child(
                                // ── Device info ──
                                v_flex()
                                    .child(div().styled(text_styles.caption).mt_0p5().child(
                                        if let Some(ref status) = pairing_status {
                                            div().text_color(status_color).child(status.to_string())
                                        } else if connected {
                                            div().text_color(status_color).child("connected")
                                        } else if paired {
                                            div().text_color(status_color).child("paired")
                                        } else {
                                            div().text_color(status_color).child("discovered")
                                        },
                                    ))
                                    .child(
                                        div()
                                            .styled(text_styles.body)
                                            .child(SharedString::from(name)),
                                    )
                                    .child(
                                        div()
                                            .styled(text_styles.body2)
                                            .text_color(colors.text_muted)
                                            .child(SharedString::from(addr_str)),
                                    ),
                            )
                            .child(
                                // ── Action buttons ──
                                h_flex().gap_1().map(move |mut btn_row| {
                                    if paired && !connected && !is_connecting {
                                        btn_row = btn_row.child(action_btn(
                                            format!("btn-{addr}-Connect"),
                                            "Connect",
                                            colors.bluetooth_accent,
                                            colors.bluetooth_accent,
                                            colors.text_colored_button,
                                            {
                                                let cmd_tx = cmd_tx.clone();
                                                move || {
                                                    let _ = cmd_tx.unbounded_send(
                                                        BluetoothPageCommand::DeviceAction {
                                                            addr,
                                                            action: DeviceRowAction::Connect,
                                                        },
                                                    );
                                                }
                                            },
                                            &text_styles,
                                        ));
                                    }
                                    if connected {
                                        btn_row = btn_row.child(action_btn(
                                            format!("btn-{addr}-Disconnect"),
                                            "Disconnect",
                                            colors.danger,
                                            colors.danger,
                                            colors.text_colored_button,
                                            {
                                                let cmd_tx = cmd_tx.clone();
                                                move || {
                                                    let _ = cmd_tx.unbounded_send(
                                                        BluetoothPageCommand::DeviceAction {
                                                            addr,
                                                            action: DeviceRowAction::Disconnect,
                                                        },
                                                    );
                                                }
                                            },
                                            &text_styles,
                                        ));
                                    }
                                    if paired && !is_connecting {
                                        btn_row = btn_row.child(action_btn(
                                            format!("btn-{addr}-Forget"),
                                            "Forget",
                                            colors.danger,
                                            colors.danger,
                                            colors.text_colored_button,
                                            {
                                                let cmd_tx = cmd_tx.clone();
                                                move || {
                                                    let _ = cmd_tx.unbounded_send(
                                                        BluetoothPageCommand::DeviceAction {
                                                            addr,
                                                            action: DeviceRowAction::Forget,
                                                        },
                                                    );
                                                }
                                            },
                                            &text_styles,
                                        ));
                                    }
                                    if !paired && pairing_status.is_none() {
                                        btn_row = btn_row.child(action_btn(
                                            format!("btn-{addr}-PairAndTrust"),
                                            "Pair & Trust",
                                            colors.bluetooth_accent,
                                            colors.bluetooth_accent,
                                            colors.text_colored_button,
                                            {
                                                let cmd_tx = cmd_tx.clone();
                                                move || {
                                                    let _ = cmd_tx.unbounded_send(
                                                        BluetoothPageCommand::DeviceAction {
                                                            addr,
                                                            action: DeviceRowAction::PairAndTrust,
                                                        },
                                                    );
                                                }
                                            },
                                            &text_styles,
                                        ));
                                    }
                                    btn_row
                                }),
                            ),
                    )
                    .child(div().h_1().when(is_connecting, |el| {
                        el.child(loading_bar(format!("device-{addr}-loading"), status_color))
                    })),
            )
    }
}
