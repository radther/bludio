//! Toggle switch — a pill-shaped on/off control.
//!
//! A track with a circular thumb that slides left (off) or right (on).
//! When on, the track fills with an accent color.

use gpui::{CursorStyle, Hsla, Stateful, div, prelude::*, px};

use crate::ui::h_flex;
use crate::ui::theme::ThemeColors;

// ── Toggle switch builder ────────────────────────────────────────────────

/// Render a pill-shaped on/off switch.
///
/// Track is a rounded pill; a circular thumb slides left (off) or right (on).
/// When on, the track fills with the accent color.
///
/// # Example
/// ```ignore
/// toggle_switch("my-toggle", true, &colors)
///     .on_click(|_, _, _| {})
/// ```
pub fn toggle_switch(
    id: impl Into<gpui::ElementId>,
    is_on: bool,
    colors: &ThemeColors,
) -> Stateful<gpui::Div> {
    let track_bg = if is_on {
        colors.audio_accent
    } else {
        colors.element_background
    };
    let track_border = if is_on {
        colors.audio_accent
    } else {
        colors.border
    };
    let thumb_color = colors.text;
    let thumb_opacity = if is_on { 1.0 } else { 0.5 };

    div()
        .id(id.into())
        .px(px(1.0))
        .py(px(1.0))
        .cursor(CursorStyle::PointingHand)
        .child(
            h_flex()
                .w(px(40.0))
                .h(px(24.0))
                .rounded_full()
                .px(px(2.0))
                .bg(track_bg)
                .border_1()
                .border_color(track_border)
                .when(is_on, |on| on.justify_end())
                .when(!is_on, |off| off.justify_start())
                .child(
                    div()
                        .size(px(18.0))
                        .rounded_full()
                        .bg(thumb_color)
                        .opacity(thumb_opacity),
                ),
        )
}

/// Render a toggle switch using a custom color instead of the default accent.
#[allow(dead_code)]
pub fn toggle_switch_with_color(
    id: impl Into<gpui::ElementId>,
    is_on: bool,
    active_color: Hsla,
    colors: &ThemeColors,
) -> Stateful<gpui::Div> {
    let track_bg = if is_on {
        active_color
    } else {
        colors.element_background
    };
    let track_border = if is_on { active_color } else { colors.border };
    let thumb_color = colors.text;
    let thumb_opacity = if is_on { 1.0 } else { 0.5 };

    div()
        .id(id.into())
        .px(px(1.0))
        .py(px(1.0))
        .cursor(CursorStyle::PointingHand)
        .child(
            h_flex()
                .w(px(40.0))
                .h(px(24.0))
                .rounded_full()
                .px(px(2.0))
                .bg(track_bg)
                .border_1()
                .border_color(track_border)
                .when(is_on, |on| on.justify_end())
                .when(!is_on, |off| off.justify_start())
                .child(
                    div()
                        .size(px(18.0))
                        .rounded_full()
                        .bg(thumb_color)
                        .opacity(thumb_opacity),
                ),
        )
}
