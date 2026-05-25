//! Left-mounted vertical icon tab bar component.
//!
//! Renders a fixed-width (48px) column of icon-only tabs. Each tab displays an
//! SVG icon. The active tab is visually highlighted; all inactive tabs show a
//! hover effect. Click callbacks are wired through closures.

use crate::ui::h_flex;
use crate::ui::theme;
use crate::ui::v_flex;
use gpui::{App, CursorStyle, Div, MouseButton, MouseUpEvent, SharedString, prelude::*, px};

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
/// * `cx` — Application context for reading the theme.
pub fn tab_bar_view(
    tabs: &[Tab],
    active_index: usize,
    on_click: impl Fn(usize, &mut gpui::Window, &mut gpui::App) + 'static,
    cx: &App,
) -> Div {
    let colors = &theme::theme(cx).colors;

    v_flex()
        .w(px(TAB_BAR_WIDTH))
        .h_full()
        .bg(colors.sidebar)
        .border_r_1()
        .border_color(colors.border)
        .children(tabs.iter().enumerate().map({
            let on_click = std::rc::Rc::new(on_click);
            let highlight = colors.accent_hover;
            let icon_color = colors.icon;
            let hover_overlay = colors.hover_overlay;
            move |(i, tab)| {
                let is_active = i == active_index;
                let icon_fn = tab.icon;
                let on_click = on_click.clone();

                h_flex()
                    .id(SharedString::from(format!("tab-{i}")))
                    .justify_center()
                    .w(px(TAB_BAR_WIDTH))
                    .h(px(TAB_HEIGHT))
                    .cursor(CursorStyle::PointingHand)
                    .when(is_active, move |el| el.bg(highlight))
                    .tooltip(crate::ui::tooltip::tooltip_text(tab.tooltip))
                    .hover(move |el| {
                        if is_active {
                            el.bg(highlight)
                        } else {
                            el.bg(hover_overlay)
                        }
                    })
                    .child(icon_fn().text_color(icon_color))
                    .on_mouse_up(MouseButton::Left, move |_: &MouseUpEvent, window, cx| {
                        on_click(i, window, cx);
                    })
            }
        }))
}
