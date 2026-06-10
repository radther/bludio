//! Developer test page: sandbox for testing components and theme switching.
//!
//! A self-contained entity that owns interactive components and a log of
//! confirmed entries. Renders an input field + scrollable list of confirmed
//! entries + theme toggle buttons.

use gpui::{
    App, ClickEvent, Context, CursorStyle, Entity, FocusHandle, Focusable, Render, SharedString,
    Subscription, Window, div, prelude::*, px,
};

use crate::backend::settings::update_settings;
use crate::ui::StyledExt;
use crate::ui::common::animation::FadeInAnimationExt;
use crate::ui::components::loading_bar::loading_bar;
use crate::ui::components::page_header::page_header;
use crate::ui::components::text_field::{TextField, TextFieldEvent};
use crate::ui::theme;
use crate::ui::theme::TextStyleSet;
use crate::ui::{h_flex, v_flex};

// ── Test page entity ───────────────────────────────────────────────────────

pub(crate) struct DevTestPage {
    text_field: Entity<TextField>,
    confirmed_texts: Vec<String>,
    focus_handle: FocusHandle,
    _text_field_sub: Subscription,
}

impl DevTestPage {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let text_field = cx.new(|cx| TextField::new(cx).placeholder("Type something..."));
        let _text_field_sub =
            cx.subscribe(
                &text_field,
                |this, _tf, event: &TextFieldEvent, cx| match event {
                    TextFieldEvent::Confirmed(text) => {
                        this.confirmed_texts.push(text.clone());
                        cx.notify();
                    }
                    TextFieldEvent::Cancelled => {}
                },
            );
        Self {
            text_field,
            confirmed_texts: Vec::new(),
            focus_handle: cx.focus_handle(),
            _text_field_sub,
        }
    }
}

impl Focusable for DevTestPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DevTestPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &theme::theme(cx).colors;
        let text_styles = TextStyleSet::default();
        let is_dark = theme::theme(cx).appearance == theme::Appearance::Dark;

        let focus_handle = self.text_field.read(cx).focus_handle(cx);

        v_flex()
            .flex_1()
            .child(
                // Header with theme switcher
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .child(page_header(
                        "Text Field Test",
                        "Sandbox for testing UI components",
                        colors,
                        &text_styles,
                    ))
                    .child(h_flex().gap_2().children(vec![
                        dark_theme_btn(is_dark, colors, &text_styles),
                        light_theme_btn(is_dark, colors, &text_styles),
                    ]))
                    .with_fade_in_up("dev-test-header", 1),
            )
            .child(
                // Input area
                v_flex()
                    .px_4()
                    .py_3()
                    .gap_2()
                    .child(
                        div()
                            .styled(text_styles.caption)
                            .text_color(colors.text_secondary)
                            .child("Type in the field and press Enter to confirm:"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .bg(colors.input_background)
                            .rounded_md()
                            .border_1()
                            .border_color(colors.border)
                            .px_2()
                            .min_h(px(32.))
                            .child(self.text_field.clone()),
                    )
                    .child(
                        h_flex().gap_1().child(
                            div()
                                .id("focus-input-btn")
                                .styled(text_styles.caption)
                                .text_color(colors.text_secondary)
                                .px_2()
                                .py_0p5()
                                .rounded_sm()
                                .bg(colors.element_background)
                                .cursor(CursorStyle::PointingHand)
                                .hover(|el| el.bg(colors.element_hover))
                                .on_click(move |_: &ClickEvent, window, app| {
                                    window.focus(&focus_handle, app);
                                })
                                .child("Click to focus input"),
                        ),
                    )
                    .with_fade_in_up("dev-test-header", 2),
            )
            .child(
                // Confirmed items list
                v_flex()
                    .flex_1()
                    .id("test-confirmed-list")
                    .overflow_y_scroll()
                    .child(
                        h_flex()
                            .px_4()
                            .py_2()
                            .bg(colors.surface)
                            .border_b_1()
                            .border_color(colors.border)
                            .child(
                                div()
                                    .styled(text_styles.heading)
                                    .text_color(colors.dev_accent)
                                    .child(format!("Confirmed ({})", self.confirmed_texts.len())),
                            ),
                    )
                    .children(self.confirmed_texts.iter().enumerate().map(|(i, text)| {
                        h_flex()
                            .px_4()
                            .py_2()
                            .gap_2()
                            .border_b_1()
                            .border_color(colors.border)
                            .child(
                                div()
                                    .styled(text_styles.caption)
                                    .text_color(colors.text_secondary)
                                    .child(format!("#{}:", i + 1)),
                            )
                            .child(div().child(SharedString::from(text.as_str())))
                            .with_fade_in_up(format!("dev-test-item-{i}"), i + 1)
                            .into_any_element()
                    }))
                    .when(self.confirmed_texts.is_empty(), |el| {
                        el.child(
                            h_flex()
                                .justify_center()
                                .h(px(120.))
                                .text_color(colors.text_secondary)
                                .styled(text_styles.body)
                                .child("No entries yet. Type something and press Enter."),
                        )
                    })
                    .with_fade_in_up("dev-test-header", 3),
            )
            .child(loading_bar("dev-test-loading-bar", colors.dev_accent))
    }
}

// ── Theme toggle buttons ───────────────────────────────────────────────────

fn dark_theme_btn(
    is_dark: bool,
    colors: &crate::ui::theme::ThemeColors,
    text_styles: &crate::ui::theme::TextStyleSet,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id("theme-dark-btn")
        .px_2()
        .py_1()
        .rounded_sm()
        .styled(text_styles.caption)
        .bg(if is_dark {
            colors.dev_accent
        } else {
            colors.element_background
        })
        .cursor(CursorStyle::PointingHand)
        .hover(|el| {
            el.bg(if is_dark {
                colors.dev_accent
            } else {
                colors.element_hover
            })
        })
        .child("Dark")
        .on_click(|_: &ClickEvent, _, cx| {
            update_settings(|s| s.theme_mode = theme::ThemeMode::Dark, cx);
        })
}

fn light_theme_btn(
    is_dark: bool,
    colors: &crate::ui::theme::ThemeColors,
    text_styles: &crate::ui::theme::TextStyleSet,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id("theme-light-btn")
        .px_2()
        .py_1()
        .rounded_sm()
        .styled(text_styles.caption)
        .bg(colors.dev_accent)
        .cursor(CursorStyle::PointingHand)
        .hover(|el| {
            el.bg(if !is_dark {
                colors.dev_accent
            } else {
                colors.element_hover
            })
        })
        .child("Light")
        .on_click(|_: &ClickEvent, _, cx| {
            update_settings(|s| s.theme_mode = theme::ThemeMode::Light, cx);
        })
}
