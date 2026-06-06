//! Bluetooth page entity: header (scan button) + error/loading states + device list.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<BluetoothPage> → Vec<Entity<BluetoothDeviceRow>>

use std::time::Duration;

use crate::backend::bluetooth::BluetoothState;
use crate::backend::subsystem::SubsystemStatus;
use crate::ui::common::animation::FadeInAnimationExt;
use crate::ui::common::icons;
use crate::ui::{h_flex, v_flex};
use crate::ui::components::error_banner::error_banner;
use crate::ui::components::loading_bar::loading_bar;
use crate::ui::components::page_header::page_header;
use crate::ui::pages::bluetooth::BluetoothPageCommand;
use crate::ui::pages::bluetooth::bluetooth_device_row::BluetoothDeviceRow;
use futures::channel::mpsc::UnboundedSender;
use gpui::{ClickEvent, Context, CursorStyle, Entity, Render, Window, div, prelude::*, px};

// ── Page entity ────────────────────────────────────────────────────────────

/// Owns a list of `BluetoothDeviceRow` entities and coordinates their state.
pub(crate) struct BluetoothPage {
    rows: Vec<Entity<BluetoothDeviceRow>>,
    discovering: bool,
    subsystem_status: SubsystemStatus,
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
            subsystem_status: SubsystemStatus::Connecting,
            cmd_tx,
            error_banner_text: None,
            error_banner_generation: 0,
        }
    }

    /// Sync rows with the latest Bluetooth state.
    /// Creates/updates/removes rows to match the device list, and caches
    /// the discovering and subsystem_status flags.
    pub(crate) fn sync_state(&mut self, state: &BluetoothState, cx: &mut Context<Self>) {
        self.discovering = state.discovering;
        self.subsystem_status = state.subsystem_status.clone();

        // Only sync device rows when connected.
        if self.subsystem_status != SubsystemStatus::Connected {
            self.rows.clear();
            return;
        }

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
        let connected_count = self.rows.iter().filter(|r| r.read(cx).connected).count();

        let cmd_tx = self.cmd_tx.clone();

        // Capture banner state for the render closure.
        let banner_text = self.error_banner_text.clone();
        let banner_generation = self.error_banner_generation;

        // Scan button is interactive only when Connected.
        let scan_enabled = self.subsystem_status == SubsystemStatus::Connected;

        v_flex()
            .flex_1()
            // ── Bluetooth header ──
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .pt_2()
                    .pb(px(0.0))
                    .child(page_header(
                        "Bluetooth",
                        match &self.subsystem_status {
                            SubsystemStatus::Connected => format!(
                                "{} device{}, {} connected",
                                self.rows.len(),
                                if self.rows.len() == 1 { "" } else { "s" },
                                connected_count,
                            ),
                            _ => String::new(),
                        },
                        colors,
                        text_styles,
                    ))
                    .child({
                        let (btn_bg, btn_hover, icon_color) = if discovering {
                            (colors.danger, colors.danger, colors.text_colored_button)
                        } else if !scan_enabled {
                            (
                                colors.element_background,
                                colors.element_background,
                                colors.text_secondary.opacity(0.4),
                            )
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
                            .cursor(if scan_enabled {
                                CursorStyle::PointingHand
                            } else {
                                CursorStyle::Arrow
                            })
                            .hover(move |el| el.bg(btn_hover))
                            .child(
                                icons::bluetooth()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .text_color(icon_color),
                            )
                            .when(scan_enabled, |el| {
                                el.on_click({
                                    let cmd_tx = cmd_tx.clone();
                                    move |_: &ClickEvent, _window, _app| {
                                        let _ =
                                            cmd_tx.unbounded_send(BluetoothPageCommand::ToggleScan);
                                    }
                                })
                            })
                    })
                    .with_fade_in_up("bt-header", 1),
            )
            // ── Scanning loading bar ──
            .child(div().h_1().when(self.discovering, |el| {
                el.child(loading_bar(
                    "bluetooth-scan-loading-bar",
                    colors.bluetooth_accent,
                ))
            }))
            // ── Subsystem status content area ──
            .child(match &self.subsystem_status {
                SubsystemStatus::Connecting => h_flex()
                    .justify_center()
                    .flex_1()
                    .text_color(colors.text_secondary)
                    .child("Connecting to Bluetooth...")
                    .with_fade_in_up("bt-connecting", 2)
                    .into_any_element(),
                SubsystemStatus::Disconnected(msg) => h_flex()
                    .justify_center()
                    .flex_1()
                    .text_color(colors.danger)
                    .child(format!("Error: {msg}"))
                    .with_fade_in_up("bt-disconnected", 2)
                    .into_any_element(),
                SubsystemStatus::Reconnecting => h_flex()
                    .justify_center()
                    .flex_1()
                    .text_color(colors.text_secondary)
                    .child("Reconnecting to Bluetooth...")
                    .with_fade_in_up("bt-reconnecting", 2)
                    .into_any_element(),
                SubsystemStatus::Connected => {
                    // ── Device list ──
                    v_flex()
                        .gap_2()
                        .mt(px(4.0))
                        .id("device-list")
                        .flex_1()
                        .overflow_y_scroll()
                        .when(self.rows.is_empty(), |el| {
                            el.child(
                                h_flex()
                                    .justify_center()
                                    .text_color(colors.text_secondary)
                                    .child("No devices. Press \"scan\" to discover.")
                                    .with_fade_in_up("bt-empty", 2),
                            )
                        })
                        .children(self.rows.iter().enumerate().map(|(i, row)| {
                            div()
                                .child(row.clone())
                                .with_fade_in_up(format!("bt-row-{i}"), i + 2)
                                .into_any_element()
                        }))
                        .into_any_element()
                }
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
