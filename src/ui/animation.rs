//! Animation helpers for GPUI elements.
//!
//! Provides `with_fade_in` and `with_fade_in_up` for any [`Styled`] element.

use std::time::Duration;

use gpui::{
    Animation, AnimationElement, AnimationExt, ElementId, IntoElement, Styled, ease_out_quint, px,
};

use crate::ui::accessibility::disable_animations;

// ── Fade-in animation ──────────────────────────────────────────────────────

/// Extension trait for fade-in animations on any styled element.
pub trait FadeInAnimationExt: Styled + Sized + IntoElement + 'static {
    /// Animate this element upward while fading in.
    ///
    /// The element starts `20 * count` pixels below its final position and
    /// animates up while opacity goes from 30% to 100%. Increase `count`
    /// for each successive item (e.g. header = 0, first list item = 1,
    /// second = 2, …) so each element has a larger staggered entrance.
    ///
    /// When `disable_animations` is true, the animation duration is zero
    /// so the element appears immediately.
    fn with_fade_in_up(self, id: impl Into<ElementId>, count: usize) -> AnimationElement<Self> {
        let duration = if disable_animations() {
            Duration::ZERO
        } else {
            Duration::from_millis(400)
        };
        self.with_animation(
            id,
            Animation::new(duration).with_easing(ease_out_quint()),
            move |this, delta| {
                let offset = 20.0 * count as f32;
                this.relative()
                    .top(px((1.0 - delta) * offset))
                    .opacity(0.3 + delta * 0.7)
            },
        )
    }
}

impl<T: Styled + Sized + IntoElement + 'static> FadeInAnimationExt for T {}
