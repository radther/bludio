//! Error banner component: animated bottom banner for transient error messages.
//!
//! Stateless element builder — manages animation internally but owns no state.
//! Callers manage visibility via `Option<String>` and generation counter.

use std::time::Duration;

use crate::ui::StyledExt;
use crate::ui::h_flex;
use crate::ui::theme::{TextStyleSet, ThemeColors};
use gpui::{
    Animation, AnimationElement, AnimationExt, App, ClickEvent, CursorStyle, Div,
    InteractiveElement, Stateful, Window, div, ease_out_quint, prelude::*, px,
};

/// Slide-up animation duration.
const ANIMATION_DURATION: Duration = Duration::from_millis(300);

/// Banner height in pixels.
const BANNER_HEIGHT: f32 = 40.0;

/// Render an animated error banner that slides up from the bottom.
///
/// - `text`: The error message to display.
/// - `generation`: Incremented on each new error to restart the animation.
/// - `colors`: Theme colors for styling.
/// - `text_styles`: Theme text styles for the error text.
/// - `on_dismiss`: Called when the user clicks the banner.
pub(crate) fn error_banner(
    text: &str,
    generation: u64,
    colors: &ThemeColors,
    text_styles: &TextStyleSet,
    on_dismiss: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnimationElement<Stateful<Div>> {
    h_flex()
        .id(("error-banner", generation))
        .w_full()
        .h(px(BANNER_HEIGHT))
        .px_4()
        .items_center()
        .bg(colors.sidebar)
        .border_t_1()
        .border_color(colors.border)
        .cursor(CursorStyle::PointingHand)
        .child(
            div()
                .styled(text_styles.caption)
                .text_color(colors.danger)
                .child(text.to_string()),
        )
        .on_click(on_dismiss)
        .with_animation(
            format!("error-banner-anim-{generation}"),
            Animation::new(ANIMATION_DURATION).with_easing(ease_out_quint()),
            move |el, delta| {
                // Slide up from below: offset starts at BANNER_HEIGHT, ends at 0
                el.relative().top(px((1.0 - delta) * BANNER_HEIGHT))
            },
        )
}
