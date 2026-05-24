# slider-component

## Purpose

A reusable, interactive horizontal slider component built as a gpui Entity. Renders as two overlapping rounded rectangles (background track + bordered fill) with click-to-jump and click-and-hold drag support, configurable range/step/colors, and EventEmitter-based value change communication.

## Requirements

### Requirement: Slider renders as two overlapping rounded rectangles

The Slider SHALL render a horizontal track consisting of two overlapping rectangles with rounded corners: a background rectangle (the empty/unfilled portion of the track) and a filled rectangle on top. The filled rectangle SHALL have a visible border. The width of the filled rectangle SHALL be proportional to the current value within the configured range.

#### Scenario: Slider at 0% shows no fill

- **WHEN** the slider value equals the minimum value (e.g., 0.0 in range 0.0–1.0)
- **THEN** the filled rectangle SHALL have zero width
- **THEN** only the background track rectangle SHALL be visible

#### Scenario: Slider at 100% shows full fill

- **WHEN** the slider value equals the maximum value (e.g., 1.0 in range 0.0–1.0)
- **THEN** the filled rectangle SHALL span the full width of the track
- **THEN** the filled rectangle's border SHALL be visible along its full perimeter

#### Scenario: Slider at 50% shows half fill

- **WHEN** the slider value equals the midpoint of the configured range (e.g., 0.5 in range 0.0–1.0)
- **THEN** the filled rectangle SHALL span exactly 50% of the track width

### Requirement: Click-to-jump sets value at click position

Clicking anywhere on the slider track SHALL immediately set the slider value to the position corresponding to the click's horizontal coordinate, mapped to the configured min/max range.

#### Scenario: Click at 25% position

- **WHEN** the user clicks on the slider at a position 25% from the left edge
- **THEN** the slider value SHALL be set to 25% of the configured range (e.g., 25.0 for range 0.0–100.0)
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

### Requirement: Value step quantization

When a step size is configured, the slider value SHALL be quantized to the nearest multiple of the step. Values from click-to-jump and drag SHALL both respect the step.

#### Scenario: Step of 0.05 quantizes value

- **WHEN** the slider is configured with `step(0.05)` and range 0.0–1.0
- **AND** the user clicks at a position that maps to 0.37
- **THEN** the emitted value SHALL be 0.35 (nearest multiple of 0.05)

#### Scenario: Step of 1.0 for integer range

- **WHEN** the slider is configured with `step(1.0)` and range 0.0–100.0
- **AND** the user positions the slider to 42.3
- **THEN** the emitted value SHALL be 42.0

### Requirement: Slider is configurable via builder API

The Slider component SHALL expose chainable builder methods for configuration: `min(f64)`, `max(f64)`, `value(f64)`, `step(f64)`, `track_color(Hsla)`, `fill_color(Hsla)`, `border_color(Hsla)`. The slider SHALL default to range 0.0–1.0 with step 0.0 (continuous) if not configured.

#### Scenario: Default configuration

- **WHEN** a Slider is created without any builder method calls
- **THEN** the value range SHALL be 0.0 to 1.0
- **THEN** the default value SHALL be 0.0
- **THEN** values SHALL be continuous (no step quantization)

#### Scenario: Custom range and step

- **WHEN** a Slider is created with `.min(10.0).max(50.0).step(5.0).value(25.0)`
- **THEN** the initial value SHALL be 25.0
- **THEN** drag and click values SHALL be quantized to multiples of 5.0
- **THEN** values SHALL be clamped to the range 10.0–50.0

### Requirement: Slider communicates via EventEmitter

The Slider Entity SHALL implement `EventEmitter<SliderEvent>` and emit `SliderEvent::Change(f64)` on every value update and `SliderEvent::Release(f64)` when the user releases the mouse button.

#### Scenario: Parent subscribes to slider events

- **WHEN** a parent view subscribes to the Slider Entity via `cx.subscribe_in(&slider, window, ...)`
- **THEN** the parent SHALL receive `SliderEvent::Change` events as the user drags or clicks
- **THEN** the parent SHALL receive `SliderEvent::Release` events when the user releases the mouse
