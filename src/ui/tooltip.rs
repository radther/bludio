//! Lightweight tooltip component.
//!
//! Provides a `tooltip_text()` helper that returns the closure expected
//! by GPUI's `.tooltip()` builder, rendering a simple styled label.

use gpui::{IntoElement, Render, SharedString, Window, div, hsla, prelude::*};

/// A minimal tooltip view that renders a single line of text.
struct TooltipLabel {
    text: SharedString,
}

impl Render for TooltipLabel {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .bg(hsla(0.0, 0.0, 0.14, 1.0))
            .text_color(hsla(0.0, 0.0, 0.95, 1.0))
            .rounded_md()
            .text_sm()
            .child(self.text.clone())
    }
}

/// Returns a closure suitable for GPUI's `.tooltip()` builder.
///
/// Usage:
/// ```ignore
/// div().tooltip(tooltip_text("My tooltip"))
/// ```
pub fn tooltip_text(
    text: impl Into<SharedString>,
) -> impl Fn(&mut Window, &mut gpui::App) -> gpui::AnyView {
    let text = text.into();
    move |_, cx| cx.new(|_| TooltipLabel { text: text.clone() }).into()
}
