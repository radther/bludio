//! Animation helpers for GPUI elements.
//!
//! Provides `with_fade_in` and `with_fade_in_up` for any [`Styled`] element.

use std::time::Duration;

use gpui::{
    Animation, AnimationElement, AnimationExt, ElementId, IntoElement, Styled, ease_out_quint, px,
};

// ── Fade-in animation ──────────────────────────────────────────────────────

/// Extension trait for fade-in animations on any styled element.
pub trait FadeInAnimationExt: Styled + Sized + IntoElement + 'static {
    /// Animate this element's opacity from 0% to 100% over `duration_ms`.
    /// Uses an ease-out-quint curve for a fast-in feel.
    fn with_fade_in(self, id: impl Into<ElementId>, duration_ms: u64) -> AnimationElement<Self> {
        self.with_animation(
            id,
            Animation::new(Duration::from_millis(duration_ms)).with_easing(ease_out_quint()),
            |this, delta| this.opacity(delta),
        )
    }

    /// Animate this element upward while fading in.
    ///
    /// The element starts `40 * count` pixels below its final position and
    /// animates up while opacity goes from 0% to 100%. Increase `count`
    /// for each successive item (e.g. header = 0, first list item = 1,
    /// second = 2, …) so each element has a larger staggered entrance.
    fn with_fade_in_up(
        self,
        id: impl Into<ElementId>,
        duration_ms: u64,
        count: usize,
    ) -> AnimationElement<Self> {
        self.with_animation(
            id,
            Animation::new(Duration::from_millis(duration_ms)).with_easing(ease_out_quint()),
            move |this, delta| {
                let offset = 40.0 * count as f32;
                this.relative()
                    .top(px((1.0 - delta) * offset))
                    .opacity(delta)
            },
        )
    }
}

impl<T: Styled + Sized + IntoElement + 'static> FadeInAnimationExt for T {}
