## ADDED Requirements

### Requirement: Error banner displays transient error messages

The system SHALL provide a shared `error_banner` function in `src/ui/components/error_banner.rs` that returns `gpui::Div` and renders an animated bottom banner for displaying error messages. The banner SHALL slide up from below the viewport when shown and slide down when dismissed. The component SHALL be usable by any page that manages its own error state.

#### Scenario: Banner appears with error message
- **WHEN** `error_banner` is called with a text string and generation counter
- **THEN** the banner SHALL render at the bottom of its parent container
- **THEN** the banner SHALL animate upward from off-screen over ~300ms using `with_animation`
- **THEN** the banner SHALL display the error text in the primary text color
- **THEN** the banner SHALL have the same background color as the tab bar

#### Scenario: Banner dismissed by user click
- **WHEN** the user clicks anywhere on the banner
- **THEN** the on_dismiss callback SHALL be invoked
- **THEN** the calling page SHALL set its error text to `None`

#### Scenario: New error replaces current banner
- **WHEN** a new error arrives while the banner is visible (generation changes)
- **THEN** the banner SHALL update the displayed text immediately
- **THEN** the animation SHALL restart with the new generation key

### Requirement: Error banner styling matches tab bar aesthetic

The banner SHALL use the same background color as the tab bar and span the full width of its parent container.

#### Scenario: Banner visual consistency
- **WHEN** the banner is rendered
- **THEN** the background color SHALL match `colors.tab_bar_background` (or equivalent)
- **THEN** the banner SHALL span the full width of its parent
- **THEN** the banner SHALL have horizontal padding matching the page header
- **THEN** the error text SHALL use the caption text style
