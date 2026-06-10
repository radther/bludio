//! Settings page: application preferences.
//!
//! Ownership chain:
//!   `BludioApp` → Entity<SettingsPage> → Entity<Dropdown> × 3

use gpui::{
    App, Context, CursorStyle, Entity, EventEmitter, FocusHandle, Focusable, Render, Subscription,
    Window, div, prelude::*,
};

use crate::backend::settings::settings;
use crate::ui::StyledExt;
use crate::ui::common::animation::FadeInAnimationExt;
use crate::ui::components::dropdown::{Dropdown, DropdownEvent};
use crate::ui::components::page_header::page_header;
use crate::ui::components::toggle::toggle_switch;
use crate::ui::theme::{TextStyleSet, ThemeColors, ThemeMode, dark_theme_ids, light_theme_ids};
use crate::ui::{h_flex, v_flex};

// ── Events ───────────────────────────────────────────────────────────────────

/// Events emitted by the SettingsPage for the parent app to handle.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub(crate) enum SettingsEvent {
    ModeChanged(ThemeMode),
    LightThemeChanged(String),
    DarkThemeChanged(String),
    DisableAnimationsChanged(bool),
    FontChanged(String),
}

// ── Settings page entity ───────────────────────────────────────────────────

/// Application settings page entity.
pub(crate) struct SettingsPage {
    light_theme_dropdown: Entity<Dropdown>,
    dark_theme_dropdown: Entity<Dropdown>,
    font_dropdown: Entity<Dropdown>,
    focus_handle: FocusHandle,
    _light_dropdown_sub: Subscription,
    _dark_dropdown_sub: Subscription,
    _font_dropdown_sub: Subscription,
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
        let font_items = vec!["Noto Sans".to_string(), "OpenDyslexic".to_string()];

        let current = settings(cx);
        let light_idx = index_for_id(light_theme_ids(), &current.light_theme_id);
        let dark_idx = index_for_id(dark_theme_ids(), &current.dark_theme_id);
        let font_idx = index_for_font(&current.font_family);

        let light_theme_dropdown = cx.new(|cx| Dropdown::new(light_items, light_idx, cx));
        let dark_theme_dropdown = cx.new(|cx| Dropdown::new(dark_items, dark_idx, cx));
        let font_dropdown = cx.new(|cx| Dropdown::new(font_items, font_idx, cx));

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

        let _font_dropdown_sub = cx.subscribe(
            &font_dropdown,
            |_this, _dropdown, event: &DropdownEvent, cx| {
                if let DropdownEvent::Selected(idx, _text) = event {
                    let name = font_name_for_index(*idx);
                    cx.emit(SettingsEvent::FontChanged(name.to_string()));
                }
            },
        );

        Self {
            light_theme_dropdown,
            dark_theme_dropdown,
            font_dropdown,
            focus_handle: cx.focus_handle(),
            _light_dropdown_sub,
            _dark_dropdown_sub,
            _font_dropdown_sub,
        }
    }

    /// Sync dropdown selected indices with the current global settings.
    /// Call this after an external settings change (e.g. from another page
    /// via `update_settings`) to keep the dropdowns consistent.
    pub(crate) fn sync_dropdowns(&mut self, cx: &mut Context<Self>) {
        let current = settings(cx);
        let light_idx = index_for_id(light_theme_ids(), &current.light_theme_id);
        let dark_idx = index_for_id(dark_theme_ids(), &current.dark_theme_id);
        let font_idx = index_for_font(&current.font_family);

        self.light_theme_dropdown.update(cx, |d, cx| {
            d.set_selected_index(light_idx, cx);
        });
        self.dark_theme_dropdown.update(cx, |d, cx| {
            d.set_selected_index(dark_idx, cx);
        });
        self.font_dropdown.update(cx, |d, cx| {
            d.set_selected_index(font_idx, cx);
        });
    }
}

impl EventEmitter<SettingsEvent> for SettingsPage {}

impl Focusable for SettingsPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::ui::theme::theme(cx);
        let colors = &theme.colors;
        let text_styles = &theme.text_styles;

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
                    .gap_6()
                    .child(self.render_appearance_section(window, cx))
                    .child(self.render_accessibility_section(window, cx))
                    .with_fade_in_up("settings-content", 2),
            )
    }
}

// ── Render sub-views ──────────────────────────────────────────────────────

impl SettingsPage {
    /// Appearance section: Font, Theme Mode, Light Theme, Dark Theme.
    fn render_appearance_section(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        let theme = crate::ui::theme::theme(cx);
        let colors = &theme.colors;
        let text_styles = &theme.text_styles;
        let is_light = settings(cx).theme_mode == ThemeMode::Light;
        let entity = cx.entity().clone();

        v_flex()
            .gap_2()
            .child(
                div()
                    .styled(text_styles.heading2)
                    .child("Appearance")
                    .border_b_1()
                    .border_color(colors.border),
            )
            .child(
                v_flex()
                    .gap_4()
                    .child(self.render_font_row(text_styles))
                    .child(self.render_theme_mode_row(
                        is_light,
                        colors,
                        text_styles,
                        window,
                        &entity,
                    ))
                    .child(self.render_light_theme_row(text_styles))
                    .child(self.render_dark_theme_row(text_styles)),
            )
    }

    /// Accessibility section: Disable Animations toggle.
    fn render_accessibility_section(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        let theme = crate::ui::theme::theme(cx);
        let colors = &theme.colors;
        let text_styles = &theme.text_styles;
        let disable_animations = settings(cx).disable_animations;
        let entity = cx.entity().clone();

        v_flex()
            .gap_2()
            .child(
                div()
                    .styled(text_styles.heading2)
                    .child("Accessibility")
                    .border_b_1()
                    .border_color(colors.border),
            )
            .child(self.render_disable_animations_row(
                disable_animations,
                colors,
                text_styles,
                window,
                &entity,
            ))
    }

    // ── Row builders ───────────────────────────────────────────────────────

    /// Font dropdown row.
    fn render_font_row(&self, text_styles: &TextStyleSet) -> gpui::Div {
        setting_row("Font", text_styles, self.font_dropdown.clone())
    }

    /// Theme mode toggle row.
    fn render_theme_mode_row(
        &self,
        is_light: bool,
        colors: &ThemeColors,
        text_styles: &TextStyleSet,
        window: &mut Window,
        entity: &Entity<SettingsPage>,
    ) -> gpui::Div {
        setting_row(
            "Theme Mode",
            text_styles,
            h_flex()
                .gap_2()
                .child(mode_button(
                    "Light",
                    is_light,
                    colors,
                    text_styles,
                    window,
                    entity,
                    ThemeMode::Light,
                ))
                .child(mode_button(
                    "Dark",
                    !is_light,
                    colors,
                    text_styles,
                    window,
                    entity,
                    ThemeMode::Dark,
                )),
        )
    }

    /// Light theme dropdown row.
    fn render_light_theme_row(&self, text_styles: &TextStyleSet) -> gpui::Div {
        setting_row(
            "Light Theme",
            text_styles,
            self.light_theme_dropdown.clone(),
        )
    }

    /// Dark theme dropdown row.
    fn render_dark_theme_row(&self, text_styles: &TextStyleSet) -> gpui::Div {
        setting_row("Dark Theme", text_styles, self.dark_theme_dropdown.clone())
    }

    /// Disable Animations toggle row — label left, switch right, whole row clickable.
    fn render_disable_animations_row(
        &self,
        disable_animations: bool,
        colors: &ThemeColors,
        text_styles: &TextStyleSet,
        window: &mut Window,
        entity: &Entity<SettingsPage>,
    ) -> gpui::Stateful<gpui::Div> {
        let new_value = !disable_animations;

        h_flex()
            .id("disable-animations-row")
            .w_full()
            .justify_between()
            .items_center()
            .cursor(CursorStyle::PointingHand)
            .child(div().styled(text_styles.body).child("Disable Animations"))
            .child(toggle_switch(
                "disable-animations-switch",
                disable_animations,
                colors,
            ))
            .on_click(
                window.listener_for(entity, move |_this: &mut SettingsPage, _, _, cx| {
                    cx.emit(SettingsEvent::DisableAnimationsChanged(new_value));
                }),
            )
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// A labelled setting row: label above, control below.
fn setting_row(
    label: &'static str,
    text_styles: &TextStyleSet,
    control: impl IntoElement,
) -> gpui::Div {
    v_flex()
        .child(div().styled(text_styles.body).child(label))
        .child(control)
}

/// Build a theme mode toggle button.
fn mode_button(
    label: &'static str,
    active: bool,
    colors: &ThemeColors,
    text_styles: &TextStyleSet,
    window: &mut Window,
    entity: &Entity<SettingsPage>,
    mode: ThemeMode,
) -> gpui::Stateful<gpui::Div> {
    let id = match label {
        "Light" => "mode-btn-light",
        "Dark" => "mode-btn-dark",
        _ => "mode-btn-unknown",
    };

    div()
        .id(id)
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
        .on_click(
            window.listener_for(entity, move |_this: &mut SettingsPage, _, _, cx| {
                cx.emit(SettingsEvent::ModeChanged(mode));
            }),
        )
}

/// Human-friendly display name for a theme ID.
fn theme_display_name(id: &str) -> &'static str {
    match id {
        "rose-pine-dawn" => "Rose Pine Dawn",
        "rose-pine" => "Rose Pine",
        "high-contrast-light" => "High Contrast Light",
        "high-contrast-dark" => "High Contrast Dark",
        "extreme-high-contrast" => "Extreme High Contrast",
        "theme2" => "Theme 2",
        "theme3" => "Theme 3",
        "theme4" => "Theme 4",
        "theme5" => "Theme 5",
        "extreme-high-contrast-dark" => "Extreme High Contrast Dark",
        "theme2dark" => "Theme 2 Dark",
        "theme3dark" => "Theme 3 Dark",
        "theme4dark" => "Theme 4 Dark",
        "theme5dark" => "Theme 5 Dark",
        _ => "Unknown Theme",
    }
}

/// Find the index of a theme ID within a list of IDs.
fn index_for_id(ids: &[&'static str], target: &str) -> usize {
    ids.iter().position(|id| *id == target).unwrap_or(0)
}

/// Font name for a given dropdown index.
fn font_name_for_index(index: usize) -> &'static str {
    match index {
        1 => "OpenDyslexic",
        _ => "Noto Sans",
    }
}

/// Find the index of a font name in the dropdown.
fn index_for_font(name: &str) -> usize {
    match name {
        "OpenDyslexic" => 1,
        _ => 0,
    }
}
