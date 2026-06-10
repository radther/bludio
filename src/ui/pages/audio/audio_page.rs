//! Audio device page: shared view for output and input devices.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<AudioPage> → Vec<Entity<AudioDeviceRow>>
//!     → Entity<TextField> + Entity<Dropdown>

use crate::backend::audio::pulse::PaWakeup;
use crate::backend::audio::{AudioCommand, AudioState, DeviceKind};
use crate::backend::subsystem::SubsystemStatus;
use crate::ui::common::animation::FadeInAnimationExt;
use crate::ui::common::icons;
use crate::ui::components::page_header::page_header;
use crate::ui::pages::audio::audio_device_row::{AudioDeviceRow, AudioDeviceRowEvent};
use crate::ui::theme::text_styles;
use crate::ui::{h_flex, v_flex};
use gpui::{
    ClickEvent, Context, CursorStyle, Entity, Render, Subscription, Window, div, prelude::*, px,
};
use std::collections::{HashMap, HashSet};

// ── Audio page entity ──────────────────────────────────────────────────────

/// Owns a list of `AudioDeviceRow` entities and coordinates their state.
pub(crate) struct AudioPage {
    kind: DeviceKind,
    rows: Vec<Entity<AudioDeviceRow>>,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    wakeup: Option<PaWakeup>,
    subsystem_status: SubsystemStatus,
    /// Whether the user is in selection mode for creating a combined sink.
    selection_mode: bool,
    /// Selected sink PA names while in selection mode.
    selected_sinks: Vec<String>,
    /// Subscriptions to row events (kept to avoid dropping them).
    _row_subs: HashMap<u32, Subscription>,
}

impl AudioPage {
    /// Generate a unique combined sink name that doesn't collide with existing sinks.
    fn generate_combine_name(&self, existing_names: &HashSet<String>) -> String {
        (1..)
            .map(|n| format!("Bludio-combined-{}", n))
            .find(|name| !existing_names.contains(name))
            .expect("infinite sequence guarantees a free name")
    }

    /// Toggle combined-sink selection mode, creating the sink on exit if any are selected.
    fn toggle_combine_mode(&mut self, cx: &mut Context<Self>) {
        if self.selection_mode {
            if !self.selected_sinks.is_empty() {
                let slaves = self.selected_sinks.clone();
                let existing_names: HashSet<String> = self
                    .rows
                    .iter()
                    .map(|r| r.read(cx).pa_name.clone())
                    .collect();
                let name = self.generate_combine_name(&existing_names);
                let _ = self
                    .cmd_tx
                    .send(AudioCommand::LoadCombineSink(name, slaves));
                if let Some(w) = self.wakeup {
                    w.wake();
                }
            }
            self.selection_mode = false;
            self.selected_sinks.clear();
        } else {
            self.selection_mode = true;
        }
        let new_mode = self.selection_mode;
        for row in &self.rows {
            row.update(cx, |row, cx| row.set_selection_mode(new_mode, cx));
        }
        cx.notify();
    }

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
            selection_mode: false,
            selected_sinks: Vec::new(),
            _row_subs: HashMap::new(),
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
        let selection_mode = self.selection_mode;
        let selected = &self.selected_sinks;

        for sink in &state.sinks {
            let is_selected = selected.contains(&sink.name);
            if let Some(row) = self.rows.iter().find(|r| r.read(cx).index == sink.index) {
                row.update(cx, |row, row_cx| {
                    row.update_from_sink(sink, selection_mode, is_selected, row_cx);
                });
            } else {
                let row = cx.new(|row_cx| {
                    AudioDeviceRow::new_sink(
                        sink,
                        selection_mode,
                        is_selected,
                        cmd_tx.clone(),
                        wakeup,
                        window,
                        row_cx,
                    )
                });
                let sub = cx.subscribe(&row, |this, _row, event: &AudioDeviceRowEvent, cx| {
                    let AudioDeviceRowEvent::ToggleSelection(pa_name) = event;
                    if let Some(pos) = this.selected_sinks.iter().position(|n| n == pa_name) {
                        this.selected_sinks.remove(pos);
                    } else {
                        this.selected_sinks.push(pa_name.clone());
                    }
                    cx.notify();
                });
                self._row_subs.insert(sink.index, sub);
                self.rows.push(row);
            }
        }
        self.rows
            .retain(|r| state.sinks.iter().any(|s| s.index == r.read(cx).index));
        let retained: HashSet<u32> = self.rows.iter().map(|r| r.read(cx).index).collect();
        self._row_subs.retain(|idx, _| retained.contains(idx));
        let active_names: HashSet<String> = state.sinks.iter().map(|s| s.name.clone()).collect();
        self.selected_sinks
            .retain(|name| active_names.contains(name));
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
        let text_styles = text_styles(cx);

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

        let is_output = self.kind == DeviceKind::Output;
        let selection_mode = self.selection_mode;

        v_flex()
            .flex_1()
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .child(page_header(title, caption, colors, &text_styles))
                    .when(
                        is_output && self.subsystem_status == SubsystemStatus::Connected,
                        |el| {
                            let (btn_bg, btn_hover, icon_color) = if selection_mode {
                                (
                                    colors.audio_accent,
                                    colors.audio_accent,
                                    colors.text_colored_button,
                                )
                            } else {
                                (
                                    colors.element_background,
                                    colors.element_hover,
                                    colors.text_secondary,
                                )
                            };
                            el.child(
                                h_flex()
                                    .id("combine-btn")
                                    .justify_center()
                                    .w(px(48.0))
                                    .h(px(48.0))
                                    .rounded_lg()
                                    .bg(btn_bg)
                                    .cursor(CursorStyle::PointingHand)
                                    .hover(move |el| el.bg(btn_hover))
                                    .child(
                                        icons::merge()
                                            .w(px(24.0))
                                            .h(px(24.0))
                                            .text_color(icon_color),
                                    )
                                    .on_click({
                                        let entity = cx.entity().clone();
                                        move |_: &ClickEvent, _window, cx| {
                                            entity.update(cx, |this, cx| {
                                                this.toggle_combine_mode(cx)
                                            });
                                        }
                                    }),
                            )
                        },
                    )
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
