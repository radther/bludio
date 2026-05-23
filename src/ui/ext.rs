//! Extension trait for ergonomic GPUI `Styled` builder chaining.
//!
//! Provides `.h_flex()`, `.v_flex()`, and `debug_bg_*()` color helpers.
//! Blanket-implemented for all `Styled + Sized` types.

use gpui::{Styled, hsla};

/// Extension methods on any `Styled` type for common layout and debug helpers.
#[allow(dead_code)]
pub trait StyledExt: Styled + Sized {
    /// Apply `flex()`, `flex_row()`, `items_center()` to create a horizontal flex container.
    fn h_flex(self) -> Self {
        self.flex().flex_row().items_center()
    }

    /// Apply `flex()`, `flex_col()` to create a vertical flex container.
    fn v_flex(self) -> Self {
        self.flex().flex_col()
    }

    // ── Debug background helpers (no theme dependency) ─────────────────────

    fn debug_bg_red(self) -> Self {
        self.bg(hsla(0. / 360., 1., 0.5, 1.))
    }
    fn debug_bg_green(self) -> Self {
        self.bg(hsla(120. / 360., 1., 0.5, 1.))
    }
    fn debug_bg_blue(self) -> Self {
        self.bg(hsla(240. / 360., 1., 0.5, 1.))
    }
    fn debug_bg_yellow(self) -> Self {
        self.bg(hsla(60. / 360., 1., 0.5, 1.))
    }
    fn debug_bg_cyan(self) -> Self {
        self.bg(hsla(160. / 360., 1., 0.5, 1.))
    }
    fn debug_bg_magenta(self) -> Self {
        self.bg(hsla(300. / 360., 1., 0.5, 1.))
    }
}

impl<T: Styled + Sized> StyledExt for T {}
