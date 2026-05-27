## MODIFIED Requirements

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

### Requirement: Slider captures bounds via canvas element

The slider SHALL render a `canvas` element that captures the slider's pixel bounds on layout. These bounds SHALL be stored in `SliderState` and used to map mouse click/drag positions to values.

#### Scenario: Bounds are captured on render

- **WHEN** the slider element is laid out by GPUI
- **THEN** the `canvas` callback SHALL fire with the element's `Bounds<Pixels>`
- **THEN** the bounds SHALL be stored in `SliderState.bounds`
- **THEN** subsequent click-to-jump and drag operations SHALL use these bounds for position mapping

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
