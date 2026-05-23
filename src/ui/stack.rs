//! Layout container free functions.
//!
//! `h_flex()` and `v_flex()` are convenience builders that create a `Div`
//! with pre-configured flex properties.

use gpui::{Div, div, prelude::*};

/// Create a horizontal flex container (`flex` + `flex_row` + `items_center`).
pub fn h_flex() -> Div {
    div().flex().flex_row().items_center()
}

/// Create a vertical flex container (`flex` + `flex_col`).
pub fn v_flex() -> Div {
    div().flex().flex_col()
}
