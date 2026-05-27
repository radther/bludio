//! Self-contained horizontal slider component.
//!
//! Follows the gpui-component pattern: `SliderState` (Entity) holds the
//! value / range / step / bounds state, while `Slider` (`RenderOnce`) is
//! the visual element. Callers configure the element with `.bg()` /
//! `.text_color()` etc. to override the fill / track colors.
//!
//! Emits `SliderEvent`s via `cx.emit()` (`EventEmitter` pattern).

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

// ── SliderState (Entity) ───────────────────────────────────────────────────

/// Holds all mutable slider state (value, range, step, bounds).
/// Emits `SliderEvent`s via the `EventEmitter` pattern.
/// Colors and visual appearance are configured on the [`Slider`] element.
pub(crate) struct SliderState {
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

impl SliderState {
    /// Create a new slider state with default range 0.0–1.0, value 0.0, continuous mode.
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
    fn update_value_from_position(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
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

impl EventEmitter<SliderEvent> for SliderState {}

impl Focusable for SliderState {
    fn focus_handle(&self, _app: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// ── Slider (RenderOnce element) ────────────────────────────────────────────

/// The visual representation of a slider.
///
/// Renders a horizontal track with a filled portion and supports
/// click-to-jump and drag. Colors default to the current theme but can
/// be overridden via `.fill_color()`.
///
/// Use `#[derive(IntoElement)]` to auto-generate the `IntoElement` impl
/// (wrapping in `Component<Slider>`).
#[derive(IntoElement)]
pub(crate) struct Slider {
    state: Entity<SliderState>,
    fill_color: Option<Hsla>,
}

impl Slider {
    /// Create a new slider element bound to the given [`SliderState`] entity.
    pub fn new(state: &Entity<SliderState>) -> Self {
        Self {
            state: state.clone(),
            fill_color: None,
        }
    }

    /// Override the fill (and border) color. Defaults to the theme accent
    /// if not set.
    pub fn fill_color(mut self, color: Hsla) -> Self {
        self.fill_color = Some(color);
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let entity = self.state.clone();
        let entity_id = entity.entity_id();
        let state = entity.read(cx);
        let fill = state.fill_fraction();
        // fill is in [0.0, 1.0] — lossless cast to f32
        #[allow(clippy::cast_possible_truncation)]
        let fill_w = relative(fill as f32);

        // ── Colors: explicit override or theme default ──
        let colors = &crate::ui::theme::theme(cx).colors;
        let fill_color = self.fill_color.unwrap_or(colors.audio_accent);
        let track_color = colors.element_background;

        // Entity clones for event handlers (each needs ownership for 'static lifetime)
        let entity_mouse = entity.clone();
        let entity_drag = entity.clone();
        let entity_bounds = entity.clone();

        div()
            .id(("slider", entity_id))
            .flex_1()
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&entity_mouse, |slider: &mut SliderState, _, _, cx| {
                    slider.handle_release(cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                window.listener_for(&entity_mouse, |slider: &mut SliderState, _, _, cx| {
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
                            move |slider, e: &MouseDownEvent, _window, cx| {
                                slider.update_value_from_position(e.position, cx);
                            },
                        ),
                    )
                    .on_drag(DragSlider(entity_id), |drag, _offset, _window, cx| {
                        cx.stop_propagation();
                        cx.new(|_| drag.clone())
                    })
                    .on_drag_move(window.listener_for(
                        &entity_drag,
                        move |slider, e: &DragMoveEvent<DragSlider>, _window, cx| match e.drag(cx) {
                            DragSlider(id) => {
                                if *id != entity_id {
                                    return;
                                }
                                slider.update_value_from_position(e.event.position, cx);
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
                            .bg(track_color)
                            .rounded_sm()
                            .overflow_hidden()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(0.0))
                                    .top(px(0.0))
                                    .h(px(20.0))
                                    .w(fill_w)
                                    .bg(fill_color)
                                    .border_1()
                                    .border_color(fill_color)
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
