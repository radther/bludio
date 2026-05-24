//! Audio device row entity: one entry in the device list.
//!
//! Owns its own `TextField` for volume editing and its own `Dropdown` for
//! profile selection. Renders the volume bar, text field, mute button,
//! default button, and profile dropdown inline.

use crate::audio::pulse::PaWakeup;
use crate::audio::{AudioCommand, DeviceKind};
use crate::ui::components::dropdown::DropdownEvent as DdEvt;
use crate::ui::components::slider::{Slider, SliderEvent};
use crate::ui::components::text_field::{TextField, TextFieldEvent};
use crate::ui::{h_flex, v_flex};
use gpui::{
    App, Context, CursorStyle, Div, Entity, FocusHandle, Focusable, FontWeight, MouseButton,
    MouseUpEvent, Render, SharedString, Stateful, Subscription, Window, div, hsla, prelude::*, px,
};

const ROW_HEIGHT: f32 = 64.0;
const LABEL_WIDTH: f32 = 44.0;
const BAR_HEIGHT: f32 = 20.0;

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
    slider: Entity<Slider>,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
    wakeup: PaWakeup,
    focus_handle: FocusHandle,
    #[allow(dead_code)]
    slider_sub: Subscription,
    #[allow(dead_code)]
    text_field_sub: Subscription,
    #[allow(dead_code)]
    dropdown_sub: Subscription,
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
                .accent(hsla(210.0 / 360.0, 0.7, 0.55, 1.0))
                .bg(hsla(0.0, 0.0, 0.22, 1.0))
                .hover_bg(hsla(0.0, 0.0, 0.3, 1.0))
                .menu_bg(hsla(0.0, 0.0, 0.16, 1.0))
                .menu_border(hsla(0.0, 0.0, 0.3, 1.0))
        });

        // ── Slider ──
        let slider = cx.new(|cx| {
            let mut s = Slider::new(cx);
            s.set_value(params.volume, cx);
            s
        });

        // ── Subscriptions ──
        let slider_sub = cx.subscribe_in(&slider, window, {
            let cmd_tx = cmd_tx.clone();
            move |this, _sl, event: &SliderEvent, _window, _cx| {
                let kind = this.kind;
                let idx = this.index;
                match event {
                    SliderEvent::Change(v) => {
                        let _ = match kind {
                            DeviceKind::Output => cmd_tx.send(AudioCommand::SetSinkVolume(idx, *v)),
                            DeviceKind::Input => {
                                cmd_tx.send(AudioCommand::SetSourceVolume(idx, *v))
                            }
                        };
                        this.wakeup.wake();
                    }
                    SliderEvent::Release(_) => {}
                }
            }
        });
        let text_field_sub = cx.subscribe_in(&text_field, window, {
            move |this, _tf, event: &TextFieldEvent, window, cx| match event {
                TextFieldEvent::Confirmed(text) => {
                    if let Ok(val) = text.parse::<f64>() {
                        let clamped = val.clamp(0.0, 100.0) / 100.0;
                        let cmd = match this.kind {
                            DeviceKind::Output => AudioCommand::SetSinkVolume(this.index, clamped),
                            DeviceKind::Input => AudioCommand::SetSourceVolume(this.index, clamped),
                        };
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
        let dropdown_sub = cx.subscribe_in(&profile_dropdown, window, {
            let cmd_tx = cmd_tx.clone();
            move |this, _dd, event: &DdEvt, _window, _cx| {
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
            slider_sub,
            text_field_sub,
            dropdown_sub,
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
        let accent = hsla(210.0 / 360.0, 0.7, 0.55, 1.0);
        let accent_hover = hsla(210.0 / 360.0, 0.7, 0.45, 1.0);

        // ── Blur-sync: reset text field to actual volume when unfocused ──
        let tf_focused = self.text_field.read(cx).focus_handle(cx).is_focused(window);
        if !tf_focused {
            let new_vol = format!("{}", (self.volume * 100.0).round());
            let current = self.text_field.read(cx).text().to_string();
            if current != new_vol {
                self.text_field.update(cx, |f, cx| f.set_text(&new_vol, cx));
            }
        }

        h_flex()
            .justify_between()
            .id(SharedString::from(format!("adevice-{}", self.index)))
            .px_4()
            .h(px(ROW_HEIGHT))
            .border_b_1()
            .border_color(hsla(0.0, 0.0, 0.20, 1.0))
            .hover(|el| el.bg(hsla(0.0, 0.0, 1.0, 0.04)))
            .child(
                v_flex()
                    .w_full()
                    .child(self.render_device_name(accent))
                    .child(self.render_controls_row(accent, accent_hover, cx)),
            )
    }
}

// ── Render sub-views ──────────────────────────────────────────────────────

impl AudioDeviceRow {
    /// Device name row with optional "● Default" badge.
    fn render_device_name(&self, accent: gpui::Hsla) -> Div {
        h_flex()
            .gap_2()
            .child(
                div()
                    .font_weight(FontWeight::MEDIUM)
                    .child(SharedString::from(self.display_name.clone())),
            )
            .when(self.is_default, |el| {
                el.child(div().text_xs().text_color(accent).child("● Default"))
            })
    }

    /// Volume bar + text field + mute + default button + profile selector.
    fn render_controls_row(
        &self,
        accent: gpui::Hsla,
        accent_hover: gpui::Hsla,
        cx: &mut Context<Self>,
    ) -> Div {
        let idx = self.index;
        let kind = self.kind;
        let cmd_tx = self.cmd_tx.clone();
        let wk = self.wakeup;

        let vol_color = if self.muted {
            hsla(0.0, 0.0, 0.4, 1.0)
        } else {
            hsla(210.0 / 360.0, 0.7, 0.55, 1.0)
        };

        // Note: `muted` and `kind` are captured by value at render time.
        // A fresh closure is created each frame via cx.notify(), so the
        // captured values always match the current row state.
        h_flex()
            .w_full()
            .gap_2()
            // ── Volume slider ──
            .child(self.slider.clone())
            // ── Inline text field for numeric volume ──
            .child(
                h_flex()
                    .w(px(LABEL_WIDTH))
                    .h(px(BAR_HEIGHT))
                    .bg(hsla(0.0, 0.0, 0.18, 1.0))
                    .rounded_sm()
                    .text_color(vol_color)
                    .child(self.text_field.clone()),
            )
            // ── Mute / Unmute button ──
            .child(mute_button(
                SharedString::from(format!("mute-btn-{idx}")),
                self.muted,
                {
                    let muted = self.muted;
                    let cmd = cmd_tx.clone();
                    move || {
                        let _ = match kind {
                            DeviceKind::Output => cmd.send(AudioCommand::SetSinkMute(idx, !muted)),
                            DeviceKind::Input => cmd.send(AudioCommand::SetSourceMute(idx, !muted)),
                        };
                        wk.wake();
                    }
                },
            ))
            // ── Default button ──
            .child(self.render_default_button(accent, accent_hover))
            // ── Profile dropdown (output devices with profiles only) ──
            .when(
                self.kind == DeviceKind::Output
                    && self.card_index.is_some()
                    && self.profile_dropdown.read(cx).has_items(),
                |el| el.child(self.profile_dropdown.clone()),
            )
    }

    /// "Default" action button.
    fn render_default_button(&self, accent: gpui::Hsla, accent_hover: gpui::Hsla) -> Stateful<Div> {
        let cmd_tx = self.cmd_tx.clone();
        let pa_name = self.pa_name.clone();
        let kind = self.kind;
        let wk = self.wakeup;

        action_btn("Default", accent, accent_hover, move || {
            let _ = match kind {
                DeviceKind::Output => cmd_tx.send(AudioCommand::SetDefaultSink(pa_name.clone())),
                DeviceKind::Input => cmd_tx.send(AudioCommand::SetDefaultSource(pa_name.clone())),
            };
            wk.wake();
        })
    }
}

// ── Mute button ────────────────────────────────────────────────────────────

fn mute_button(id: SharedString, muted: bool, on_toggle: impl Fn() + 'static) -> Stateful<Div> {
    let lbl = if muted { "Unmute" } else { "Mute" };
    let bg = if muted {
        hsla(0.0, 0.7, 0.55, 1.0)
    } else {
        hsla(0.0, 0.0, 0.22, 1.0)
    };
    let hbg = if muted {
        hsla(0.0, 0.7, 0.45, 1.0)
    } else {
        hsla(0.0, 0.0, 0.3, 1.0)
    };
    div()
        .id(id)
        .px_2()
        .py_1()
        .rounded_sm()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .bg(bg)
        .cursor(CursorStyle::PointingHand)
        .hover(move |el| el.bg(hbg))
        .child(SharedString::from(lbl))
        .on_mouse_up(MouseButton::Left, move |_: &MouseUpEvent, _, _| on_toggle())
}

// ── Action button ──────────────────────────────────────────────────────────

fn action_btn(
    label: &str,
    bg: gpui::Hsla,
    hover_bg: gpui::Hsla,
    on_click: impl Fn() + 'static,
) -> Stateful<Div> {
    div()
        .id(SharedString::from(format!("btn-{label}")))
        .px_2()
        .py_1()
        .rounded_sm()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .bg(bg)
        .cursor(CursorStyle::PointingHand)
        .hover(move |el| el.bg(hover_bg))
        .child(SharedString::from(label.to_string()))
        .on_mouse_up(MouseButton::Left, move |_: &MouseUpEvent, _window, _cx| {
            on_click();
        })
}
