## Context

The Bludio Configuration page (`src/ui/audio/configuration_page.rs`) lists all PulseAudio cards via `CardRow` entities. Each row currently shows a display name and a single profile `Dropdown`. pavucontrol displays both a profile dropdown and a codec dropdown for Bluetooth cards. The codec is distinct from the profile: a profile (e.g., `a2dp-sink`) defines the card's overall role, while a codec (e.g., `aac`, `ldac`) defines the specific audio encoding used within that role. Codecs are only relevant for Bluetooth cards and are exposed by PipeWire/PulseAudio's BlueZ module through the PulseAudio messaging API, not through standard card introspection.

## Goals / Non-Goals

**Goals:**
- Display a codec dropdown alongside the profile dropdown for Bluetooth cards on the Configuration page.
- Query available/active codecs via PulseAudio's `send_message_to_object` API.
- Allow users to switch codecs via the same messaging API.
- Keep the UI conditional — only show the codec dropdown when codecs are available.
- Follow existing Bludio patterns: entity ownership, event subscription, `AudioCommand` dispatch.

**Non-Goals:**
- Showing codec information on the Output/Input device pages (only on Configuration).
- Supporting non-Bluetooth codec sources (e.g., USB sound card DSP modes).
- Persistent codec preferences across reconnects.
- Codecs on sink/source rows — this is card-level only, matching pavucontrol.

## Decisions

### Use `libpulse-binding`'s `send_message_to_object` (requires `pa_v15`)

**Rationale:** pavucontrol uses `pa_context_send_message_to_object` to communicate with the BlueZ module's message handler at `/card/<name>/bluez`. This is the only documented way to query and switch Bluetooth codecs in PipeWire-Pulse. The Rust binding exposes this under the `pa_v15` feature.

**Alternative considered:** Parsing D-Bus directly via BlueZ. Rejected because PipeWire manages codec selection internally; the PA messaging API is the canonical interface.

### Discovery: send `list-codecs` and `get-codec` per Bluetooth card during state build

**Rationale:** pavucontrol initiates codec discovery inside the card callback by first listing message handlers on `/core`, then sending targeted requests. We can simplify by sending `list-codecs` and `get-codec` directly for every card whose name starts with `bluez_`. The `send_message_to_object` callback tells us via `success` whether the handler exists.

**Trade-off:** Slightly more messages on non-BlueZ systems, but eliminates the extra `list-handlers` round-trip. The PA thread is event-driven and these are cheap.

### Data model: add `active_codec` and `codecs` to `CardInfo`

**Rationale:** Codecs are card-level metadata, just like profiles. Adding them to `CardInfo` keeps the existing `AudioState → CardRow` data flow intact without new channels or special-case passing.

**Alternative considered:** A separate `HashMap<u32, CodecData>` in `AudioState`. Rejected because it complicates row syncing — `CardRow` already receives the full `CardInfo`.

### UI layout: horizontal `h_flex()` with profile left, codec right

**Rationale:** pavucontrol places both selectors on the same row. Using `h_flex()` matches GPUI conventions and keeps the card row compact. The codec dropdown is conditionally rendered via `.when(codecs.non_empty())`.

### Command variant: `AudioCommand::SetCardCodec(u32, String, String)`

**Rationale:** Similar to `SetCardProfile`, but requires three fields: `(card_index, card_name, codec_name)`. The PA messaging API targets `/card/<name>/bluez` by the card's string name, not by numeric index, so the card name must be forwarded from the UI to the PA thread. The PA thread translates this into `send_message_to_object("/card/<name>/bluez", "switch-codec", "\"<codec>\"")`. The codec name must be JSON-quoted because PipeWire-Pulse expects a JSON string parameter.

## Risks / Trade-offs

- **[Risk]** `pa_v15` feature may not be available on older `libpulse-binding` versions. **Mitigation:** The project currently uses `2.30`, which supports `pa_v15`. We will update `Cargo.toml` to enable it.
- **[Risk]** `send_message_to_object` may fail silently if the PipeWire version does not expose the BlueZ message handler. **Mitigation:** The callback receives a `success` boolean; on failure, `CardInfo.codecs` remains empty and the UI gracefully skips the codec dropdown.
- **[Risk]** JSON parsing of codec responses adds a new dependency (`serde_json`). **Mitigation:** `serde` and `serde_json` are commonly already available transitively. If not, adding them is low-cost.
- **[Risk]** Codec state is not included in PA's standard subscription events. **Mitigation:** We already rebuild full state on every PA event; codec discovery piggybacks on that rebuild cycle.
- **[Trade-off]** Synchronous `send_message_to_object` calls block the PA mainloop during state build. Acceptable because these are single-shot messages per Bluetooth card and the PA thread is dedicated.

### Codec reactivity follows the existing profile pattern

The PA thread already subscribes to `Facility::Card` events. Any external codec change (e.g., from another pavucontrol instance) triggers a card subscription event, which sets `needs_rebuild = true`. The next mainloop iteration rebuilds full state, re-queriing `get-codec` for every Bluetooth card. `ConfigurationPage::sync_cards` propagates the new `CardInfo` to each `CardRow`, and `CardRow::update_from_card` updates the codec dropdown's selected index via `Dropdown::set_items`. This is identical to how profile reactivity already works — no new subscription machinery is needed.

## Open Questions

- Should we also implement `get-profile-sticky` / `set-profile-sticky` (profile lock toggle) as pavucontrol does? **Deferred** — not part of this change.
- Should codec descriptions be localized? **Deferred** — PipeWire returns English descriptions; we pass them through as-is.
