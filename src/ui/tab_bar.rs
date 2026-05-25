//! Left-mounted vertical icon tab bar entity.
//!
//! Renders a fixed-width (48px) column of icon-only tabs with an animated
//! selection indicator. Tab clicks emit `TabBarEvent::TabClicked(usize)` up
//! to the parent, which then calls `set_active_index()` to update the visual
//! state (App-as-source-of-truth data flow).
//!
//! Animation: a 3px accent-colored bar slides from the old tab position to
//! the new one using ease-out cubic interpolation over ~250ms. Interrupted
//! animations are handled via a generation counter.

use crate::ui::h_flex;
use crate::ui::theme;
use crate::ui::tooltip;
use crate::ui::v_flex;
use gpui::{
    App, Context, CursorStyle, EventEmitter, FocusHandle, Focusable, MouseButton, MouseUpEvent,
    Render, SharedString, Window, div, prelude::*, px,
};
use std::time::{Duration, Instant};

// ── Constants ──────────────────────────────────────────────────────────────

/// Width of the tab bar column.
pub const TAB_BAR_WIDTH: f32 = 48.0;

/// Height of an individual tab button.
pub const TAB_HEIGHT: f32 = 48.0;

/// Width of the sliding selection indicator bar.
const INDICATOR_WIDTH: f32 = 3.0;

/// Duration of the slide animation.
const ANIMATION_DURATION: Duration = Duration::from_millis(250);

// ── Tab configuration ──────────────────────────────────────────────────────

/// Configuration for a single tab.
pub struct Tab {
    /// Function that returns the icon element for this tab.
    pub icon: fn() -> gpui::Svg,
    /// Tooltip text shown on hover.
    pub tooltip: &'static str,
}

// ── Events ─────────────────────────────────────────────────────────────────

/// Events emitted by the TabBar for parent views to handle.
#[derive(Clone, Debug)]
pub(crate) enum TabBarEvent {
    /// User clicked a tab. Contains the tab index.
    TabClicked(usize),
}

// ── Entity ─────────────────────────────────────────────────────────────────

/// Left-mounted vertical icon tab bar with animated selection indicator.
///
/// The **app** is the source of truth for which page is active.
/// The tab bar reflects this through `set_active_index()`, which starts
/// a slide animation from the previous visual position.
///
/// Ownership chain:
///   `BludioApp` → Entity<TabBar>
pub(crate) struct TabBar {
    tabs: Vec<Tab>,
    /// The currently active tab index (set by parent via `set_active_index`).
    active_index: usize,
    /// Visual Y-position of the selection indicator (smoothly interpolated).
    indicator_offset: f32,
    /// Incremented on each selection change; stale animation tasks check this
    /// to avoid overwriting position with out-of-date values.
    animation_generation: u64,
    focus_handle: FocusHandle,
}

impl TabBar {
    /// Create a new tab bar entity.
    pub(crate) fn new(tabs: Vec<Tab>, active_index: usize, cx: &mut Context<Self>) -> Self {
        Self {
            indicator_offset: active_index as f32 * TAB_HEIGHT,
            active_index,
            tabs,
            animation_generation: 0,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Set the active tab index and animate the selection indicator.
    ///
    /// This is the **single entry point** for changing the selected tab.
    /// Call this after updating the app's logical active page, regardless
    /// of whether the trigger was a click, keyboard shortcut, etc.
    pub(crate) fn set_active_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index == self.active_index {
            return;
        }
        self.active_index = index;
        let from = self.indicator_offset;
        self.animation_generation = self.animation_generation.wrapping_add(1);
        let generation = self.animation_generation;
        let target = index as f32 * TAB_HEIGHT;

        // ── Spawn animation driver (frame loop) ──
        cx.spawn(async move |this, cx| {
            let start = Instant::now();
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let elapsed = start.elapsed();
                let raw_t = (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
                // Ease-out cubic interpolation
                let t = 1.0 - (1.0 - raw_t).powi(3);
                let pos = from + (target - from) * t;
                let done = raw_t >= 1.0;

                if let Some(this) = this.upgrade() {
                    this.update(cx, |this, cx| {
                        // Only apply if this is still the latest animation
                        if this.animation_generation == generation {
                            this.indicator_offset = pos;
                            cx.notify();
                        }
                    });
                } else {
                    break;
                }
                if done {
                    break;
                }
            }
        })
        .detach();

        cx.notify();
    }
}

// ── EventEmitter + Focusable ──────────────────────────────────────────────

impl EventEmitter<TabBarEvent> for TabBar {}

impl Focusable for TabBar {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// ── Render ────────────────────────────────────────────────────────────────

impl Render for TabBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &theme::theme(cx).colors;
        let indicator_y = px(self.indicator_offset);
        let active_index = self.active_index;
        let entity = cx.entity().clone();

        // Build tab button elements
        let tab_buttons = self.tabs.iter().enumerate().map({
            let entity = entity.clone();
            move |(i, tab)| {
                let is_active = i == active_index;
                h_flex()
                    .id(SharedString::from(format!("tab-{i}")))
                    .justify_center()
                    .w(px(TAB_BAR_WIDTH))
                    .h(px(TAB_HEIGHT))
                    .cursor(CursorStyle::PointingHand)
                    .tooltip(tooltip::tooltip_text(tab.tooltip))
                    .hover(move |el| {
                        if is_active {
                            el.bg(colors.accent_hover)
                        } else {
                            el.bg(colors.hover_overlay)
                        }
                    })
                    .child((tab.icon)().text_color(if is_active {
                        colors.accent
                    } else {
                        colors.icon
                    }))
                    .on_mouse_up(MouseButton::Left, {
                        let entity = entity.clone();
                        move |_: &MouseUpEvent, _window, app_cx: &mut App| {
                            entity.update(app_cx, |_this, entity_cx| {
                                entity_cx.emit(TabBarEvent::TabClicked(i));
                            });
                        }
                    })
            }
        });

        v_flex()
            .w(px(TAB_BAR_WIDTH))
            .h_full()
            .bg(colors.sidebar)
            .border_r_1()
            .border_color(colors.border)
            .relative()
            // ── Animated selection indicator ──
            .child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .top(indicator_y)
                    .w(px(INDICATOR_WIDTH))
                    .h(px(TAB_HEIGHT))
                    .bg(colors.accent)
                    .rounded_r(px(2.0)),
            )
            .children(tab_buttons)
    }
}
