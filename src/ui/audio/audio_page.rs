//! Audio device page: shared view for output and input devices.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<AudioPage> → Vec<Entity<AudioDeviceRow>>
//!     → Entity<TextField> + Entity<Dropdown>

use crate::audio::pulse::PaWakeup;
use crate::audio::{AudioCommand, AudioState, DeviceKind};
use crate::ui::audio::device_row::AudioDeviceRow;
use crate::ui::{h_flex, v_flex};
use gpui::{Context, Entity, FontWeight, Render, SharedString, Window, div, hsla, prelude::*, px};

// ── Audio page entity ──────────────────────────────────────────────────────

/// Owns a list of `AudioDeviceRow` entities and coordinates their state.
pub(crate) struct AudioPage {
    kind: DeviceKind,
    rows: Vec<Entity<AudioDeviceRow>>,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    wakeup: Option<PaWakeup>,
    connected: bool,
    error: Option<String>,
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
            connected: false,
            error: None,
        }
    }

    /// Sync rows with the latest audio state. Creates/updates/removes rows
    /// to match the device list.
    pub(crate) fn sync_rows(
        &mut self,
        state: &AudioState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.connected = state.connected;
        self.error.clone_from(&state.error);

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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let surface = hsla(0.0, 0.0, 0.14, 1.0);
        let text_secondary = hsla(0.0, 0.0, 0.6, 1.0);

        let title = match self.kind {
            DeviceKind::Output => "Output Devices",
            DeviceKind::Input => "Input Devices",
        };

        v_flex()
            .flex_1()
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .bg(surface)
                    .border_b_1()
                    .border_color(hsla(0.0, 0.0, 0.25, 1.0))
                    .child(div().font_weight(FontWeight::BOLD).child(title)),
            )
            .when_some(self.error.clone(), |el, err| {
                el.child(
                    div()
                        .px_4()
                        .py_2()
                        .bg(gpui::rgba(0xff3c_3c33))
                        .text_color(hsla(0.0, 0.8, 0.75, 1.0))
                        .child(SharedString::from(format!("Error: {err}"))),
                )
            })
            .when(!self.connected, |el| {
                el.child(
                    h_flex()
                        .justify_center()
                        .flex_1()
                        .text_color(text_secondary)
                        .child("Connecting to PulseAudio..."),
                )
            })
            .when(self.connected && self.rows.is_empty(), |el| {
                el.child(
                    h_flex()
                        .justify_center()
                        .h(px(200.0))
                        .text_color(text_secondary)
                        .child(match self.kind {
                            DeviceKind::Output => "No output devices found",
                            DeviceKind::Input => "No input devices found",
                        }),
                )
            })
            .when(self.connected && !self.rows.is_empty(), |el| {
                el.child(
                    div()
                        .id("audio-device-list")
                        .flex_1()
                        .overflow_y_scroll()
                        .children(self.rows.clone()),
                )
            })
    }
}
