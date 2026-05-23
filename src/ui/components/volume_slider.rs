//! Volume slider: a clickable bar that reports the clicked position as a
//! fraction (0.0–1.0). This is the slider only — text field and mute button
//! are owned by the parent UI that embeds this slider.

use gpui::{
    Bounds, CursorStyle, Element, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    MouseButton, MouseUpEvent, Pixels, Window, div, hsla, prelude::*, px,
};
use std::cell::Cell;
use std::rc::Rc;

const BAR_HEIGHT: f32 = 20.0;
const BAR_WIDTH: f32 = 200.0;

// ── Bounds tracker for volume bar ─────────────────────────────────────────

struct BoundsTracker<E: Element> {
    inner: E,
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}
impl<E: Element> IntoElement for BoundsTracker<E> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl<E: Element> Element for BoundsTracker<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;
    fn id(&self) -> Option<gpui::ElementId> {
        self.inner.id()
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }
    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut gpui::App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector_id, window, cx)
    }
    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        self.bounds.set(Some(bounds));
        self.inner
            .prepaint(id, inspector_id, bounds, request_layout, window, cx)
    }
    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut gpui::App,
    ) {
        self.inner.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            prepaint,
            window,
            cx,
        )
    }
}

// ── Volume bar ─────────────────────────────────────────────────────────────

pub(crate) fn volume_bar(
    volume: f64,
    muted: bool,
    on_click: impl Fn(f64) + 'static,
) -> impl IntoElement {
    let fill_color = if muted {
        hsla(0.0, 0.0, 0.35, 1.0)
    } else {
        hsla(210.0 / 360.0, 0.7, 0.55, 1.0)
    };
    let empty_color = hsla(0.0, 0.0, 0.22, 1.0);
    let fill_w = (volume * BAR_WIDTH as f64).clamp(0.0, BAR_WIDTH as f64) as f32;
    let b: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
    let bc = b.clone();
    BoundsTracker {
        inner: div()
            .flex()
            .flex_row()
            .w(px(BAR_WIDTH))
            .h(px(BAR_HEIGHT))
            .rounded_sm()
            .overflow_hidden()
            .cursor(CursorStyle::PointingHand)
            .child(div().h(px(BAR_HEIGHT)).w(px(fill_w)).bg(fill_color))
            .child(
                div()
                    .h(px(BAR_HEIGHT))
                    .w(px(BAR_WIDTH - fill_w))
                    .bg(empty_color),
            )
            .on_mouse_up(MouseButton::Left, move |e: &MouseUpEvent, _, _| {
                let Some(b) = bc.get() else {
                    return;
                };
                let x = ((e.position.x - b.left()) / px(1.0)).clamp(0.0, BAR_WIDTH);
                on_click((x as f64 / BAR_WIDTH as f64).clamp(0.0, 1.0));
            }),
        bounds: b,
    }
}
