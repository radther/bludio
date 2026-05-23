//! Left-mounted vertical icon tab bar component.
//!
//! Renders a fixed-width (48px) column of icon-only tabs. Each tab displays an
//! SVG icon. The active tab is visually highlighted; all inactive tabs show a
//! hover effect. Click callbacks are wired through closures.

use crate::ui::v_flex;
use gpui::{CursorStyle, Div, MouseButton, MouseUpEvent, SharedString, div, hsla, prelude::*, px};

/// The width of the tab bar column.
pub const TAB_BAR_WIDTH: f32 = 48.0;

/// The height of an individual tab button.
pub const TAB_HEIGHT: f32 = 48.0;

/// Configuration for a single tab.
pub struct Tab {
    /// Function that returns the icon element for this tab.
    pub icon: fn() -> gpui::Svg,
    /// Tooltip text shown on hover.
    pub tooltip: &'static str,
}

/// Render a vertical icon-only tab bar.
///
/// * `tabs` — Slice of tab configurations.
/// * `active_index` — Index into `tabs` of the currently active tab.
/// * `on_click` — Callback invoked with the clicked tab index on mouse-up.
pub fn tab_bar_view(
    tabs: &[Tab],
    active_index: usize,
    on_click: impl Fn(usize, &mut gpui::Window, &mut gpui::App) + 'static,
) -> Div {
    let bg = hsla(0.0, 0.0, 0.12, 1.0);
    let hover_color = hsla(0.0, 0.0, 1.0, 0.06);
    let highlight = hsla(210.0 / 360.0, 0.5, 0.35, 0.5);

    v_flex()
        .w(px(TAB_BAR_WIDTH))
        .h_full()
        .bg(bg)
        .border_r_1()
        .border_color(hsla(0.0, 0.0, 0.25, 1.0))
        .children(tabs.iter().enumerate().map({
            let on_click = std::rc::Rc::new(on_click);
            move |(i, tab)| {
                let is_active = i == active_index;
                let icon_fn = tab.icon;
                let on_click = on_click.clone();

                div()
                    .id(SharedString::from(format!("tab-{i}")))
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(TAB_BAR_WIDTH))
                    .h(px(TAB_HEIGHT))
                    .cursor(CursorStyle::PointingHand)
                    .when(is_active, |el| el.bg(highlight))
                    .tooltip(crate::ui::tooltip::tooltip_text(tab.tooltip))
                    .hover(move |el| {
                        if is_active {
                            el.bg(highlight)
                        } else {
                            el.bg(hover_color)
                        }
                    })
                    .child(icon_fn().text_color(hsla(0.0, 0.0, 0.9, 1.0)))
                    .on_mouse_up(MouseButton::Left, move |_: &MouseUpEvent, window, cx| {
                        on_click(i, window, cx);
                    })
            }
        }))
}
