//! Extension trait for ergonomic GPUI `Styled` builder chaining.
//!
//! Provides `.styled()` and `debug_bg_*()` color helpers.
//! Blanket-implemented for all `Styled + Sized` types.

use gpui::{AbsoluteLength, FontWeight, Styled, hsla};

/// Extension methods on any `Styled` type for common layout and debug helpers.
pub trait StyledExt: Styled + Sized {
    // ── TextStyle convenience ────────────────────────────────────────────

    /// Apply font weight and font size from a predefined text style tuple.
    ///
    /// The tuple is typically a field from [`TextStyleSet`] (e.g.,
    /// `text_styles.heading`). Font family is inherited from the root
    /// element via GPUI's cascade.
    fn styled(self, style: (AbsoluteLength, FontWeight)) -> Self {
        self.text_size(style.0).font_weight(style.1)
    }

    // ── Debug background helpers (no theme dependency) ─────────────────────

    #[allow(dead_code)]
    fn debug_bg_red(self) -> Self {
        self.bg(hsla(0. / 360., 1., 0.5, 1.))
    }
    #[allow(dead_code)]
    fn debug_bg_green(self) -> Self {
        self.bg(hsla(120. / 360., 1., 0.5, 1.))
    }
    #[allow(dead_code)]
    fn debug_bg_blue(self) -> Self {
        self.bg(hsla(240. / 360., 1., 0.5, 1.))
    }
    #[allow(dead_code)]
    fn debug_bg_yellow(self) -> Self {
        self.bg(hsla(60. / 360., 1., 0.5, 1.))
    }
    #[allow(dead_code)]
    fn debug_bg_cyan(self) -> Self {
        self.bg(hsla(160. / 360., 1., 0.5, 1.))
    }
    #[allow(dead_code)]
    fn debug_bg_magenta(self) -> Self {
        self.bg(hsla(300. / 360., 1., 0.5, 1.))
    }
}

impl<T: Styled + Sized> StyledExt for T {}
