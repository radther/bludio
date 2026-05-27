//! Page header component: heading + caption in a vertical stack.
//!
//! Stateless element builder — no interactivity, just layout.
//! Callers add their own padding, backgrounds, and borders.

use crate::ui::StyledExt;
use crate::ui::theme::{TextStyleSet, ThemeColors};
use crate::ui::v_flex;
use gpui::{SharedString, div, prelude::*};

/// A page header with a heading and caption vertically stacked.
///
/// Receives theme tokens so styling can be applied inside the component.
pub(crate) fn page_header(
    heading: impl Into<SharedString>,
    caption: impl Into<SharedString>,
    colors: &ThemeColors,
    text_styles: &TextStyleSet,
) -> gpui::Div {
    v_flex()
        .px_4()
        .pt_2()
        .mb_2()
        .child(div().styled(text_styles.heading).child(heading.into()))
        .child(
            div()
                .styled(text_styles.caption)
                .text_color(colors.text_secondary)
                .child(caption.into()),
        )
}
