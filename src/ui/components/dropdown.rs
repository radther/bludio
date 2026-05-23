//! General-purpose dropdown selector entity.
//!
//! Self-contained entity that renders a trigger button inline and a floating
//! options list via `deferred(anchored(...))` — the options float above
//! content instead of pushing it around.
//!
//! Matches the `TextField` pattern: emits `DropdownEvent`s via `cx.emit()`
//! (EventEmitter pattern).

use gpui::{
    Anchor, App, Bounds, Context, CursorStyle, DispatchPhase, Element, Entity, EventEmitter,
    FocusHandle, Focusable, GlobalElementId, Hsla, InspectorElementId, IntoElement, KeyDownEvent,
    LayoutId, MouseButton, MouseUpEvent, Pixels, Render, SharedString, Window, anchored, deferred,
    div, hsla, prelude::*, px,
};
use std::cell::Cell;
use std::rc::Rc;

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
/// (EventEmitter pattern).
pub(crate) struct Dropdown {
    items: Vec<String>,
    selected_index: usize,
    is_open: bool,
    focus_handle: FocusHandle,
    placeholder: SharedString,
    accent: Hsla,
    bg: Hsla,
    hover_bg: Hsla,
    menu_bg: Hsla,
    menu_border: Hsla,
    /// Bounds of the trigger element, captured during prepaint for positioning.
    trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
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
            accent: hsla(210.0 / 360.0, 0.7, 0.55, 1.0),
            bg: hsla(0.0, 0.0, 0.22, 1.0),
            hover_bg: hsla(0.0, 0.0, 0.3, 1.0),
            menu_bg: hsla(0.0, 0.0, 0.16, 1.0),
            menu_border: hsla(0.0, 0.0, 0.3, 1.0),
            trigger_bounds: Rc::new(Cell::new(None)),
        }
    }

    /// Replace the item list and selection. Closes the dropdown if open.
    pub fn set_items(&mut self, items: Vec<String>, selected_index: usize, cx: &mut Context<Self>) {
        self.items = items.clone();
        self.selected_index = selected_index.clamp(0, items.len().saturating_sub(1));
        self.is_open = false;
        cx.notify();
    }

    /// Get the currently selected text (or placeholder if nothing selected).
    pub fn selected_text(&self) -> SharedString {
        self.items
            .get(self.selected_index)
            .cloned()
            .map(SharedString::from)
            .unwrap_or_else(|| self.placeholder.clone())
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

    /// Accent color for the selected item and trigger text.
    pub fn accent(mut self, color: Hsla) -> Self {
        self.accent = color;
        self
    }

    /// Background color for the trigger button.
    pub fn bg(mut self, color: Hsla) -> Self {
        self.bg = color;
        self
    }

    /// Hover background color for the trigger button.
    pub fn hover_bg(mut self, color: Hsla) -> Self {
        self.hover_bg = color;
        self
    }

    /// Background color for the menu.
    pub fn menu_bg(mut self, color: Hsla) -> Self {
        self.menu_bg = color;
        self
    }

    /// Border color for the menu.
    pub fn menu_border(mut self, color: Hsla) -> Self {
        self.menu_border = color;
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

// ── Render ─────────────────────────────────────────────────────────────────

impl Render for Dropdown {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle(cx))
            .on_key_down({
                let entity = cx.weak_entity();
                move |event: &KeyDownEvent, _window, app| {
                    if let Some(this) = entity.upgrade() {
                        let key = event.keystroke.key.clone();
                        this.update(app, |this, cx| match key.as_str() {
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
                        });
                    }
                }
            })
            .child(DropdownElement {
                entity: cx.entity(),
            })
    }
}

// ── Custom Element (captures bounds, renders deferred anchored menu) ───────

struct DropdownElement {
    entity: Entity<Dropdown>,
}

struct DropdownLayoutState {
    trigger: Option<gpui::AnyElement>,
    menu: Option<gpui::AnyElement>,
}

impl IntoElement for DropdownElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for DropdownElement {
    type RequestLayoutState = DropdownLayoutState;
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // Read all state up front, then drop borrows before building elements.
        let (
            is_open,
            selected,
            items,
            selected_idx,
            trigger_pos,
            accent,
            bg,
            hover_bg,
            menu_bg,
            menu_border,
        ) = {
            let d = self.entity.read(cx);
            (
                d.is_open,
                d.selected_text(),
                d.items.clone(),
                d.selected_index,
                d.trigger_bounds.get().map(|b| b.origin),
                d.accent,
                d.bg,
                d.hover_bg,
                d.menu_bg,
                d.menu_border,
            )
        };

        // ── Build trigger element ──
        let entity = self.entity.clone();
        let entity2 = self.entity.clone();
        let mut trigger = div()
            .id("dropdown-trigger")
            .px_2()
            .py_1()
            .rounded_sm()
            .text_xs()
            .bg(bg)
            .cursor(CursorStyle::PointingHand)
            .hover(move |el| el.bg(hover_bg))
            .child(selected.clone())
            .on_mouse_up(MouseButton::Left, {
                let entity = entity.clone();
                move |_: &MouseUpEvent, _window, app| {
                    entity.update(app, |this, cx| {
                        this.is_open = !this.is_open;
                        cx.notify();
                    });
                }
            })
            .into_any_element();
        let _trigger_layout_id = trigger.request_layout(window, cx);

        // ── Build floating menu if open ──
        let mut menu_deferred: Option<gpui::AnyElement> = None;
        let mut menu_deferred_layout_id = None;
        if is_open && !items.is_empty() {
            let menu_element = v_flex()
                .bg(menu_bg)
                .border_1()
                .border_color(menu_border)
                .rounded_md()
                .children(items.iter().enumerate().map(|(i, item)| {
                    let item = item.clone();
                    let is_active = i == selected_idx;
                    let entity = entity2.clone();
                    div()
                        .id(SharedString::from(format!("dropdown-item-{i}")))
                        .px_2()
                        .py_1()
                        .text_xs()
                        .cursor(CursorStyle::PointingHand)
                        .when(is_active, move |el| el.text_color(accent))
                        .hover(|el| el.bg(hsla(0.0, 0.0, 1.0, 0.06)))
                        .child(SharedString::from(item.clone()))
                        .on_mouse_up(MouseButton::Left, {
                            let item = item.clone();
                            let entity = entity.clone();
                            move |_: &MouseUpEvent, _window, app| {
                                entity.update(app, |this, cx| {
                                    this.is_open = false;
                                    this.selected_index = i;
                                    cx.emit(DropdownEvent::Selected(i, item.clone()));
                                    cx.notify();
                                });
                            }
                        })
                }))
                .into_any_element();

            // Float above content using deferred + anchored.
            let mut anchored_menu = anchored()
                .anchor(Anchor::TopLeft)
                .snap_to_window_with_margin(px(8.));
            if let Some(pos) = trigger_pos {
                anchored_menu = anchored_menu.position(pos);
            }
            let anchored_elem = anchored_menu.child(menu_element).into_any_element();
            let mut deferred_elem = deferred(anchored_elem).with_priority(1).into_any_element();
            let lid = deferred_elem.request_layout(window, cx);
            menu_deferred_layout_id = Some(lid);
            menu_deferred = Some(deferred_elem);
        }

        let style = gpui::Style {
            size: gpui::Size {
                width: gpui::Length::Auto,
                height: gpui::Length::Auto,
            },
            ..Default::default()
        };
        let layout_ids: Vec<LayoutId> = [_trigger_layout_id]
            .into_iter()
            .chain(menu_deferred_layout_id)
            .collect();
        let layout_id = window.request_layout(style, layout_ids, cx);

        (
            layout_id,
            DropdownLayoutState {
                trigger: Some(trigger),
                menu: menu_deferred,
            },
        )
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Capture trigger bounds for anchoring the menu.
        let dropdown = self.entity.read(cx);
        dropdown.trigger_bounds.set(Some(bounds));

        if let Some(trigger) = request_layout.trigger.as_mut() {
            trigger.prepaint(window, cx);
        }
        if let Some(menu) = request_layout.menu.as_mut() {
            menu.prepaint(window, cx);
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Close dropdown on click outside.
        let entity = self.entity.clone();
        let is_open = self.entity.read(cx).is_open;
        if is_open {
            window.on_mouse_event({
                let entity = entity.clone();
                move |_: &MouseUpEvent, phase, _window, cx| {
                    if phase == DispatchPhase::Bubble {
                        entity.update(cx, |this, cx| {
                            if this.is_open {
                                this.is_open = false;
                                cx.emit(DropdownEvent::Cancelled);
                                cx.notify();
                            }
                        });
                    }
                }
            });
        }

        if let Some(mut trigger) = request_layout.trigger.take() {
            trigger.paint(window, cx);
        }
        if let Some(mut menu) = request_layout.menu.take() {
            menu.paint(window, cx);
        }
    }
}
