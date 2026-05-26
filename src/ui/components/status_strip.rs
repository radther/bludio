//! Status strip — a thin colored bar at the left edge of a row.
//!
//! Used by Bluetooth and Audio device rows to indicate connection state,
//! default status, etc.

use gpui::{Div, div, prelude::*, px};

/// Build a left-edge status strip with the given color.
///
/// The strip is a thin vertical bar (8px wide) with a 4px left margin,
/// rounded on the left side. It stretches to fill the parent's height
/// via `absolute` positioning.
pub fn status_strip(color: gpui::Hsla) -> Div {
    div().relative().w_2().ml_2().child(
        div()
            .absolute()
            .left(px(0.))
            .top_0()
            .bottom_0()
            .w_2()
            .rounded_l_md()
            .rounded_r_xs()
            .bg(color),
    )
}
