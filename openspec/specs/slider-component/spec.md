# slider-component

## Purpose

A reusable, interactive horizontal slider component built as a gpui Entity. Renders as two overlapping rounded rectangles (background track + bordered fill) with click-to-jump and click-and-hold drag support, configurable range/step/colors, and EventEmitter-based value change communication.

## Requirements

### Requirement: Slider renders as two overlapping rounded rectangles

The slider SHALL render as a `SliderState` Entity (holds value/range/step/bounds state) paired with a `Slider` RenderOnce element (visual representation). The visual track consists of two overlapping rounded rectangles: a background rectangle (unfilled portion) and a filled rectangle on top with a visible border. The filled rectangle width SHALL be proportional to the current value within the configured range.

#### Scenario: Slider at 0% shows no fill

- **WHEN** the slider value equals the minimum value (0.0)
- **THEN** the filled rectangle SHALL have zero width
- **THEN** only the background track rectangle SHALL be visible

#### Scenario: Slider at 100% shows full fill

- **WHEN** the slider value equals the maximum value (1.0)
- **THEN** the filled rectangle SHALL span the full width of the track
- **THEN** the filled rectangle's border SHALL be visible along its full perimeter

#### Scenario: Slider at 50% shows half fill

- **WHEN** the slider value equals 0.5
- **THEN** the filled rectangle SHALL span exactly 50% of the track width

### Requirement: Click-to-jump sets value at click position

Clicking anywhere on the slider track SHALL immediately set the slider value to the position corresponding to the click's horizontal coordinate, mapped to the configured min/max range.

#### Scenario: Click at 25% position

- **WHEN** the user clicks on the slider at a position 25% from the left edge
- **THEN** the slider value SHALL be set to 25% of the configured range (e.g., 0.25 for range 0.0–1.0)
- **THEN** a `SliderEvent::Change` event SHALL be emitted with the new value
- **THEN** the filled rectangle SHALL update to 25% width

#### Scenario: Click at 0% position (left edge)

- **WHEN** the user clicks at the leftmost edge of the slider
- **THEN** the slider value SHALL be set to the configured minimum

#### Scenario: Click at 100% position (right edge)

- **WHEN** the user clicks at the rightmost edge of the slider
- **THEN** the slider value SHALL be set to the configured maximum

### Requirement: Click-and-hold drag updates value continuously

Pressing the left mouse button on the slider and holding while moving the mouse SHALL continuously update the slider value based on the cursor's horizontal position. The drag SHALL continue to work when the cursor moves outside the slider's vertical bounds (Y is ignored). The value SHALL be clamped to the configured min/max range regardless of cursor position.

#### Scenario: Drag from 50% to 75%

- **WHEN** the user presses and holds left mouse button at 50% position
- **AND** drags the cursor to 75% position
- **THEN** the slider value SHALL update continuously as the cursor moves
- **THEN** `SliderEvent::Change` events SHALL be emitted for each value update

#### Scenario: Drag cursor goes beyond right edge of slider

- **WHEN** the user drags the cursor to the right of the slider bounds
- **THEN** the value SHALL be clamped to the configured maximum
- **THEN** the filled rectangle SHALL show 100% fill

#### Scenario: Drag cursor goes beyond left edge of slider

- **WHEN** the user drags the cursor to the left of the slider bounds
- **THEN** the value SHALL be clamped to the configured minimum
- **THEN** the filled rectangle SHALL show zero fill

#### Scenario: Drag with cursor above or below the slider

- **WHEN** the user drags the cursor above or below the slider's vertical bounds (Y outside the track)
- **THEN** the slider value SHALL continue to follow the cursor's X coordinate
- **THEN** the Y coordinate SHALL be ignored

#### Scenario: Release emits Release event

- **WHEN** the user releases the left mouse button after dragging
- **THEN** a `SliderEvent::Release` event SHALL be emitted with the final value

### Requirement: Slider captures bounds via canvas element

The slider SHALL render a `canvas` element that captures the slider's pixel bounds on layout. These bounds SHALL be stored in `SliderState` and used to map mouse click/drag positions to values.

#### Scenario: Bounds are captured on render

- **WHEN** the slider element is laid out by GPUI
- **THEN** the `canvas` callback SHALL fire with the element's `Bounds<Pixels>`
- **THEN** the bounds SHALL be stored in `SliderState.bounds`
- **THEN** subsequent click-to-jump and drag operations SHALL use these bounds for position mapping

### Requirement: Value step quantization

When a step size is configured, the slider value SHALL be quantized to the nearest multiple of the step. Values from click-to-jump and drag SHALL both respect the step.

#### Scenario: Step of 0.05 quantizes value

- **WHEN** the slider is configured with step 0.05 and range 0.0–1.0
- **AND** the user clicks at a position that maps to 0.37
- **THEN** the emitted value SHALL be 0.35 (nearest multiple of 0.05)

#### Scenario: Step of 1.0 for integer range

- **WHEN** the slider is configured with step 1.0 and range 0.0–100.0
- **AND** the user positions the slider to 42.3
- **THEN** the emitted value SHALL be 42.0

### Requirement: Slider is configurable via constructor and methods

The `SliderState` SHALL be created with `SliderState::new(cx)` which defaults to range 0.0–1.0, value 0.0, and continuous mode (step 0.0). The value SHALL be set after construction via `set_value(value, cx)`. The `Slider` element SHALL accept `.fill_color(Hsla)` to override the default fill color (audio accent from theme).

#### Scenario: Default configuration

- **WHEN** a `SliderState` is created with `SliderState::new(cx)`
- **THEN** the value range SHALL be 0.0 to 1.0
- **THEN** the default value SHALL be 0.0
- **THEN** values SHALL be continuous (no step quantization)

#### Scenario: Custom fill color

- **WHEN** a `Slider` element is created with `.fill_color(custom_color)`
- **THEN** the filled rectangle and border SHALL use the custom color
- **THEN** the track background SHALL still use the theme's `element_background` color

#### Scenario: Default fill color from theme

- **WHEN** a `Slider` element is created without calling `.fill_color()`
- **THEN** the filled rectangle SHALL use the theme's `audio_accent` color

### Requirement: Slider communicates via EventEmitter

The `SliderState` Entity SHALL implement `EventEmitter<SliderEvent>` and emit `SliderEvent::Change(f64)` on every value update and `SliderEvent::Release(f64)` when the user releases the mouse button. The `Slider` element is a `RenderOnce` wrapper that reads from the `SliderState` entity and renders the visual track.

#### Scenario: Parent subscribes to slider events

- **WHEN** a parent view subscribes to the `SliderState` entity via `cx.subscribe(&slider, ...)`
- **THEN** the parent SHALL receive `SliderEvent::Change` events as the user drags or clicks
- **THEN** the parent SHALL receive `SliderEvent::Release` events when the user releases the mouse
