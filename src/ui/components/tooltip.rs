//! Lightweight tooltip component.
//!
//! Provides a `tooltip_text()` helper that returns the closure expected
//! by GPUI's `.tooltip()` builder, rendering a simple styled label.

use gpui::{IntoElement, Render, SharedString, Window, div, prelude::*};

use crate::ui::StyledExt;
use crate::ui::theme::{self, text_styles};

/// A minimal tooltip view that renders a single line of text.
struct TooltipLabel {
    text: SharedString,
}

impl Render for TooltipLabel {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let colors = &theme::theme(cx).colors;
        let text_styles = text_styles(cx);
        div()
            .px_2()
            .py_1()
            .bg(colors.surface)
            .text_color(colors.text)
            .rounded_md()
            .styled(text_styles.caption)
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
