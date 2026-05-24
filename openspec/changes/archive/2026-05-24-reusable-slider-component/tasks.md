## 1. Create Slider Entity and element

- [x] 1.1 Create `src/ui/components/slider.rs` with `SliderEvent` enum (`Change(f64)`, `Release(f64)`)
- [x] 1.2 Implement `Slider` Entity with fields: `value`, `min`, `max`, `step`, `bounds`, `dragging`, `focus_handle`
- [x] 1.3 Implement `Slider::new(cx)` constructor and `set_value(value, cx)` method
- [x] 1.4 Implement `EventEmitter<SliderEvent>` and `Focusable` for `Slider`
- [x] 1.5 Add value-to-percentage and percentage-to-value helpers with step quantization and min/max clamping

## 2. Implement SliderBar (RenderOnce element)

- [x] 2.1 Define `SliderBar` struct with `#[derive(IntoElement)]`, holding `Entity<Slider>` + style fields (`track_color`, `fill_color`, `border_color`, `min`/`max`/`value`/`step`)
- [x] 2.2 Add builder methods on `SliderBar`: `.min()`, `.max()`, `.value()`, `.step()`, `.track_color()`, `.fill_color()`, `.border_color()`
- [x] 2.3 Implement `RenderOnce for SliderBar`: render div with `track_focus`, two overlapping rounded rectangles (track + bordered fill using absolute positioning)
- [x] 2.4 Add canvas-based bounds capture (absolute, size_full canvas child) storing bounds on the Entity
- [x] 2.5 Wire `on_drag` + `on_drag_move` on the track container using `window.listener_for(&entity, ...)` for value updates
- [x] 2.6 Wire `on_mouse_up` + `on_mouse_up_out` to emit `SliderEvent::Release`
- [x] 2.7 Connect value changes in drag/mousedown handlers to `cx.emit(SliderEvent::Change(...))` and `cx.notify()`

## 3. Implement Render for Slider Entity

- [x] 3.1 Implement `Render for Slider`: construct and return `SliderBar` element reading current state fields
- [x] 3.2 Ensure `.id()` is set on the slider container for interactive element tracking

## 4. Migrate TextField to RenderOnce

- [x] 4.1 Define `TextFieldComponent` struct with `#[derive(IntoElement)]`, holding `Entity<TextField>` + `placeholder`/`filter_char` config
- [x] 4.2 Move text layout, selection/cursor quad computation from `TextFieldElement::prepaint` into `RenderOnce::render` — shape the text line, store it on the Entity for mouse hit-testing
- [x] 4.3 Replace `PaintQuad`-based selection/cursor in `paint()` with `canvas()` elements that call `window.paint_quad()` during paint
- [x] 4.4 Replace `window.handle_input()` call in `paint()` with a `canvas()` element that calls `window.handle_input()` during paint
- [x] 4.5 Replace `Rc<Cell<Option<Bounds>>>` bounds capture with canvas-based capture storing on Entity directly
- [x] 4.6 Rewrite `Render for TextField` to return `TextFieldComponent` instead of manually building a `TextFieldElement`
- [x] 4.7 Remove the `TextFieldElement` struct, its `IntoElement` impl, and the full `Element` trait impl (~250 lines)
- [x] 4.8 Keep `TextField` Entity, `EventEmitter<TextFieldEvent>`, `EntityInputHandler`, `Focusable`, key handlers, blink logic, and all public methods unchanged
- [x] 4.9 Verify IME input, keyboard shortcuts (Enter/Esc/Ctrl+A/C/V/X), mouse selection, copy/paste, cursor blink all work (compiles, manual test pending in task 8.6)

## 5. Migrate Dropdown to RenderOnce

- [x] 5.1 Define `DropdownComponent` struct with `#[derive(IntoElement)]`, holding `Entity<Dropdown>` + style fields (`accent`, `bg`, `hover_bg`, `menu_bg`, `menu_border`)
- [x] 5.2 Build trigger button div and `deferred(anchored(...))` floating menu directly in `RenderOnce::render` (no manual `request_layout` on `AnyElement`s)
- [x] 5.3 Replace `Rc<Cell<Option<Bounds>>>` trigger bounds capture with canvas-based capture on the Entity
- [x] 5.4 Wire `on_mouse_up` on trigger, `on_mouse_up` on menu items, `on_key_down` for keyboard nav, and `window.on_mouse_event()` for click-outside-to-close — all via `window.listener_for()`
- [x] 5.5 Rewrite `Render for Dropdown` to return `DropdownComponent` instead of manually building a `DropdownElement`
- [x] 5.6 Remove the `DropdownElement` struct, its `IntoElement` impl, and the full `Element` trait impl (~170 lines)
- [x] 5.7 Keep `Dropdown` Entity, `EventEmitter<DropdownEvent>`, `Focusable`, all public methods (`set_items`, `selected_text`, `has_items`), and builder API unchanged
- [x] 5.8 Verify trigger toggle, floating menu positioning, item selection, keyboard nav (arrows+enter), Escape to close, click-outside-to-close all work (compiles, manual test pending in task 8.7)

## 6. Integrate Slider into module and AudioDeviceRow

- [x] 6.1 Add `pub mod slider;` to `src/ui/components/mod.rs`
- [x] 6.2 In `AudioDeviceRow`: add `slider: Entity<Slider>` field and `_slider_sub: Subscription`
- [x] 6.3 In `AudioDeviceRow::new_impl`: construct `Slider` Entity with `value(volume)`, subscribe via `cx.subscribe_in()` to handle `SliderEvent::Change` → send `AudioCommand::SetSinkVolume` / `SetSourceVolume` + `wakeup.wake()`
- [x] 6.4 In `AudioDeviceRow::render_controls_row`: replace `volume_bar(...)` call with `self.slider.clone()` Entity child
- [x] 6.5 In `AudioDeviceRow::update_from_sink` / `update_from_source`: sync slider value via `self.slider.update(cx, |s, cx| s.set_value(new_volume, cx))`
- [x] 6.6 In `AudioDeviceRow::render`: add blur-sync logic updating slider fill color (gray when muted, accent when unmuted)

## 7. Remove old volume_slider

- [x] 7.1 Remove `pub mod volume_slider;` from `src/ui/components/mod.rs`
- [x] 7.2 Delete `src/ui/components/volume_slider.rs`

## 8. Build, test, and lint

- [x] 8.1 Run `cargo build` and fix all compilation errors
- [x] 8.2 Run `cargo clippy` and fix all warnings
- [x] 8.3 Run `cargo fmt` for code style compliance
- [x] 8.4 Run `cargo test` to verify existing tests pass
- [x] 8.5 Manual visual test: launch app, verify slider renders and click/drag works
- [x] 8.6 Manual test: verify TextField input, IME, keyboard shortcuts, mouse selection, cursor blink
- [x] 8.7 Manual test: verify Dropdown open/close, item selection, keyboard nav, click-outside
- [x] 8.8 Verify AudioDeviceRow volume control integration end-to-end (both slider and text field stay in sync)
