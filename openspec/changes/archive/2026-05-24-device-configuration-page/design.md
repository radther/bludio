## Context

Bludio currently has two audio pages (Output Devices, Input Devices) implemented as `Entity<AudioPage>` with `Vec<Entity<AudioDeviceRow>>`. These rows show sinks (outputs) and sources (inputs) with volume controls, mute, default buttons, and for output devices, a profile dropdown. However, the profile dropdown only appears on rows where the card has an active sink — cards without any active sink (the "Off" profile case) are invisible.

`AudioState` already contains a `cards: Vec<CardInfo>` field with profile info. The PA backend (`pulse.rs`) already subscribes to card events and lists all cards. `AudioCommand::SetCardProfile` already exists in the command channel. What's missing is a UI page that displays cards directly.

The pavucontrol reference (`cardwidget.cc`/`cardwidget.h`) shows a per-card widget with: name label, icon, profile dropdown (sorted by priority), and an optional codec selector. For Bludio's first pass, we'll do the name + profile dropdown — matching the existing `AudioDeviceRow` pattern but simpler (no volume controls).

## Goals / Non-Goals

**Goals:**
- Add a "Configuration" page accessible via a new tab in the left tab bar
- Display all PulseAudio cards from `AudioState.cards`, each with its name and a profile dropdown
- Allow profile switching for any card via the existing `SetCardProfile` command
- Follow all existing architectural patterns: Entity ownership, `cx.subscribe_in()`, `EventEmitter`, Tokio bridge, PA wakeup
- Minor additions to `CardInfo` (new `description` field) and `pulse.rs` (proplist read for `device.description`)
- Cards persist in the UI even when their active profile is "off"

**Non-Goals:**
- Codec selector (like pavucontrol's `codecList`) — this requires `HAVE_PULSE_MESSAGING_API` which Bludio's PA binding may not support
- Port info display — adds complexity without clear user benefit at this stage
- Profile lock toggle
- Hiding unavailable profiles toggle
- Per-card icon resolution (PA proplist icon lookup)
- Filtering or sorting beyond what PA provides

## Decisions

### Decided: New `CardRow` entity with only a Dropdown (no volume controls)

The `AudioDeviceRow` entity bundles volume slider, text field, mute button, default button, and profile dropdown. Cards don't have volumes — they're hardware containers. A dedicated `CardRow` entity avoids unnecessary state and is cleaner than conditionally hiding half the controls.

**Alternative considered**: Reuse `AudioDeviceRow` with hidden volume controls. Rejected — the entity would carry dead state (text field, slider subscriptions), and the render code would have many `when(false, ...)` guards. A separate entity is more maintainable.

### Decided: New `ConfigurationPage` entity (not reusing `AudioPage`)

`AudioPage` syncs rows against `state.sinks` or `state.sources`. `ConfigurationPage` syncs against `state.cards`. The sync logic and row types are different enough that reuse would require significant conditional branching. A separate page entity matches the existing pattern where distinct concerns get their own entities.

**Alternative considered**: Adding a third `DeviceKind` variant and teaching `AudioPage` to handle cards. Rejected — cards have different data shape (no volume, no mute, no default flag), different row type, and different sync logic. The conditional complexity would violate single-responsibility.

### Decided: Reuse existing Dropdown component for profile selection

The `Dropdown` entity already handles trigger rendering, floating menu with anchored positioning, keyboard navigation (up/down/enter/escape), click-outside-to-close, and `EventEmitter<DropdownEvent>`. No changes needed.

**Alternative considered**: Custom inline select. Rejected — the Dropdown is battle-tested, already used in `AudioDeviceRow`, and provides identical UX to what the profile selector needs.

### Decided: New tab between Input Devices and Dev Test

Current tab order: Bluetooth → Output Devices → Input Devices → Dev Test. The Configuration page fits naturally after Input Devices since it's audio-related. New order: Bluetooth → Output Devices → Input Devices → **Configuration** → Dev Test.

### Decided: Card display name from proplist `device.description`, falling back to PA card name

`libpulse-binding`'s `CardInfo` struct exposes a `proplist: Proplist` field with a `get_str()` method. We add a `description: Option<String>` field to our `CardInfo` type, populated in `pulse.rs` from `ci.proplist.get_str(properties::DEVICE_DESCRIPTION)`. `CardRow` displays `description` if present, otherwise falls back to the PA card name (e.g., `alsa_card.pci-0000_00_1f.3`).

This matches pavucontrol's `updateCard()`:
```c
description = pa_proplist_gets(info.proplist, PA_PROP_DEVICE_DESCRIPTION);
w->name = description ? description : info.name;
```

**Alternative considered**: Using raw PA card names. Rejected — the `device.description` property is widely supported (set by ALSA/udev/PipeWire) and produces human-readable names like "Built-in Audio" instead of `alsa_card.pci-0000_00_1f.3`.

### Decided: `cx.subscribe_in()` for child→parent communication (matching existing pattern)

`CardRow` emits events when profile changes via the dropdown. `ConfigurationPage` subscribes to each row. However, since `CardRow` just needs to send `SetCardProfile` commands directly (it already has `cmd_tx` and `wakeup`), and the page doesn't need to react to profile changes (the PA backend sends a state update which triggers `sync_cards`), the subscription is for future extensibility. The existing `AudioDeviceRow` pattern is followed for consistency even where the immediate need is minimal.

## Risks / Trade-offs

- **[Low] Card description may be missing on some systems**: Not all PA cards set `device.description` in their proplist. Mitigation: `CardRow` falls back to the raw PA card name when `description` is `None`, so the UI is never blank.
- **[Low] Profile dropdown fires even when selecting the already-active profile**: The current `Dropdown` emits on any click, including re-selecting the current item. Mitigation: PA's `set_card_profile_by_index` is idempotent — setting the same profile is a no-op. No special guard needed.
