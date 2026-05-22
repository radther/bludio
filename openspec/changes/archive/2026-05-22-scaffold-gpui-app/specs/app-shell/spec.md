## ADDED Requirements

### Requirement: Application launches and opens a window

The system SHALL compile as a Rust binary that, when executed, opens a native OS window using the GPUI rendering pipeline.

#### Scenario: Binary compiles successfully

- **WHEN** `cargo build` is run in the project root
- **THEN** the build completes without errors

#### Scenario: Application starts and creates a window

- **WHEN** the compiled binary is executed
- **THEN** a native OS window opens with no terminal output errors

### Requirement: Window displays centered "bludio" text

The system SHALL render the lowercase text "bludio" centered horizontally and vertically within the window, in white color on a solid black background.

#### Scenario: Text is visible and centered

- **WHEN** the application window is open
- **THEN** the text "bludio" is displayed in white
- **THEN** the text is centered both horizontally and vertically within the window
- **THEN** the window background is solid black (`#000000`)

### Requirement: Window uses reasonable default dimensions

The system SHALL open the window with dimensions of 1100 pixels wide by 700 pixels tall, centered on the primary display.

#### Scenario: Window opens at specified size

- **WHEN** the application starts
- **THEN** the window is 1100×700 pixels
- **THEN** the window is centered on the display
- **THEN** the window has an app identity string (e.g., `"com.bludio.app"`)
