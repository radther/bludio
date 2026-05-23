//! Bluetooth page: header (scan button) + error/loading states + device list.
//!
//! This is a self-contained page module. The top-level entry point is
//! `bluetooth_page_view()`, which composes all sub-components.

use crate::app::BludioApp;
use crate::bluetooth::BluetoothState;
use crate::ui::{h_flex, v_flex};
use gpui::{
    Context, CursorStyle, Div, FontWeight, MouseButton, MouseUpEvent, SharedString, Stateful, div,
    hsla, prelude::*, px, rgba,
};

pub(crate) const ROW_HEIGHT: f32 = 64.0;

// ── Action type ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Connect,
    Disconnect,
    Forget,
    PairAndTrust,
}

// ── Public entry point ─────────────────────────────────────────────────────

struct ButtonColors {
    accent: gpui::Hsla,
    accent_hover: gpui::Hsla,
    danger: gpui::Hsla,
    danger_hover: gpui::Hsla,
}

/// Render the full Bluetooth page: header, error/loading states, device list.
pub(crate) fn bluetooth_page_view(state: &BluetoothState, cx: &mut Context<BludioApp>) -> Div {
    let discovering = state.discovering;
    let error = state.error.clone();
    let initialized = state.adapter.is_some();
    let surface = hsla(0.0, 0.0, 0.14, 1.0);
    let text_secondary = hsla(0.0, 0.0, 0.6, 1.0);
    let accent = hsla(210.0 / 360.0, 0.7, 0.55, 1.0);
    let accent_hover = hsla(210.0 / 360.0, 0.7, 0.45, 1.0);
    let danger = hsla(0.0, 0.7, 0.55, 1.0);
    let danger_hover = hsla(0.0, 0.7, 0.45, 1.0);

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
        .when(!initialized && state.error.is_none(), |el| {
            el.child(loading_indicator(text_secondary))
        })
        .when(initialized, |el| el.child(device_list_view(state, cx)))
}

// ── Header sub-components ──────────────────────────────────────────────────

fn scan_button(
    discovering: bool,
    danger: gpui::Hsla,
    accent: gpui::Hsla,
    danger_hover: gpui::Hsla,
    accent_hover: gpui::Hsla,
    cx: &mut Context<BludioApp>,
) -> Stateful<Div> {
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
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                if this.bt_state.discovering {
                    this.bt_state.discovering = false;
                } else {
                    this.bt_state.discovering = true;
                    cx.spawn(async move |this, cx| {
                        crate::app::run_discovery(this, cx).await;
                    })
                    .detach();
                }
                cx.notify();
            }),
        )
}

fn error_banner(err: String) -> Div {
    div()
        .px_4()
        .py_2()
        .bg(rgba(0xff3c3c33))
        .text_color(hsla(0.0, 0.8, 0.75, 1.0))
        .child(SharedString::from(format!("Error: {}", err)))
}

fn loading_indicator(text_secondary: gpui::Hsla) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .flex_1()
        .text_color(text_secondary)
        .child("Connecting to Bluetooth...")
}

// ── Device list ────────────────────────────────────────────────────────────

/// Render the full device list: scrollable rows with per-device status
/// badges and conditional action buttons.
fn device_list_view(state: &BluetoothState, cx: &mut Context<BludioApp>) -> Stateful<Div> {
    let devices = &state.devices;
    let text_secondary = hsla(0.0, 0.0, 0.6, 1.0);
    let accent = hsla(210.0 / 360.0, 0.7, 0.55, 1.0);
    let danger = hsla(0.0, 0.7, 0.55, 1.0);
    let colors = ButtonColors {
        accent,
        accent_hover: hsla(210.0 / 360.0, 0.7, 0.45, 1.0),
        danger,
        danger_hover: hsla(0.0, 0.7, 0.45, 1.0),
    };
    let success = hsla(140.0 / 360.0, 0.6, 0.5, 1.0);

    div()
        .id("device-list")
        .flex_1()
        .overflow_y_scroll()
        .children(devices.iter().map(|device| {
            let addr = device.address;
            let name = device.display_name.clone();
            let addr_str = addr.to_string();
            let paired = device.paired;
            let connected = device.connected;

            h_flex()
                .justify_between()
                .id(SharedString::from(format!("device-{addr}")))
                .px_4()
                .h(px(ROW_HEIGHT))
                .border_b_1()
                .border_color(hsla(0.0, 0.0, 0.20, 1.0))
                .hover(|el| el.bg(hsla(0.0, 0.0, 1.0, 0.04)))
                .child(device_info(
                    name,
                    addr_str,
                    paired,
                    connected,
                    text_secondary,
                    success,
                ))
                .child(device_buttons(addr, paired, connected, &colors, cx))
                .into_any_element()
        }))
        .when(devices.is_empty(), |el| {
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
}

// ── Row sub-components ─────────────────────────────────────────────────────

fn device_info(
    name: String,
    addr_str: String,
    paired: bool,
    connected: bool,
    text_secondary: gpui::Hsla,
    success: gpui::Hsla,
) -> Div {
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
        }))
}

fn device_buttons(
    addr: bluer::Address,
    paired: bool,
    connected: bool,
    colors: &ButtonColors,
    cx: &mut Context<BludioApp>,
) -> Div {
    h_flex().gap_1().map(|mut btn_row| {
        if paired && !connected {
            btn_row = btn_row.child(action_btn(
                "Connect",
                colors.accent,
                colors.accent_hover,
                addr,
                Action::Connect,
                cx,
            ));
        }
        if connected {
            btn_row = btn_row.child(action_btn(
                "Disconnect",
                colors.danger,
                colors.danger_hover,
                addr,
                Action::Disconnect,
                cx,
            ));
        }
        if paired {
            btn_row = btn_row.child(action_btn(
                "Forget",
                colors.danger,
                colors.danger_hover,
                addr,
                Action::Forget,
                cx,
            ));
        }
        if !paired {
            btn_row = btn_row.child(action_btn(
                "Pair & Trust",
                colors.accent,
                colors.accent_hover,
                addr,
                Action::PairAndTrust,
                cx,
            ));
        }
        btn_row
    })
}

// ── Button builder ─────────────────────────────────────────────────────────

fn action_btn(
    label: &str,
    bg: gpui::Hsla,
    hover_bg: gpui::Hsla,
    addr: bluer::Address,
    action: Action,
    cx: &mut Context<BludioApp>,
) -> Stateful<Div> {
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
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |this, _: &MouseUpEvent, _window, cx| {
                let adapter = match &this.bt_state.adapter {
                    Some(a) => a.clone(),
                    None => return,
                };
                cx.spawn(async move |this, cx| {
                    crate::actions::execute(adapter, addr, action, this, cx).await;
                })
                .detach();
            }),
        )
}
