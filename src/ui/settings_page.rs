//! Settings page: application preferences.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<SettingsPage> → Entity<Dropdown> × 2

use gpui::{
    App, ClickEvent, Context, CursorStyle, Entity, EventEmitter, FocusHandle, Focusable, Render,
    SharedString, Subscription, Window, div, prelude::*,
};

use crate::ui::theme::ThemeMode;

use crate::settings::{settings, update_settings};
use crate::ui::animation::FadeInAnimationExt;
use crate::ui::components::dropdown::{Dropdown, DropdownEvent};
use crate::ui::components::page_header::page_header;
use crate::ui::ext::StyledExt;
use crate::ui::theme::{dark_theme_ids, light_theme_ids};
use crate::ui::{h_flex, v_flex};

// ── Events ───────────────────────────────────────────────────────────────────

/// Events emitted by the SettingsPage for the parent app to handle.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub(crate) enum SettingsEvent {
    #[allow(dead_code)]
    ModeChanged(ThemeMode),
    LightThemeChanged(String),
    DarkThemeChanged(String),
}

// ── Settings page entity ───────────────────────────────────────────────────

/// Application settings page entity.
pub(crate) struct SettingsPage {
    light_theme_dropdown: Entity<Dropdown>,
    dark_theme_dropdown: Entity<Dropdown>,
    focus_handle: FocusHandle,
    _light_dropdown_sub: Subscription,
    _dark_dropdown_sub: Subscription,
}

impl SettingsPage {
    /// Create a new settings page entity.
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let light_items = light_theme_ids()
            .iter()
            .map(|id| theme_display_name(id).to_string())
            .collect::<Vec<_>>();
        let dark_items = dark_theme_ids()
            .iter()
            .map(|id| theme_display_name(id).to_string())
            .collect::<Vec<_>>();

        let current = settings(cx);
        let light_idx = index_for_id(light_theme_ids(), &current.light_theme_id);
        let dark_idx = index_for_id(dark_theme_ids(), &current.dark_theme_id);

        let light_theme_dropdown = cx.new(|cx| Dropdown::new(light_items, light_idx, cx));
        let dark_theme_dropdown = cx.new(|cx| Dropdown::new(dark_items, dark_idx, cx));

        let _light_dropdown_sub = cx.subscribe(
            &light_theme_dropdown,
            |_this, _dropdown, event: &DropdownEvent, cx| {
                if let DropdownEvent::Selected(idx, _text) = event {
                    let ids = light_theme_ids();
                    if let Some(id) = ids.get(*idx) {
                        cx.emit(SettingsEvent::LightThemeChanged(id.to_string()));
                    }
                }
            },
        );

        let _dark_dropdown_sub = cx.subscribe(
            &dark_theme_dropdown,
            |_this, _dropdown, event: &DropdownEvent, cx| {
                if let DropdownEvent::Selected(idx, _text) = event {
                    let ids = dark_theme_ids();
                    if let Some(id) = ids.get(*idx) {
                        cx.emit(SettingsEvent::DarkThemeChanged(id.to_string()));
                    }
                }
            },
        );

        Self {
            light_theme_dropdown,
            dark_theme_dropdown,
            focus_handle: cx.focus_handle(),
            _light_dropdown_sub,
            _dark_dropdown_sub,
        }
    }
}

impl EventEmitter<SettingsEvent> for SettingsPage {}

impl Focusable for SettingsPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Read settings values up front so we can drop the immutable borrow
        // of `cx` before any mutable borrows (dropdown updates).
        let (theme_mode, light_theme_id, dark_theme_id) = {
            let s = settings(cx);
            (
                s.theme_mode,
                s.light_theme_id.clone(),
                s.dark_theme_id.clone(),
            )
        };

        let light_idx = index_for_id(light_theme_ids(), &light_theme_id);
        let dark_idx = index_for_id(dark_theme_ids(), &dark_theme_id);

        self.light_theme_dropdown.update(cx, |d, cx| {
            d.set_selected_index(light_idx, cx);
        });
        self.dark_theme_dropdown.update(cx, |d, cx| {
            d.set_selected_index(dark_idx, cx);
        });

        let theme = crate::ui::theme::theme(cx);
        let colors = &theme.colors;
        let text_styles = &theme.text_styles;
        let is_light = theme_mode == ThemeMode::Light;

        v_flex()
            .flex_1()
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .child(page_header(
                        "Settings",
                        "Configure application appearance and behavior.",
                        colors,
                        text_styles,
                    ))
                    .with_fade_in_up("settings-header", 1),
            )
            .child(
                v_flex()
                    .flex_1()
                    .id("settings-content")
                    .overflow_y_scroll()
                    .px_8()
                    .gap_4()
                    // ── Theme Mode ──
                    .child(
                        v_flex()
                            .gap_2()
                            .child(div().styled(text_styles.body2).child("Theme Mode"))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(mode_button(
                                        "Light",
                                        is_light,
                                        colors,
                                        text_styles,
                                        |_, _, cx| {
                                            update_settings(
                                                |s| s.theme_mode = ThemeMode::Light,
                                                cx,
                                            );
                                        },
                                    ))
                                    .child(mode_button(
                                        "Dark",
                                        !is_light,
                                        colors,
                                        text_styles,
                                        |_, _, cx| {
                                            update_settings(|s| s.theme_mode = ThemeMode::Dark, cx);
                                        },
                                    )),
                            ),
                    )
                    // ── Light Theme ──
                    .child(
                        v_flex()
                            .gap_2()
                            .child(div().styled(text_styles.body2).child("Light Theme"))
                            .child(self.light_theme_dropdown.clone()),
                    )
                    // ── Dark Theme ──
                    .child(
                        v_flex()
                            .gap_2()
                            .child(div().styled(text_styles.body2).child("Dark Theme"))
                            .child(self.dark_theme_dropdown.clone()),
                    )
                    .with_fade_in_up("settings-content", 2),
            )
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Build a theme mode toggle button.
fn mode_button(
    label: &'static str,
    active: bool,
    colors: &crate::ui::theme::ThemeColors,
    text_styles: &crate::ui::theme::TextStyleSet,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(SharedString::from(format!("mode-btn-{label}")))
        .px_3()
        .py_1()
        .rounded_sm()
        .styled(text_styles.caption)
        .bg(if active {
            colors.audio_accent
        } else {
            colors.element_background
        })
        .text_color(if active {
            colors.text_colored_button
        } else {
            colors.text
        })
        .cursor(CursorStyle::PointingHand)
        .hover(move |el| {
            el.bg(if active {
                colors.audio_accent
            } else {
                colors.element_hover
            })
        })
        .child(label)
        .on_click(on_click)
}

/// Human-friendly display name for a theme ID.
fn theme_display_name(id: &str) -> &'static str {
    match id {
        "rose-pine-dawn" => "Rose Pine Dawn",
        "rose-pine" => "Rose Pine",
        _ => "Unknown Theme",
    }
}

/// Find the index of a theme ID within a list of IDs.
fn index_for_id(ids: &[&'static str], target: &str) -> usize {
    ids.iter().position(|id| *id == target).unwrap_or(0)
}
