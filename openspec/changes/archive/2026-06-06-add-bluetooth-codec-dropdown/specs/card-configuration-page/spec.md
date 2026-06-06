## MODIFIED Requirements

### Requirement: Configuration page lists all audio cards with profiles

The system SHALL provide a "Configuration" page accessible via a tab in the left tab bar that displays all PulseAudio hardware cards. Each card row SHALL show a human-readable name (from the card proplist `device.description` property) and a dropdown selector listing all available profiles for that card. The dropdown SHALL show the currently active profile as the selected item. Cards SHALL be displayed even when their active profile is "off" (produces no sinks or sources).

#### Scenario: Configuration tab shows all cards

- **WHEN** the user navigates to the "Configuration" tab
- **THEN** a scrollable list of all PulseAudio cards from `AudioState.cards` SHALL be displayed
- **THEN** each card row SHALL show a human-readable name (e.g., "Built-in Audio" or "USB Audio Device")
- **THEN** the list SHALL include cards whose active profile is "off"

#### Scenario: Card with no device.description fallback

- **WHEN** a card's proplist does not contain a `device.description` property
- **THEN** the card row SHALL display the raw PA card name as a fallback
- **THEN** the card SHALL still appear in the list with its profile dropdown

#### Scenario: Empty state when no cards available

- **WHEN** PulseAudio reports zero cards
- **THEN** the configuration page SHALL display a message indicating no audio cards found

#### Scenario: Connected but no cards

- **WHEN** PulseAudio is connected but the system has no audio cards (e.g., no sound hardware)
- **THEN** an appropriate empty-state message SHALL be shown
- **THEN** the page SHALL not show an error indicator

### Requirement: Profile switching via dropdown on card rows

The system SHALL display the active card profile on each card row and SHALL allow the user to switch between available profiles via a dropdown selector. Selecting a profile SHALL send `AudioCommand::SetCardProfile` to the PA backend thread.

#### Scenario: Active profile shown on card row

- **WHEN** a card has an active profile
- **THEN** the dropdown on the card row SHALL display the active profile name

#### Scenario: User selects a different profile

- **WHEN** the user clicks the profile dropdown and selects a different profile from the expanded list
- **THEN** the system SHALL send `AudioCommand::SetCardProfile` with the card index and selected profile name
- **THEN** the PA wakeup SHALL be triggered
- **THEN** the dropdown SHALL collapse and display the newly selected profile after state refresh

#### Scenario: Profile dropdown shows all available profiles

- **WHEN** the user clicks the profile dropdown on a card row
- **THEN** the dropdown SHALL expand to list all available profiles from `CardInfo.profiles`
- **THEN** the currently active profile SHALL be visually distinguished in the expanded list

#### Scenario: Card with a single profile

- **WHEN** a card has only one available profile
- **THEN** the dropdown SHALL still be displayed with that single profile as the only option

### Requirement: Configuration page reacts to external changes

The system SHALL subscribe to PulseAudio card events and SHALL refresh the displayed card list when card properties change externally (e.g., profile switched by another app, new card plugged in, card removed).

#### Scenario: External profile change is reflected

- **WHEN** a card's profile is changed externally (e.g., via `pactl set-card-profile`)
- **THEN** the configuration page SHALL update the displayed active profile for that card
- **THEN** no user interaction SHALL be required to see the update

#### Scenario: New card appears after hardware plugged in

- **WHEN** a new audio hardware device is plugged in and registered as a PulseAudio card
- **THEN** the configuration page SHALL show the new card in the list

#### Scenario: Card removed after hardware unplugged

- **WHEN** an audio hardware device is unplugged and its card is removed from PulseAudio
- **THEN** the configuration page SHALL remove that card from the list

### Requirement: Configuration page shows connection state

The system SHALL display connection status information on the configuration page when PulseAudio is not connected.

#### Scenario: PulseAudio not connected

- **WHEN** PulseAudio is not connected (e.g., server not running)
- **THEN** the configuration page SHALL display a "Connecting to PulseAudio..." message
- **THEN** the card list SHALL not be displayed

## ADDED Requirements

### Requirement: Bluetooth codec dropdown on card rows

The system SHALL display a second dropdown on each card row for Bluetooth audio codec selection, positioned horizontally next to the profile dropdown. The codec dropdown SHALL only appear for cards that have non-empty `CardInfo.codecs`.

#### Scenario: Bluetooth card shows both profile and codec dropdowns

- **WHEN** a Bluetooth card has available codecs (`CardInfo.codecs` is non-empty)
- **THEN** the card row SHALL display both a profile dropdown and a codec dropdown in a horizontal layout
- **THEN** the profile dropdown SHALL appear first (left), followed by the codec dropdown (right)

#### Scenario: Non-Bluetooth card shows only profile dropdown

- **WHEN** a card has empty `CardInfo.codecs`
- **THEN** the card row SHALL display only the profile dropdown
- **THEN** no codec-related UI elements SHALL be visible

#### Scenario: Codec dropdown shows active codec

- **WHEN** a Bluetooth card has an active codec (`CardInfo.active_codec` is `Some`)
- **THEN** the codec dropdown SHALL display the active codec's description as the selected item

#### Scenario: User selects a different codec

- **WHEN** the user clicks the codec dropdown and selects a different codec
- **THEN** the system SHALL send `AudioCommand::SetCardCodec` with the card index and selected codec name
- **THEN** the PA wakeup SHALL be triggered
- **THEN** the dropdown SHALL collapse and display the newly selected codec after state refresh

#### Scenario: Codec dropdown lists all available codecs

- **WHEN** the user clicks the codec dropdown on a Bluetooth card row
- **THEN** the dropdown SHALL expand to list all available codecs from `CardInfo.codecs`
- **THEN** the currently active codec SHALL be visually distinguished in the expanded list

#### Scenario: External codec change is reflected

- **WHEN** a Bluetooth card's codec is changed externally (e.g., via another pavucontrol instance)
- **THEN** the PA subscription event SHALL trigger a full state rebuild
- **THEN** the configuration page SHALL update the displayed active codec for that card
- **THEN** no user interaction SHALL be required to see the update
