//! General-purpose dropdown selector entity.
//!
//! Self-contained entity that renders a trigger button inline and a floating
//! options list via `deferred(anchored(...))` — the options float above
//! content instead of pushing it around.
//!
//! Matches the `TextField` pattern: emits `DropdownEvent`s via `cx.emit()`
//! (`EventEmitter` pattern).

use crate::ui::StyledExt;
use gpui::{
    Anchor, App, Bounds, Context, CursorStyle, DispatchPhase, Entity, EventEmitter, FocusHandle,
    Focusable, IntoElement, KeyDownEvent, MouseButton, MouseUpEvent, Pixels, Render, RenderOnce,
    SharedString, Window, anchored, canvas, deferred, div, prelude::*, px,
};

use crate::ui::v_flex;

// ── Events ─────────────────────────────────────────────────────────────────

/// Events emitted by the Dropdown for parent views to handle.
#[derive(Clone, Debug)]
pub(crate) enum DropdownEvent {
    /// User clicked an item. Contains the index and text.
    Selected(usize, String),
    /// Dropdown was dismissed (Escape, click outside).
    Cancelled,
}

// ── Entity ─────────────────────────────────────────────────────────────────

/// A general-purpose dropdown selector.
///
/// Renders a trigger button inline. When clicked, shows a floating options
/// list positioned at the trigger. Emits `DropdownEvent`s via `cx.emit()`
/// (`EventEmitter` pattern).
pub(crate) struct Dropdown {
    items: Vec<String>,
    selected_index: usize,
    is_open: bool,
    focus_handle: FocusHandle,
    placeholder: SharedString,
    /// Bounds of the trigger element, captured during prepaint for positioning.
    trigger_bounds: Option<Bounds<Pixels>>,
}

impl Dropdown {
    /// Create a new dropdown with the given items and selected index.
    pub fn new(items: Vec<String>, selected_index: usize, cx: &mut Context<Self>) -> Self {
        Self {
            items,
            selected_index,
            is_open: false,
            focus_handle: cx.focus_handle(),
            placeholder: SharedString::from(""),
            trigger_bounds: None,
        }
    }

    /// Replace the item list and selection. Closes the dropdown if open.
    pub fn set_items(&mut self, items: &[String], selected_index: usize, cx: &mut Context<Self>) {
        self.items = items.to_vec();
        self.selected_index = selected_index.clamp(0, items.len().saturating_sub(1));
        self.is_open = false;
        cx.notify();
    }

    /// Get the currently selected text (or placeholder if nothing selected).
    pub fn selected_text(&self) -> SharedString {
        self.items
            .get(self.selected_index)
            .cloned()
            .map_or_else(|| self.placeholder.clone(), SharedString::from)
    }

    /// Whether the dropdown has any items to show.
    pub fn has_items(&self) -> bool {
        !self.items.is_empty()
    }

    // ── Styling builders ────────────────────────────────────────────────

    /// Set the placeholder text shown when no item is selected.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }
}

// ── Focusable ──────────────────────────────────────────────────────────────

impl EventEmitter<DropdownEvent> for Dropdown {}

impl Focusable for Dropdown {
    fn focus_handle(&self, _app: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// ── DropdownComponent (RenderOnce element) ──────────────────────────────

/// The visual representation of a Dropdown.
///
/// Renders a trigger button inline and, when open, a floating options list
/// via `deferred(anchored(...))`. Trigger bounds are captured via canvas.
#[derive(IntoElement)]
struct DropdownComponent {
    entity: Entity<Dropdown>,
}

impl DropdownComponent {
    fn new(entity: Entity<Dropdown>) -> Self {
        Self { entity }
    }
}

impl RenderOnce for DropdownComponent {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let entity = self.entity;

        // Read all state up front.
        let (is_open, selected, items, selected_idx, trigger_pos) = {
            let d = entity.read(cx);
            (
                d.is_open,
                d.selected_text(),
                d.items.clone(),
                d.selected_index,
                d.trigger_bounds.map(|b| b.origin),
            )
        };
        let colors = &crate::ui::theme::theme(cx).colors;
        let text_styles = &crate::ui::theme::theme(cx).text_styles;

        // Register click-outside-to-close when open (must happen during paint, not render).
        let click_outside = if is_open {
            let e = entity.clone();
            Some(
                canvas(
                    |_, _, _| {},
                    move |_, (), window, _cx| {
                        window.on_mouse_event({
                            let e = e.clone();
                            move |_: &MouseUpEvent, phase, _window, cx| {
                                if phase == DispatchPhase::Bubble {
                                    e.update(cx, |this, cx| {
                                        if this.is_open {
                                            this.is_open = false;
                                            cx.emit(DropdownEvent::Cancelled);
                                            cx.notify();
                                        }
                                    });
                                }
                            }
                        });
                    },
                )
                .absolute()
                .size_full(),
            )
        } else {
            None
        };

        // Build the floating menu if open.
        let menu = if is_open && !items.is_empty() {
            let mut anchored_menu = anchored()
                .anchor(Anchor::TopLeft)
                .snap_to_window_with_margin(px(8.));
            if let Some(pos) = trigger_pos {
                anchored_menu = anchored_menu.position(pos);
            }
            Some(
                deferred(
                    anchored_menu.child(
                        v_flex()
                            .bg(colors.menu_background)
                            .border_1()
                            .border_color(colors.menu_border)
                            .rounded_md()
                            .children(items.iter().enumerate().map(|(i, item)| {
                                let item = item.clone();
                                let is_active = i == selected_idx;
                                let entity = entity.clone();
                                div()
                                    .id(SharedString::from(format!("dropdown-item-{i}")))
                                    .px_2()
                                    .py_1()
                                    .styled(text_styles.caption)
                                    .cursor(CursorStyle::PointingHand)
                                    .when(is_active, move |el| el.text_color(colors.accent))
                                    .hover(|el| el.bg(colors.hover_overlay))
                                    .child(SharedString::from(item.clone()))
                                    .on_mouse_up(MouseButton::Left, {
                                        let item = item.clone();
                                        let entity = entity.clone();
                                        window.listener_for(
                                            &entity,
                                            move |this: &mut Dropdown, _, _, cx| {
                                                this.is_open = false;
                                                this.selected_index = i;
                                                cx.emit(DropdownEvent::Selected(i, item.clone()));
                                                cx.notify();
                                            },
                                        )
                                    })
                            })),
                    ),
                )
                .with_priority(1),
            )
        } else {
            None
        };

        let entity_for_key = entity.clone();
        let entity_for_trigger = entity.clone();
        let entity_for_bounds = entity.clone();

        div()
            .track_focus(&entity.read(cx).focus_handle(cx))
            .on_key_down(window.listener_for(
                &entity_for_key,
                move |this: &mut Dropdown, event: &KeyDownEvent, _window, cx| {
                    let key = event.keystroke.key.clone();
                    match key.as_str() {
                        "escape" => {
                            this.is_open = false;
                            cx.emit(DropdownEvent::Cancelled);
                            cx.notify();
                        }
                        "enter" if this.is_open => {
                            let idx = this.selected_index;
                            if let Some(text) = this.items.get(idx).cloned() {
                                this.is_open = false;
                                cx.emit(DropdownEvent::Selected(idx, text));
                                cx.notify();
                            }
                        }
                        "up" | "down" if this.is_open => {
                            let len = this.items.len();
                            if len > 0 {
                                this.selected_index = if key == "up" {
                                    this.selected_index.wrapping_sub(1) % len
                                } else {
                                    (this.selected_index + 1) % len
                                };
                                cx.notify();
                            }
                        }
                        _ => {}
                    }
                },
            ))
            .child(
                // ── Trigger button ──
                div()
                    .id("dropdown-trigger")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .styled(text_styles.caption)
                    .bg(colors.element_background)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |el| el.bg(colors.element_hover))
                    .child(selected.clone())
                    .on_mouse_up(
                        MouseButton::Left,
                        window.listener_for(
                            &entity_for_trigger,
                            |this: &mut Dropdown, _, _, cx| {
                                this.is_open = !this.is_open;
                                cx.notify();
                            },
                        ),
                    ),
            )
            .when_some(menu, gpui::ParentElement::child)
            .when_some(click_outside, gpui::ParentElement::child)
            .child(
                // ── Canvas for trigger bounds capture ──
                canvas(
                    move |bounds, _window, cx| {
                        entity_for_bounds.update(cx, |d, _| {
                            d.trigger_bounds = Some(bounds);
                        });
                    },
                    |_, (), _, _| {},
                )
                .absolute()
                .size_full(),
            )
    }
}

// ── Render (Entity → RenderOnce) ──────────────────────────────────────────

impl Render for Dropdown {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        DropdownComponent::new(cx.entity())
    }
}
