## Context

The codebase currently uses three different component patterns:

| Component | Pattern | Boilerplate |
|---|---|---|
| `volume_bar()` | Plain function, `BoundsTracker` hack | ~100 lines of ad-hoc element impl |
| `TextField` | Entity + custom `Element` (`TextFieldElement`) | ~250 lines for request_layout/prepaint/paint |
| `Dropdown` | Entity + custom `Element` (`DropdownElement`) | ~170 lines for request_layout/prepaint/paint |

Upstream gpui (and thus gpui-unofficial 1.2.7) provides `RenderOnce` + `#[derive(IntoElement)]` — a component pattern where a struct holding configuration and entity references implements `RenderOnce::render(self, &mut Window, &mut App) -> impl IntoElement`. The `#[derive(IntoElement)]` macro auto-generates a `Component<T>` wrapper that handles all `Element` trait boilerplate. This pattern is used extensively in Zed (e.g., `Specimen`, `CharacterGrid`, `MentionCrease`) and is the foundation of every component in gpui-component (Slider, Button, Checkbox, Dropdown, etc.).

The gpui-component slider (`crates/ui/src/slider.rs`) provides a complete, production-tested reference that demonstrates the pattern for interactive components with drag support, bounds capture, and entity-aware event handlers. We adapt it for Bludio's `Slider` and apply the same pattern to migrate `TextField` and `Dropdown`.

### How RenderOnce works

```rust
// 1. Define a struct holding configuration + entity refs
#[derive(IntoElement)]                         // generates IntoElement → Component<T>
pub struct MyComponent {
    entity: Entity<MyEntity>,
    color: Hsla,
    // ... style fields, no mutable state
}

// 2. Implement RenderOnce
impl RenderOnce for MyComponent {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.entity.read(cx);
        div()
            .child(/* ... */)
            .on_click(window.listener_for(&self.entity, |entity, event, window, cx| {
                entity.update(cx, |e, cx| { /* handle event */ });
            }))
    }
}

// 3. Entity handles Render by returning the component
impl Render for MyEntity {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        MyComponent {
            entity: cx.entity(),
            color: self.color,
        }
    }
}
```

Key differences from the Entity+custom Element pattern:
- `RenderOnce::render()` takes `self` (owned), not `&mut self`. The struct is recreated each frame from the Entity's state — configuration flows one direction.
- No `RequestLayoutState`, `PrepaintState`, or hand-written `Element` trait impl. The derive generates a `Component<T>` bridging the two.
- Event handlers use `window.listener_for(&entity, ...)` instead of `cx.listener(...)`. This works because `RenderOnce::render()` receives `&mut App`, not `&mut Context<Self>`.
- Bounds capture uses a canvas child element (absolutely positioned, `size_full`) rather than `Rc<Cell<Bounds>>` in a custom `prepaint`.

### What RenderOnce cannot do

`RenderOnce::render()` receives `&mut App`, not `&mut Context<Self>`. This means you **cannot** call `cx.emit()`, `cx.notify()`, `cx.focus_handle()`, or `cx.subscribe()` directly in `render()`. All of these belong on the Entity — which continues to implement `Render` (returning the component struct), `EventEmitter`, and `Focusable`. The render function only builds the element tree from the Entity's current state.

## Goals / Non-Goals

**Goals:**

*Slider component:*
- A reusable `Slider` Entity + `SliderBar` RenderOnce component in `src/ui/components/slider.rs`
- Two-layer visual: background track rectangle + filled rectangle on top with a visible border, both rounded
- Click-to-jump and click-and-hold horizontal drag with out-of-bounds clamping (X mapped to range, Y ignored)
- Configurable range (min/max, default 0.0–1.0), step size, and colors via builder methods on `SliderBar`
- `EventEmitter<SliderEvent>` with `Change(f64)` (continuous) and `Release(f64)` (on mouse up)
- Replaces `volume_bar()` in `AudioDeviceRow`

*TextField migration:*
- Extract `TextFieldElement`'s rendering into a `RenderOnce` struct while keeping the `TextField` Entity's state, `EventEmitter<TextFieldEvent>`, `EntityInputHandler`, and `Focusable` impls intact
- Replace custom `Element` trait impl (~250 lines) with `#[derive(IntoElement)]` + canvas-based text/selection/cursor painting
- No behavior change: keyboard input, IME, mouse selection, copy/paste, cursor blink all work identically

*Dropdown migration:*
- Extract `DropdownElement`'s rendering into a `RenderOnce` struct while keeping the `Dropdown` Entity's state, `EventEmitter<DropdownEvent>`, and `Focusable` impls
- Replace custom `Element` trait impl (~170 lines) and `Rc<Cell<Bounds>>` trigger bounds hack with `#[derive(IntoElement)]` + canvas bounds capture
- No behavior change: trigger toggle, floating anchored menu, keyboard nav, click-outside-to-close all work identically

**Non-Goals:**
- Vertical slider orientation (horizontal only)
- Range/multi-thumb slider (single value only)
- Keyboard control for slider
- Logarithmic scale
- Changing TextField or Dropdown behavior — migration is purely structural
- Changing public API of any component Entity unless required as signitures may change slightly

## Decisions

### Decision 1: RenderOnce for all three components

**Chosen**: Migrate `TextField`, `Dropdown`, and the new `Slider` to the `RenderOnce` + `#[derive(IntoElement)]` pattern in a single change.

**Rationale**: Unifies the codebase on one component pattern. Eliminates ~420 lines of hand-written Element trait boilerplate across TextField and Dropdown. Sets the standard for all future components. `RenderOnce` is the idiomatic gpui pattern — it's how Zed and gpui-component build components.

**Alternative considered**: Only use RenderOnce for the Slider, migrate TextField/Dropdown later. Rejected — the user wants all components on the same level now. Doing it in one change avoids a period where the codebase has three different patterns.

### Decision 2: Entity stays as state/logic owner, rendering moves to RenderOnce struct

**Chosen**: The Entity continues to hold all mutable state, implement `EventEmitter`, `Focusable`, and (for TextField) `EntityInputHandler`. The Entity's `Render` impl returns a RenderOnce struct. The RenderOnce struct holds an `Entity<T>` clone and any style configuration, reads state in `render()`, and wires event handlers via `window.listener_for()`.

**Rationale**: This is the separation demonstrated by gpui-component's `SliderState` + `Slider`. The Entity remains the single source of truth; the RenderOnce struct is a pure function of the Entity's state. Event handling still flows through `cx.emit()` on the Entity, preserving the existing `cx.subscribe_in()` contracts with parent views.

**Alternative considered**: Move event handling into the RenderOnce struct. Rejected — `RenderOnce::render()` receives `&mut App`, not `&mut Context<Self>`, so `cx.emit()` is unavailable. The Entity must handle events to emit.

### Decision 3: Canvas-based bounds capture replaces Rc<Cell<Bounds>>

**Chosen**: Use an absolutely-positioned, zero-size canvas child element to capture bounds in all three components. The canvas's prepaint callback stores bounds directly on the Entity via `entity.update(cx, ...)`.

```rust
.child(
    canvas(
        move |bounds, _window, cx| {
            entity.update(cx, |this, _cx| this.bounds = bounds);
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full()
)
```

**Rationale**: Simpler than the `Rc<Cell<Option<Bounds>>>` pattern used in Dropdown — no shared ownership, no `Option` unwrapping. Proven in gpui-component's slider. The canvas is invisible (no paint callback) and only serves as a bounds hook.

**Alternative considered**: Keep `Rc<Cell<Bounds>>`. Rejected as it requires manual lifecycle management and is more verbose.

### Decision 4: Drag via gpui's on_drag / on_drag_move (Slider only)

**Chosen**: Use gpui's built-in `on_drag()` to start drag tracking and `on_drag_move()` to handle continuous position updates. These natively handle the full lifecycle including out-of-bounds tracking.

**Rationale**: gpui's purpose-built drag API handles mousedown→mousemove→mouseup (including cursor leaving bounds) natively. The gpui-component slider validates this works correctly.

### Decision 5: Dual events — Change + Release (Slider only)

**Chosen**: `SliderEvent::Change(f64)` on every value update during drag, `SliderEvent::Release(f64)` on mouse up.

**Rationale**: Follows gpui-component's event model. `Change` provides responsive real-time updates. `Release` allows consumers to batch expensive operations. `AudioDeviceRow` uses `Change` for immediate volume feedback.

### Decision 6: Builder API on RenderOnce structs

**Chosen**: Configuration through chainable methods on the RenderOnce struct: `SliderBar::track_color()`, `TextFieldComponent::placeholder()`, etc. The Entity's `Render` impl reads its fields and passes them to the struct's builder.

**Rationale**: Consistent with gpui-component's API design. The RenderOnce struct is the natural place for visual configuration.

## TextField migration specifics

The `TextFieldElement` currently handles three responsibilities that move into `RenderOnce::render()`:

1. **Text layout**: `window.text_system().shape_line()` → `ShapedLine` — done in render, stored temporarily on the Entity for `index_for_mouse_position()`
2. **Selection/cursor painting**: `PaintQuad` via `window.paint_quad()` → replaced with `canvas()` elements that paint the selection highlight and cursor quad. The canvas receives bounds during paint and has full `window: &mut Window` access for `paint_quad()`
3. **Input handling**: `window.handle_input()` → continues via a canvas element in the render tree that calls `window.handle_input()` during paint. The `EntityInputHandler` impl on `TextField` Entity is unchanged

The key change: instead of `TextFieldElement::prepaint()` computing `ShapedLine` + `PaintQuad`s into `TextFieldPrepaint` and `paint()` emitting them, the `RenderOnce::render()` method:
- Shapes the text line
- Puts the line + computed quads into `Entity<TextField>` (for mouse hit-testing)
- Wraps selection/cursor painting in `canvas()` elements
- Wraps `window.handle_input()` in a `canvas()` element

## Dropdown migration specifics

The `DropdownElement` currently handles:

1. **Trigger rendering**: A styled div with text and click handler → same div, just built in `RenderOnce::render()` instead of `request_layout`
2. **Floating menu**: `deferred(anchored(...))` built as `AnyElement` in `request_layout` → same, built in `RenderOnce::render()`
3. **Trigger bounds capture**: `Rc<Cell<Option<Bounds>>>` in `prepaint` → canvas-based capture in `RenderOnce::render()`
4. **Click-outside-to-close**: `window.on_mouse_event()` in `paint` → same call, made from the render function

The `RenderOnce` approach is simpler because the deferred anchored menu is just a child element — no need to manually call `request_layout` on it and manage layout IDs.

## Risks / Trade-offs

- **Ctrl+modifier mismatch on TextField**: During migration, ensure `shortcut_mod` detection (Ctrl vs Cmd per platform) is preserved. This is a copy of existing logic, low risk.
- **Cursor blink timing**: The cursor blink uses `cx.spawn()` with timers. In `RenderOnce`, the blink state is read from the Entity — timers are still spawned on the Entity's context. No change needed.
- **IME input handling**: `window.handle_input()` from within a canvas element in the render tree must work identically to the current `paint()`-based approach. Verified in gpui-component patterns — canvas gets full `&mut Window` during paint.
- **Event flood during drag**: `SliderEvent::Change` emits on every mousemove pixel. Mitigated at the consumer — `AudioDeviceRow` sends volume commands synchronously; PulseAudio handles rate-limiting.
- **RenderOnce composability**: If a parent wraps a `RenderOnce` child that also contains `window.listener_for()` handlers, ensure no event conflicts. No known issues — this is standard gpui usage.
