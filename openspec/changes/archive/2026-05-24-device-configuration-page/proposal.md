## Why

Bludio's Output Devices page only shows PulseAudio sinks — hardware cards that exist but have no active sinks (e.g., a USB headset with the "Off" profile) are completely invisible. Users cannot see all their hardware audio cards or change a card's profile unless that card happens to have an active sink. This matches pavucontrol's own "Configuration" tab, which displays every card with its profile dropdown including "Off" and all other available profiles. Bringing this to Bludio gives users full visibility into their audio hardware and the ability to enable/disable or reconfigure any card independently of its sink/source state.

## What Changes

- Add a new **Configuration** page (tab) that shows all PulseAudio cards from `AudioState.cards`
- Each card row displays the card description/name and a dropdown to select from all available profiles (including "off"/"Off")
- Selecting a profile sends `SetCardProfile` via the existing audio command channel to the PA backend thread
- The page follows the same Entity-based architecture as `AudioPage`: `Entity<ConfigurationPage>` → `Vec<Entity<CardRow>>`
- Cards persist in the list even when their active profile produces no sinks/sources (the "off" case)
- The new tab appears in the sidebar tab bar between Input Devices and Dev Test

## Capabilities

### New Capabilities

- `card-configuration-page`: A dedicated page showing all PulseAudio hardware cards with their available profiles as a dropdown. Users can select any profile (including "Off") for any card. Reuses the existing `AudioState.cards` data, `AudioCommand::SetCardProfile` command, `Dropdown` component, and PA backend wiring without modification.

### Modified Capabilities

<!-- No existing specs need requirement-level changes. The existing audio-device-management spec's profile dropdown behavior on sink rows is unchanged. -->

## Impact

- **New files**: `src/ui/audio/configuration_page.rs`, `src/ui/audio/card_row.rs`
- **Modified files**: `src/ui/audio/mod.rs` (new modules), `src/app.rs` (new Page variant, new entity field, tab bar update, audio state loop update)
- **No changes** to PA backend (`pulse.rs`), audio types (`mod.rs`), Dropdown component, or slider component
- **No API or dependency changes**
