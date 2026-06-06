# Accessibility Options

## Purpose

Global accessibility controls that affect the entire application, including animation suppression for users with vestibular disorders.

## Requirements

### Requirement: Disable Animations toggle is shown in Accessibility section
The system SHALL render a "Disable Animations" toggle in the Accessibility section of the Settings page. The toggle SHALL reflect the current value of the `disable_animations` setting.

#### Scenario: Toggle is visible
- **WHEN** the Settings page renders
- **THEN** an "Accessibility" section header SHALL appear
- **THEN** a "Disable Animations" toggle SHALL appear below the header
- **THEN** the toggle SHALL be on if `disable_animations` is true, off otherwise

### Requirement: Toggling Disable Animations persists and applies globally
The system SHALL emit an event when the "Disable Animations" toggle is clicked. `BludioApp` SHALL handle the event by calling `update_settings()` to mutate `disable_animations`, which persists the setting to disk and triggers a UI re-render.

#### Scenario: User enables Disable Animations
- **WHEN** the user clicks the "Disable Animations" toggle to enable it
- **THEN** `SettingsPage` SHALL emit a disable-animations-changed event with value `true`
- **THEN** `BludioApp` SHALL call `update_settings()` to set `disable_animations = true`
- **THEN** the settings SHALL be persisted to disk
- **THEN** the UI SHALL re-render

#### Scenario: User disables Disable Animations
- **WHEN** the user clicks the "Disable Animations" toggle to disable it
- **THEN** `SettingsPage` SHALL emit a disable-animations-changed event with value `false`
- **THEN** `BludioApp` SHALL call `update_settings()` to set `disable_animations = false`
- **THEN** subsequent page renders SHALL animate normally

### Requirement: Animation layer respects the global disable flag
The system SHALL suppress fade-in animations globally when `disable_animations` is true. The `FadeInAnimationExt::with_fade_in_up` method SHALL check the global flag and, when disabled, return an animation with zero duration so the element appears in its final state immediately.

#### Scenario: Animations disabled
- **WHEN** `disable_animations` is true
- **THEN** any element that calls `with_fade_in_up` SHALL appear immediately without motion or opacity transition
- **THEN** no view code is required to check the flag directly

#### Scenario: Animations enabled
- **WHEN** `disable_animations` is false
- **THEN** `with_fade_in_up` SHALL behave exactly as before, animating opacity and position over 400ms

### Requirement: Disable Animations defaults to false
The system SHALL initialize `disable_animations` to `false` when no saved value exists, ensuring animations are enabled by default for new users and users with legacy settings files.

#### Scenario: New user starts app
- **WHEN** the application starts with no settings file
- **THEN** `disable_animations` SHALL default to `false`
- **THEN** all animations SHALL run normally

#### Scenario: Legacy settings file without disable_animations
- **WHEN** the application starts with a settings file that lacks the `disable_animations` field
- **THEN** `disable_animations` SHALL default to `false`
- **THEN** all animations SHALL run normally
