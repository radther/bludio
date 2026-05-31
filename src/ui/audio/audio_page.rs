//! Audio device page: shared view for output and input devices.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<AudioPage> → Vec<Entity<AudioDeviceRow>>
//!     → Entity<TextField> + Entity<Dropdown>

use crate::audio::pulse::PaWakeup;
use crate::audio::{AudioCommand, AudioState, DeviceKind};
use crate::subsystem::SubsystemStatus;
use crate::ui::animation::FadeInAnimationExt;
use crate::ui::audio::device_row::AudioDeviceRow;
use crate::ui::components::page_header::page_header;
use crate::ui::{h_flex, v_flex};
use gpui::{Context, Entity, Render, Window, div, prelude::*, px};

// ── Audio page entity ──────────────────────────────────────────────────────

/// Owns a list of `AudioDeviceRow` entities and coordinates their state.
pub(crate) struct AudioPage {
    kind: DeviceKind,
    rows: Vec<Entity<AudioDeviceRow>>,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    wakeup: Option<PaWakeup>,
    subsystem_status: SubsystemStatus,
}

impl AudioPage {
    /// Create a new page entity.
    pub(crate) fn new(
        kind: DeviceKind,
        cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
        wakeup: Option<PaWakeup>,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            kind,
            rows: Vec::new(),
            cmd_tx,
            wakeup,
            subsystem_status: SubsystemStatus::Connecting,
        }
    }

    /// Swap the command sender and wakeup (used after audio reconnect).
    pub(crate) fn update_cmd_tx(
        &mut self,
        cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
        wakeup: Option<PaWakeup>,
    ) {
        self.cmd_tx = cmd_tx;
        self.wakeup = wakeup;
    }

    /// Set the wakeup handle after construction (async arrival).
    pub(crate) fn set_wakeup(&mut self, wakeup: PaWakeup) {
        self.wakeup = Some(wakeup);
    }

    /// Sync rows with the latest audio state. Creates/updates/removes rows
    /// to match the device list.
    pub(crate) fn sync_rows(
        &mut self,
        state: &AudioState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.subsystem_status = state.subsystem_status.clone();

        // Only sync device rows when connected.
        if self.subsystem_status != SubsystemStatus::Connected {
            self.rows.clear();
            return;
        }

        match self.kind {
            DeviceKind::Output => self.sync_output_rows(state, window, cx),
            DeviceKind::Input => self.sync_input_rows(state, window, cx),
        }
    }

    fn sync_output_rows(
        &mut self,
        state: &AudioState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(wakeup) = self.wakeup else {
            return;
        };
        let cmd_tx = self.cmd_tx.clone();

        for sink in &state.sinks {
            if let Some(row) = self.rows.iter().find(|r| r.read(cx).index == sink.index) {
                row.update(cx, |row, row_cx| {
                    row.update_from_sink(sink, row_cx);
                });
            } else {
                let row = cx.new(|row_cx| {
                    AudioDeviceRow::new_sink(sink, cmd_tx.clone(), wakeup, window, row_cx)
                });
                self.rows.push(row);
            }
        }
        self.rows
            .retain(|r| state.sinks.iter().any(|s| s.index == r.read(cx).index));
    }

    fn sync_input_rows(&mut self, state: &AudioState, window: &mut Window, cx: &mut Context<Self>) {
        let Some(wakeup) = self.wakeup else {
            return;
        };
        let cmd_tx = self.cmd_tx.clone();

        for source in &state.sources {
            if source.is_monitor {
                continue;
            }
            if let Some(row) = self.rows.iter().find(|r| r.read(cx).index == source.index) {
                row.update(cx, |row, row_cx| {
                    row.update_from_source(source, row_cx);
                });
            } else {
                let row = cx.new(|row_cx| {
                    AudioDeviceRow::new_source(source, cmd_tx.clone(), wakeup, window, row_cx)
                });
                self.rows.push(row);
            }
        }
        self.rows.retain(|r| {
            state
                .sources
                .iter()
                .any(|s| s.index == r.read(cx).index && !s.is_monitor)
        });
    }
}

impl Render for AudioPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::ui::theme::theme(cx);
        let colors = &theme.colors;
        let text_styles = &theme.text_styles;

        let count = self.rows.len();
        let (title, caption) = match self.kind {
            DeviceKind::Output => (
                "Output Devices",
                match &self.subsystem_status {
                    SubsystemStatus::Connected => format!(
                        "{} output device{}",
                        count,
                        if count == 1 { "" } else { "s" }
                    ),
                    _ => String::new(),
                },
            ),
            DeviceKind::Input => (
                "Input Devices",
                match &self.subsystem_status {
                    SubsystemStatus::Connected => format!(
                        "{} input device{}",
                        count,
                        if count == 1 { "" } else { "s" }
                    ),
                    _ => String::new(),
                },
            ),
        };

        v_flex()
            .flex_1()
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .child(page_header(title, caption, colors, text_styles))
                    .with_fade_in_up("audio-header", 1),
            )
            .child(match &self.subsystem_status {
                SubsystemStatus::Connecting => h_flex()
                    .justify_center()
                    .flex_1()
                    .text_color(colors.text_secondary)
                    .child("Connecting to PulseAudio...")
                    .into_any_element(),
                SubsystemStatus::Disconnected(msg) => h_flex()
                    .justify_center()
                    .flex_1()
                    .text_color(colors.danger)
                    .child(format!("Error: {msg}"))
                    .into_any_element(),
                SubsystemStatus::Reconnecting => h_flex()
                    .justify_center()
                    .flex_1()
                    .text_color(colors.text_secondary)
                    .child("Reconnecting to PulseAudio...")
                    .into_any_element(),
                SubsystemStatus::Connected => {
                    if self.rows.is_empty() {
                        h_flex()
                            .justify_center()
                            .h(px(200.0))
                            .text_color(colors.text_secondary)
                            .child(match self.kind {
                                DeviceKind::Output => "No output devices found",
                                DeviceKind::Input => "No input devices found",
                            })
                            .into_any_element()
                    } else {
                        v_flex()
                            .gap_2()
                            .id("audio-device-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .children(self.rows.iter().enumerate().map(|(i, row)| {
                                div()
                                    .child(row.clone())
                                    .with_fade_in_up(format!("audio-row-{i}"), i + 2)
                                    .into_any_element()
                            }))
                            .into_any_element()
                    }
                }
            })
    }
}
