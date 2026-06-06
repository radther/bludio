## Why

The Configuration page currently only shows a profile dropdown for each PulseAudio card. However, for Bluetooth audio devices, the active audio codec (e.g., SBC, AAC, aptX, LDAC) is a distinct and important setting that users expect to see and control — pavucontrol displays it alongside the profile. Adding codec selection improves parity with existing Linux audio tooling and gives users direct control over Bluetooth audio quality vs. latency tradeoffs.

## What Changes

- Add a **codec dropdown** next to the profile dropdown on the Configuration page, visible only for Bluetooth cards that expose codec information.
- Query available Bluetooth codecs and the active codec via PulseAudio's **messaging API** (`send_message_to_object` to `/card/<name>/bluez`).
- Allow users to **switch the active codec** via the same messaging API (`switch-codec`).
- Update the PA backend thread to perform codec discovery and switching asynchronously.
- Add new `AudioState` fields and `AudioCommand` variants for codec data.
- Update `CardRow` UI to render both dropdowns in a horizontal flex layout.
- Enable the `pa_v15` feature on `libpulse-binding` to access the messaging API.

## Capabilities

### New Capabilities
- `bluetooth-codec-selection`: Query, display, and switch Bluetooth audio codecs on the Configuration page via PulseAudio's card messaging API.

### Modified Capabilities
- `card-configuration-page`: Extend card rows to show a codec dropdown in addition to the profile dropdown. Add requirements for codec visibility, switching, and Bluetooth-only conditional display.

## Impact

- `src/audio/pulse.rs` — new messaging API calls (`send_message_to_object`), codec state building, codec command execution.
- `src/audio/mod.rs` — new data types (`CodecInfo`, codec fields on `CardInfo`) and `AudioCommand` variant.
- `src/ui/audio/card_row.rs` — second `Dropdown` entity for codec, horizontal layout for profile + codec.
- `src/ui/audio/configuration_page.rs` — pass codec data through to rows.
- `Cargo.toml` — enable `pa_v15` feature on `libpulse-binding`.
- `openspec/specs/card-configuration-page/spec.md` — delta spec with new codec requirements.
