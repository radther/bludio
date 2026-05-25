//! Self-contained horizontal slider component.
//!
//! Entity-based slider that renders as two overlapping rounded rectangles
//! (background track + bordered fill). Supports click-to-jump and
//! click-and-hold drag with out-of-bounds clamping.
//!
//! Emits `SliderEvent`s via `cx.emit()` (`EventEmitter` pattern).
//! Follows gpui-component's `SliderState` + `Slider` (`RenderOnce`) pattern.
//!
//! Architecture:
//!   Slider (Entity) — holds state, emits events
//!   `SliderBar` (`RenderOnce`) — renders the visual track + fill

use gpui::{
    App, Bounds, Context, DragMoveEvent, Entity, EntityId, EventEmitter, FocusHandle, Focusable,
    Hsla, IntoElement, MouseButton, MouseDownEvent, Pixels, Point, Render, RenderOnce, Window,
    canvas, div, prelude::*, px, relative,
};

use crate::ui::h_flex;

// ── Events ─────────────────────────────────────────────────────────────────

/// Events emitted by the Slider for parent views to handle.
#[derive(Clone, Debug)]
pub(crate) enum SliderEvent {
    /// Emitted continuously while the slider value is being changed by the user.
    Change(f64),
    /// Emitted once when the user releases the slider after a drag or click.
    ///
    /// The value is present for consumers that want debounced/batched updates;
    /// currently unused by `AudioDeviceRow` but part of the public API.
    #[allow(dead_code)]
    Release(f64),
}

// ── Drag marker (for on_drag / on_drag_move) ───────────────────────────────

/// Marker type for `on_drag`. gpui's drag system requires a Render-able Entity
/// as the drag payload. This empty entity serves that role.
#[derive(Clone)]
struct DragSlider(EntityId);

impl Render for DragSlider {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

// ── Entity ─────────────────────────────────────────────────────────────────

/// A self-contained horizontal slider.
///
/// Holds all mutable state (value, range, step, bounds). Emits `SliderEvent`s
/// via the `EventEmitter` pattern. The visual rendering is handled by `SliderBar`
/// (a `RenderOnce` component).
pub(crate) struct Slider {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    /// Captured bounds of the slider element, used to map mouse coordinates to values.
    bounds: Option<Bounds<Pixels>>,
    /// Whether the user is currently dragging (so we only emit `Release` after a real interaction).
    dragging: bool,
    focus_handle: FocusHandle,
}

impl Slider {
    /// Create a new slider with default range 0.0–1.0, value 0.0, continuous mode.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 1.0,
            step: 0.0,
            bounds: None,
            dragging: false,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Set the slider value and notify.
    pub fn set_value(&mut self, value: f64, cx: &mut Context<Self>) {
        self.value = value.clamp(self.min, self.max);
        cx.notify();
    }

    /// Get the current value.
    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Whether the user is currently dragging the slider.
    #[allow(dead_code)]
    pub(crate) fn is_dragging(&self) -> bool {
        self.dragging
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    /// Map a value in [min, max] to a percentage [0.0, 1.0].
    fn value_to_percentage(&self, value: f64) -> f64 {
        let range = self.max - self.min;
        if range <= 0.0 {
            0.0
        } else {
            ((value - self.min) / range).clamp(0.0, 1.0)
        }
    }

    /// Map a percentage [0.0, 1.0] to a value in [min, max], quantized by step.
    fn percentage_to_value(&self, percentage: f64) -> f64 {
        let raw = self.min + (self.max - self.min) * percentage;
        if self.step > 0.0 {
            (raw / self.step).round() * self.step
        } else {
            raw
        }
        .clamp(self.min, self.max)
    }

    /// Compute the fill width as a fraction [0.0, 1.0] of the total width.
    fn fill_fraction(&self) -> f64 {
        self.value_to_percentage(self.value)
    }

    /// Update value based on a mouse position relative to the slider's bounds.
    /// Called during click-to-jump and drag.
    fn update_value_from_position(
        &mut self,
        position: Point<Pixels>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.dragging = true;
        let Some(bounds) = self.bounds else {
            return;
        };
        let inner_x = (position.x - bounds.left()).clamp(px(0.0), bounds.size.width);
        let percentage = inner_x / bounds.size.width;
        self.value = self.percentage_to_value(f64::from(percentage));
        cx.emit(SliderEvent::Change(self.value));
        cx.notify();
    }

    /// Emit `Release` if the user was actively interacting.
    fn handle_release(&mut self, cx: &mut Context<Self>) {
        if !self.dragging {
            return;
        }
        self.dragging = false;
        cx.emit(SliderEvent::Release(self.value));
    }
}

// ── EventEmitter + Focusable ─────────────────────────────────────────────

impl EventEmitter<SliderEvent> for Slider {}

impl Focusable for Slider {
    fn focus_handle(&self, _app: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// ── SliderBar (RenderOnce element) ─────────────────────────────────────────

/// The visual representation of a Slider.
///
/// Renders a horizontal track with a filled portion. The fill has a visible
/// border while the background track does not. Supports click-to-jump and drag.
///
/// Use `#[derive(IntoElement)]` to auto-generate the `IntoElement` impl
/// (wrapping in `Component<SliderBar>`).
#[derive(IntoElement)]
struct SliderBar {
    entity: Entity<Slider>,
    track_color: Hsla,
    fill_color: Hsla,
    border_color: Hsla,
}

impl SliderBar {
    /// Create a new slider bar bound to the given Slider entity.
    fn new(entity: Entity<Slider>, cx: &App) -> Self {
        let colors = &crate::ui::theme::theme(cx).colors;
        Self {
            entity,
            track_color: colors.element_background,
            fill_color: colors.accent,
            border_color: colors.accent,
        }
    }

    /// Background color of the unfilled track.
    #[allow(dead_code)]
    pub fn track_color(mut self, color: Hsla) -> Self {
        self.track_color = color;
        self
    }

    /// Color of the filled portion.
    #[allow(dead_code)]
    pub fn fill_color(mut self, color: Hsla) -> Self {
        self.fill_color = color;
        self
    }

    /// Border color of the filled portion.
    #[allow(dead_code)]
    pub fn border_color(mut self, color: Hsla) -> Self {
        self.border_color = color;
        self
    }
}

impl RenderOnce for SliderBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let entity = self.entity.clone();
        let entity_id = entity.entity_id();
        let fill = entity.read(cx).fill_fraction();
        #[allow(clippy::cast_possible_truncation)]
        let fill_w = relative(fill as f32);

        // Entity clones for event handlers (each needs ownership for 'static lifetime)
        let entity_mouse = entity.clone();
        let entity_drag = entity.clone();
        let entity_bounds = entity.clone();

        div()
            .id(("slider", entity_id))
            .flex_1()
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&entity_mouse, |slider: &mut Slider, _, _, cx| {
                    slider.handle_release(cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                window.listener_for(&entity_mouse, |slider: &mut Slider, _, _, cx| {
                    slider.handle_release(cx);
                }),
            )
            .child(
                h_flex()
                    .id("slider-bar-container")
                    .w_full()
                    .on_mouse_down(
                        MouseButton::Left,
                        window.listener_for(
                            &entity_drag,
                            move |slider, e: &MouseDownEvent, window, cx| {
                                slider.update_value_from_position(e.position, window, cx);
                            },
                        ),
                    )
                    .on_drag(DragSlider(entity_id), |drag, _offset, _window, cx| {
                        cx.stop_propagation();
                        cx.new(|_| drag.clone())
                    })
                    .on_drag_move(window.listener_for(
                        &entity_drag,
                        move |slider, e: &DragMoveEvent<DragSlider>, window, cx| match e.drag(cx) {
                            DragSlider(id) => {
                                if *id != entity_id {
                                    return;
                                }
                                slider.update_value_from_position(e.event.position, window, cx);
                            }
                        },
                    ))
                    .cursor(gpui::CursorStyle::PointingHand)
                    .child(
                        div()
                            .id("slider-bar")
                            .relative()
                            .w_full()
                            .h(px(20.0))
                            .bg(self.track_color)
                            .rounded_sm()
                            .overflow_hidden()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(0.0))
                                    .top(px(0.0))
                                    .h(px(20.0))
                                    .w(fill_w)
                                    .bg(self.fill_color)
                                    .border_1()
                                    .border_color(self.border_color)
                                    .rounded_sm(),
                            )
                            .child(
                                canvas(
                                    move |bounds, _window, cx| {
                                        entity_bounds.update(cx, |slider, _cx| {
                                            slider.bounds = Some(bounds);
                                        });
                                    },
                                    |_, (), _, _| {},
                                )
                                .absolute()
                                .size_full(),
                            ),
                    ),
            )
    }
}

// ── Render (Entity → RenderOnce) ──────────────────────────────────────────

impl Render for Slider {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        SliderBar::new(cx.entity(), cx)
    }
}
