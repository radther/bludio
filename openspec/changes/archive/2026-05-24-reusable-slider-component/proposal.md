## Why

The current `volume_bar` in `src/ui/components/volume_slider.rs` is a temporary implementation: a plain function returning `impl IntoElement` with a `BoundsTracker` hack. It only supports click-to-jump — no drag. It's not an Entity and can't participate in the `cx.emit()/cx.subscribe_in()` pattern.

More broadly, the three interactive components (`TextField`, `Dropdown`, volume bar) use three different patterns: the Entity+custom Element pattern (TextField, Dropdown) requires ~200 lines of hand-written `request_layout`/`prepaint`/`paint` boilerplate per component, and the volume bar is a stateless function. Upstream gpui provides `RenderOnce` + `#[derive(IntoElement)]` — a cleaner component pattern used extensively in both Zed and gpui-component — that eliminates this boilerplate. Migrating all three to `RenderOnce` unifies the codebase on a single, modern pattern and sets the standard for all future components.

## What Changes

- **New `Slider` component** in `src/ui/components/slider.rs`: Stateful Entity + `RenderOnce` element (`SliderBar`), following gpui-component's reference implementation. Two-layer rounded visual (track + bordered fill), click-to-jump, click-and-hold drag with out-of-bounds clamping, configurable range/step/colors, `EventEmitter<SliderEvent>` with `Change`/`Release` events
- **Migrate `TextField` to RenderOnce**: Extract `TextFieldElement`'s custom `Element` trait impl into a `RenderOnce` struct. The `TextField` Entity keeps its state, `EventEmitter`, and `EntityInputHandler` impl; a new render component handles the div tree, mouse/key handlers, and canvas-based text painting
- **Migrate `Dropdown` to RenderOnce**: Same treatment — `Dropdown` Entity keeps state/events, a new render component replaces `DropdownElement`'s custom `Element` impl. Trigger bounds captured via canvas instead of `Rc<Cell<Bounds>>`
- **Replace usage in `AudioDeviceRow`**: Swap `volume_bar()` call with the new `Slider` Entity, wired via `cx.subscribe_in()`
- **Remove old implementation**: Delete `src/ui/components/volume_slider.rs`

## Capabilities

### New Capabilities

- `slider-component`: A reusable, interactive slider component Entity that renders a horizontal track with a filled portion, supports click-to-jump and click-and-hold drag, clamps values to a configurable range, and emits value-changed events via the EventEmitter pattern

### Modified Capabilities

- `audio-volume-control`: The volume bar requirement now delegates to the `Slider` Entity instead of the custom `volume_bar()` function. The spec-level behavior (click sets volume, visual fill, text field sync) is unchanged. A new drag scenario is added.

## Impact

- **New file**: `src/ui/components/slider.rs`
- **Modified files**: `src/ui/components/text_field.rs` (RenderOnce migration), `src/ui/components/dropdown.rs` (RenderOnce migration), `src/ui/components/mod.rs` (add slider module, remove volume_slider module), `src/ui/audio/device_row.rs` (replace `volume_bar()` with Slider Entity)
- **Removed files**: `src/ui/components/volume_slider.rs`
- Entity API surfaces unchanged — `AudioDeviceRow` continues to subscribe to the same `TextFieldEvent`/`DropdownEvent` variants. The `TextField`/`Dropdown` Entity's public methods remain identical.
- No external dependencies introduced — `RenderOnce` and `#[derive(IntoElement)]` are in gpui-unofficial 1.2.7
