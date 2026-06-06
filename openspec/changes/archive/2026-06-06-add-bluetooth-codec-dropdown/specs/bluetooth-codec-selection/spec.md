## ADDED Requirements

### Requirement: Bluetooth codec discovery via PulseAudio messaging API

The system SHALL discover available Bluetooth audio codecs and the active codec for each Bluetooth card by using PulseAudio's `send_message_to_object` messaging API. The system SHALL target the card-specific BlueZ message handler at path `/card/<card_name>/bluez`.

#### Scenario: Successful codec list retrieval

- **WHEN** the PA backend thread rebuilds audio state after a card event
- **THEN** for each card whose name starts with "bluez_", the system SHALL send `list-codecs` to `/card/<card_name>/bluez`
- **THEN** the system SHALL parse the JSON array response into a list of `CodecInfo` objects (name + description)
- **THEN** the codec list SHALL be stored in `CardInfo.codecs`

#### Scenario: Successful active codec retrieval

- **WHEN** the PA backend thread rebuilds audio state after a card event
- **THEN** for each Bluetooth card, the system SHALL send `get-codec` to `/card/<card_name>/bluez`
- **THEN** the system SHALL parse the JSON string response into the active codec name
- **THEN** the active codec name SHALL be stored in `CardInfo.active_codec`

#### Scenario: Non-Bluetooth card skips codec discovery

- **WHEN** a card's name does not start with "bluez_"
- **THEN** the system SHALL NOT send codec-related messages for that card
- **THEN** `CardInfo.codecs` SHALL be empty and `CardInfo.active_codec` SHALL be `None`

#### Scenario: Bluetooth card with no codec support

- **WHEN** a Bluetooth card exists but the BlueZ message handler does not respond to `list-codecs`
- **THEN** `CardInfo.codecs` SHALL remain empty
- **THEN** the codec dropdown SHALL NOT be displayed for that card

### Requirement: Bluetooth codec switching via messaging API

The system SHALL allow the user to switch the active Bluetooth codec by sending a `switch-codec` message via the PulseAudio messaging API.

#### Scenario: User selects a different codec

- **WHEN** the user selects a different codec from the codec dropdown on a Bluetooth card row
- **THEN** the system SHALL send `AudioCommand::SetCardCodec` with the card index and selected codec name
- **THEN** the PA thread SHALL send `switch-codec` to `/card/<card_name>/bluez` with the codec name as a JSON-quoted string parameter
- **THEN** the PA wakeup SHALL be triggered to refresh state

#### Scenario: Codec switch updates active codec display

- **WHEN** the codec switch command completes and the PA state is rebuilt
- **THEN** the configuration page SHALL reflect the newly active codec in the dropdown
- **THEN** no user interaction SHALL be required to see the update

### Requirement: Codec data model

The system SHALL represent Bluetooth codecs with a dedicated data type attached to each `CardInfo`.

#### Scenario: CodecInfo structure

- **WHEN** codec data is parsed from the messaging API response
- **THEN** each codec SHALL have a `name` (machine identifier, e.g., "aac") and a `description` (human-readable, e.g., "AAC")
- **THEN** the `CardInfo` struct SHALL contain `active_codec: Option<String>` and `codecs: Vec<CodecInfo>`
