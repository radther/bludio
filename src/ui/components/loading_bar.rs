//! Animated loading bar component.
//!
//! An edge-to-edge bar with a smaller inner bar that bounces left-to-right,
//! back again, pauses, and repeats. The inner bar has alpha gradients on both
//! sides (transparent → solid → transparent).

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, Div, Hsla, div, ease_in_out, linear_color_stop, linear_gradient,
    prelude::*, relative,
};

use crate::ui::h_flex;

// ── Constants ────────────────────────────────────────────────────────────

/// Full cycle duration in milliseconds (right + left + pause).
const CYCLE_MS: f32 = 1000.0;
/// Duration of the rightward movement phase in milliseconds.
const RIGHT_MS: f32 = 400.0;
/// Duration of the leftward movement phase in milliseconds.
const LEFT_MS: f32 = 400.0;
/// Fraction of the inner bar width relative to the parent container.
const INNER_BAR_WIDTH_FRAC: f32 = 0.25;
/// Fraction of the inner bar used for each fade edge.
const FADE_FRAC: f32 = 2.6 / 3.0;

// ── Cycle math ───────────────────────────────────────────────────────────

/// Map an animation delta (0.0-1.0 over the full cycle) to a position
/// fraction (0.0-1.0) representing how far across the track the inner bar
/// should be placed.
fn cycle_position(delta: f32) -> f32 {
    let phase1_end = RIGHT_MS / CYCLE_MS;
    let phase2_end = phase1_end + (LEFT_MS / CYCLE_MS);

    if delta < phase1_end {
        // Moving right: 0.0 → 1.0
        let t = delta / phase1_end;
        ease_in_out(t)
    } else if delta < phase2_end {
        // Moving left: 1.0 → 0.0
        let t = (delta - phase1_end) / (phase2_end - phase1_end);
        ease_in_out(1.0 - t)
    } else {
        // Pause at left
        0.0
    }
}

// ── Builder ────────────────────────────────────────────────────────────────

/// Build an animated loading bar with the given solid color.
///
/// The bar spans the full width of its container. The inner "thumb" is
/// 25 % of the container width and has transparent-to-solid gradients on
/// both ends. It starts fully off-screen left, sweeps across to fully
/// off-screen right, returns left, pauses 500 ms, then repeats.
///
/// When animations are globally disabled, the bar is rendered as a static
/// full-width solid color instead.
pub(crate) fn loading_bar(id: impl Into<gpui::ElementId>, color: Hsla) -> Div {
    if crate::ui::accessibility::disable_animations() {
        return div().w_full().h_1().bg(color);
    }

    let fade_width = relative(FADE_FRAC);
    let transparent = color.opacity(0.0);

    let inner_bar = h_flex()
        .w(relative(INNER_BAR_WIDTH_FRAC))
        .h_1()
        .child(div().w(fade_width).h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(transparent, 0.0),
            linear_color_stop(color, 1.0),
        )))
        .child(div().flex_1().h_full().bg(color))
        .child(div().w(fade_width).h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(color, 0.0),
            linear_color_stop(transparent, 1.0),
        )));

    let min_offset = -INNER_BAR_WIDTH_FRAC;
    let max_offset = 1.0;
    let offset_range = max_offset - min_offset;

    div()
        .w_full()
        .h_1()
        .relative()
        .overflow_hidden()
        .child(inner_bar.with_animation(
            id,
            Animation::new(Duration::from_millis(CYCLE_MS as u64)).repeat(),
            move |bar, delta| {
                let position = cycle_position(delta);
                bar.left(relative(min_offset + position * offset_range))
            },
        ))
}
