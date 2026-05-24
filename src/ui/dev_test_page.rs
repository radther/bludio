//! Developer test page: sandbox for testing components.
//!
//! A self-contained entity that owns interactive components and a log of
//! confirmed entries. Renders an input field + scrollable list of confirmed
//! entries. Originally built for TextField testing, now a general dev sandbox.

use gpui::{
    App, Context, CursorStyle, Entity, FocusHandle, Focusable, FontWeight, MouseButton,
    MouseUpEvent, Render, SharedString, Subscription, Window, div, hsla, prelude::*, px,
};

use crate::ui::components::text_field::{TextField, TextFieldEvent};
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
        let accent = hsla(210.0 / 360.0, 0.7, 0.55, 1.0);
        let surface = hsla(0.0, 0.0, 0.14, 1.0);
        let text_secondary = hsla(0.0, 0.0, 0.6, 1.0);

        let focus_handle = self.text_field.read(cx).focus_handle(cx);

        v_flex()
            .flex_1()
            .child(
                // Header
                h_flex()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .bg(surface)
                    .border_b_1()
                    .border_color(hsla(0.0, 0.0, 0.25, 1.0))
                    .child(div().font_weight(FontWeight::BOLD).child("Text Field Test")),
            )
            .child(
                // Input area
                v_flex()
                    .px_4()
                    .py_3()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(text_secondary)
                            .child("Type in the field and press Enter to confirm:"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .bg(hsla(0.0, 0.0, 0.18, 1.0))
                            .rounded_md()
                            .border_1()
                            .border_color(hsla(0.0, 0.0, 0.25, 1.0))
                            .px_2()
                            .min_h(px(32.))
                            .child(self.text_field.clone()),
                    )
                    .child(
                        // Click-to-focus helper
                        h_flex().gap_1().child(
                            div()
                                .text_xs()
                                .text_color(text_secondary)
                                .px_2()
                                .py_0p5()
                                .rounded_sm()
                                .bg(hsla(0.0, 0.0, 0.22, 1.0))
                                .cursor(CursorStyle::PointingHand)
                                .hover(|el| el.bg(hsla(0.0, 0.0, 0.3, 1.0)))
                                .on_mouse_up(
                                    MouseButton::Left,
                                    move |_: &MouseUpEvent, window, app| {
                                        window.focus(&focus_handle, app);
                                    },
                                )
                                .child("Click to focus input"),
                        ),
                    ),
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
                            .bg(hsla(0.0, 0.0, 0.11, 1.0))
                            .border_b_1()
                            .border_color(hsla(0.0, 0.0, 0.25, 1.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(accent)
                                    .child(format!("Confirmed ({})", self.confirmed_texts.len())),
                            ),
                    )
                    .children(self.confirmed_texts.iter().enumerate().map(|(i, text)| {
                        h_flex()
                            .px_4()
                            .py_2()
                            .gap_2()
                            .border_b_1()
                            .border_color(hsla(0.0, 0.0, 0.18, 1.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_secondary)
                                    .child(format!("#{}:", i + 1)),
                            )
                            .child(div().child(SharedString::from(text.as_str())))
                            .into_any_element()
                    }))
                    .when(self.confirmed_texts.is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .h(px(120.))
                                .text_color(text_secondary)
                                .text_sm()
                                .child("No entries yet. Type something and press Enter."),
                        )
                    }),
            )
    }
}
