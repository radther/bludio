//! Audio device row entity: one entry in the device list.
//!
//! Owns its own `TextField` for volume editing and its own `Dropdown` for
//! profile selection. Renders the volume bar, text field, mute button,
//! default button, and profile dropdown inline.

use crate::audio::pulse::PaWakeup;
use crate::audio::{AudioCommand, DeviceKind};
use crate::ui::components::button::action_btn;
use crate::ui::components::dropdown::DropdownEvent as DdEvt;
use crate::ui::components::slider::{Slider, SliderEvent, SliderState};
use crate::ui::components::status_strip::status_strip;
use crate::ui::components::text_field::{TextField, TextFieldEvent};
use crate::ui::{StyledExt, h_flex, v_flex};
use gpui::{
    App, Context, Div, Entity, FocusHandle, Focusable, Render, SharedString, Stateful,
    Subscription, Window, div, prelude::*, px,
};

const LABEL_WIDTH: f32 = 48.0;

// ── Audio device row entity ────────────────────────────────────────────────

/// One row in the audio device list. Owns its own `TextField` for volume
/// editing and its own `Dropdown` for profile selection.
pub(crate) struct AudioDeviceRow {
    pub(crate) index: u32,
    card_index: Option<u32>,
    display_name: String,
    pa_name: String,
    volume: f64,
    muted: bool,
    is_default: bool,
    kind: DeviceKind,
    text_field: Entity<TextField>,
    profile_dropdown: Entity<crate::ui::components::dropdown::Dropdown>,
    slider: Entity<SliderState>,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    wakeup: PaWakeup,
    focus_handle: FocusHandle,
    _slider_sub: Subscription,
    _text_field_sub: Subscription,
    _dropdown_sub: Subscription,
}

/// Bundled parameters for the shared `new_impl` constructor.
struct RowParams {
    kind: DeviceKind,
    index: u32,
    display_name: String,
    pa_name: String,
    volume: f64,
    muted: bool,
    is_default: bool,
    card_index: Option<u32>,
    profile_data: Option<(Vec<String>, usize)>,
}

impl AudioDeviceRow {
    /// Create a new device row entity for a sink (output).
    pub(crate) fn new_sink(
        sink: &crate::audio::SinkInfo,
        cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
        wakeup: PaWakeup,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let profiles: Vec<String> = sink
            .available_profiles
            .iter()
            .map(|p| p.name.clone())
            .collect();
        let selected_idx = sink
            .active_profile
            .as_ref()
            .and_then(|active| profiles.iter().position(|p| p == active))
            .unwrap_or(0);
        Self::new_impl(
            RowParams {
                kind: DeviceKind::Output,
                index: sink.index,
                display_name: sink.description.clone(),
                pa_name: sink.name.clone(),
                volume: sink.volume,
                muted: sink.muted,
                is_default: sink.is_default,
                card_index: sink.card_index,
                profile_data: Some((profiles, selected_idx)),
            },
            cmd_tx,
            wakeup,
            window,
            cx,
        )
    }

    /// Create a new device row entity for a source (input).
    pub(crate) fn new_source(
        source: &crate::audio::SourceInfo,
        cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
        wakeup: PaWakeup,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_impl(
            RowParams {
                kind: DeviceKind::Input,
                index: source.index,
                display_name: source.description.clone(),
                pa_name: source.name.clone(),
                volume: source.volume,
                muted: source.muted,
                is_default: source.is_default,
                card_index: None,
                profile_data: None,
            },
            cmd_tx,
            wakeup,
            window,
            cx,
        )
    }

    /// Shared constructor — called by both `new_sink` and `new_source`.
    fn new_impl(
        params: RowParams,
        cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
        wakeup: PaWakeup,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let text_field = cx.new(|cx| {
            TextField::new(cx)
                .filter_char(|c| c.is_ascii_digit())
                .placeholder("vol")
                .align(gpui::TextAlign::Center)
        });
        text_field.update(cx, |f, cx| {
            f.set_text(&format!("{}", (params.volume * 100.0).round()), cx);
        });

        let profile_dropdown = cx.new(|cx| {
            let (profiles, selected) = params.profile_data.unwrap_or_default();
            crate::ui::components::dropdown::Dropdown::new(profiles, selected, cx)
                .placeholder("unknown")
        });

        // ── Slider ──
        let slider = cx.new(|cx| {
            let mut s = SliderState::new(cx);
            s.set_value(params.volume, cx);
            s
        });

        // ── Subscriptions ──
        let _slider_sub = cx.subscribe(&slider, {
            let cmd_tx = cmd_tx.clone();
            move |this, _sl, event: &SliderEvent, _cx| {
                let kind = this.kind;
                let idx = this.index;
                match event {
                    SliderEvent::Change(v) => {
                        let _ = cmd_tx.send(AudioCommand::SetVolume(kind, idx, *v));
                        this.wakeup.wake();
                    }
                    SliderEvent::Release(_) => {}
                }
            }
        });
        let _text_field_sub = cx.subscribe_in(&text_field, window, {
            move |this, _tf, event: &TextFieldEvent, window, cx| match event {
                TextFieldEvent::Confirmed(text) => {
                    if let Ok(val) = text.parse::<f64>() {
                        let clamped = val.clamp(0.0, 100.0) / 100.0;
                        let cmd = AudioCommand::SetVolume(this.kind, this.index, clamped);
                        let _ = this.cmd_tx.send(cmd);
                        this.wakeup.wake();
                        this.text_field.update(cx, |f, cx| {
                            f.set_text(&format!("{}", (clamped * 100.0).round()), cx);
                        });
                        cx.notify();
                    } else {
                        // Empty / invalid — revert to actual volume.
                        let vol_text = format!("{}", (this.volume * 100.0).round());
                        this.text_field
                            .update(cx, |f, cx| f.set_text(&vol_text, cx));
                    }
                    window.focus(&this.focus_handle, cx);
                }
                TextFieldEvent::Cancelled => {
                    // Revert to actual volume.
                    let vol_text = format!("{}", (this.volume * 100.0).round());
                    this.text_field
                        .update(cx, |f, cx| f.set_text(&vol_text, cx));
                    cx.notify();
                    window.focus(&this.focus_handle, cx);
                }
            }
        });
        let _dropdown_sub = cx.subscribe(&profile_dropdown, {
            let cmd_tx = cmd_tx.clone();
            move |this, _dd, event: &DdEvt, _cx| {
                if let DdEvt::Selected(_idx, profile) = event
                    && let Some(card_idx) = this.card_index
                {
                    let _ = cmd_tx.send(AudioCommand::SetCardProfile(card_idx, profile.clone()));
                    this.wakeup.wake();
                }
            }
        });

        Self {
            index: params.index,
            card_index: params.card_index,
            display_name: params.display_name,
            pa_name: params.pa_name,
            volume: params.volume,
            muted: params.muted,
            is_default: params.is_default,
            kind: params.kind,
            text_field,
            profile_dropdown,
            slider,
            cmd_tx,
            wakeup,
            focus_handle: cx.focus_handle(),
            _slider_sub,
            _text_field_sub,
            _dropdown_sub,
        }
    }

    /// Sync row state from a fresh sink snapshot.
    pub(crate) fn update_from_sink(
        &mut self,
        sink: &crate::audio::SinkInfo,
        cx: &mut Context<Self>,
    ) {
        self.card_index = sink.card_index;
        self.display_name.clone_from(&sink.description);
        self.pa_name.clone_from(&sink.name);
        self.volume = sink.volume;
        self.muted = sink.muted;
        self.is_default = sink.is_default;

        let profiles: Vec<String> = sink
            .available_profiles
            .iter()
            .map(|p| p.name.clone())
            .collect();
        let selected_idx = sink
            .active_profile
            .as_ref()
            .and_then(|active| profiles.iter().position(|p| p == active))
            .unwrap_or(0);
        self.profile_dropdown
            .update(cx, |d, cx| d.set_items(&profiles, selected_idx, cx));
        self.slider.update(cx, |s, cx| s.set_value(sink.volume, cx));
        cx.notify();
    }

    /// Sync row state from a fresh source snapshot.
    pub(crate) fn update_from_source(
        &mut self,
        source: &crate::audio::SourceInfo,
        cx: &mut Context<Self>,
    ) {
        self.display_name.clone_from(&source.description);
        self.pa_name.clone_from(&source.name);
        self.volume = source.volume;
        self.muted = source.muted;
        self.is_default = source.is_default;
        self.slider
            .update(cx, |s, cx| s.set_value(source.volume, cx));
        cx.notify();
    }
}

impl Focusable for AudioDeviceRow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AudioDeviceRow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // ── Blur-sync: reset text field to actual volume when unfocused ──
        let tf_focused = self.text_field.read(cx).focus_handle(cx).is_focused(window);
        if !tf_focused {
            let new_vol = format!("{}", (self.volume * 100.0).round());
            let current = self.text_field.read(cx).text().to_string();
            if current != new_vol {
                self.text_field.update(cx, |f, cx| f.set_text(&new_vol, cx));
            }
        }

        let (hover_overlay, status_color) = {
            let colors = &crate::ui::theme::theme(cx).colors;
            let status = if self.is_default {
                colors.audio_accent
            } else {
                colors.text_secondary
            };
            (colors.hover_overlay, status)
        };

        h_flex()
            .items_stretch()
            .id(SharedString::from(format!("adevice-{}", self.index)))
            .hover(|el| el.bg(hover_overlay))
            .child(status_strip(status_color))
            .child(
                v_flex()
                    .gap_3()
                    .pr_4()
                    .pl_4()
                    .flex_1()
                    .child(self.render_title_row(cx))
                    .child(self.render_volume_row(cx)),
            )
    }
}

// ── Render sub-views ──────────────────────────────────────────────────────

impl AudioDeviceRow {
    /// Title row: device name on the left, action buttons on the right.
    fn render_title_row(&self, cx: &mut Context<Self>) -> Div {
        let colors = &crate::ui::theme::theme(cx).colors;
        let text_styles = &crate::ui::theme::theme(cx).text_styles;

        let (mute_label, mute_bg, mute_hover, mute_text) = if self.muted {
            (
                "Unmute",
                colors.danger,
                colors.danger,
                colors.text_colored_button,
            )
        } else {
            (
                "Mute",
                colors.element_background,
                colors.element_hover,
                colors.text,
            )
        };

        h_flex()
            .justify_between()
            .child(self.render_device_name(cx))
            .child(
                h_flex()
                    .gap_2()
                    .child(action_btn(
                        format!("mute-btn-{}", self.index),
                        mute_label,
                        mute_bg,
                        mute_hover,
                        mute_text,
                        {
                            let cmd_tx = self.cmd_tx.clone();
                            let kind = self.kind;
                            let idx = self.index;
                            let muted = self.muted;
                            let wk = self.wakeup;
                            move || {
                                let _ = cmd_tx.send(AudioCommand::SetMute(kind, idx, !muted));
                                wk.wake();
                            }
                        },
                        text_styles,
                    ))
                    .child(self.render_default_button(cx))
                    .when(
                        self.kind == DeviceKind::Output
                            && self.card_index.is_some()
                            && self.profile_dropdown.read(cx).has_items(),
                        |el| el.child(self.profile_dropdown.clone()),
                    ),
            )
    }

    /// Device name with optional "● Default" badge.
    fn render_device_name(&self, cx: &App) -> Div {
        let colors = &crate::ui::theme::theme(cx).colors;
        let text_styles = &crate::ui::theme::theme(cx).text_styles;
        h_flex()
            .gap_2()
            .child(
                div()
                    .styled(text_styles.body)
                    .child(SharedString::from(self.display_name.clone())),
            )
            .when(self.is_default, |el| {
                el.child(
                    div()
                        .styled(text_styles.caption)
                        .text_color(colors.audio_accent)
                        .child("Default"),
                )
            })
    }

    /// Volume row: slider spanning full width, with text field at the right end.
    fn render_volume_row(&self, cx: &mut Context<Self>) -> Div {
        let colors = &crate::ui::theme::theme(cx).colors;
        let text_styles = &crate::ui::theme::theme(cx).text_styles;
        let vol_color = if self.muted {
            colors.muted
        } else {
            colors.audio_accent
        };

        h_flex()
            .w_full()
            .gap_4()
            // ── Volume slider (flex_1, takes all available space) ──
            .child(Slider::new(&self.slider).fill_color(vol_color))
            // ── Inline text field for numeric volume ──
            .child(
                h_flex()
                    .w(px(LABEL_WIDTH))
                    .bg(colors.element_background)
                    .rounded_sm()
                    // .p_2()
                    .text_color(colors.text)
                    .styled(text_styles.caption)
                    .child(self.text_field.clone()),
            )
    }

    /// "Default" action button.
    fn render_default_button(&self, cx: &App) -> Stateful<Div> {
        let cmd_tx = self.cmd_tx.clone();
        let pa_name = self.pa_name.clone();
        let kind = self.kind;
        let wk = self.wakeup;
        let colors = &crate::ui::theme::theme(cx).colors;
        let text_styles = &crate::ui::theme::theme(cx).text_styles;

        action_btn(
            format!("btn-default-{}", self.index),
            "Default",
            colors.audio_accent,
            colors.audio_accent,
            colors.text_colored_button,
            move || {
                let _ = match kind {
                    DeviceKind::Output => {
                        cmd_tx.send(AudioCommand::SetDefaultSink(pa_name.clone()))
                    }
                    DeviceKind::Input => {
                        cmd_tx.send(AudioCommand::SetDefaultSource(pa_name.clone()))
                    }
                };
                wk.wake();
            },
            text_styles,
        )
    }
}
