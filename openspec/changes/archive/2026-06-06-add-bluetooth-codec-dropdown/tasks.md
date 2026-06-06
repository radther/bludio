## 1. Dependency & Build Setup

- [x] 1.1 Enable `pa_v15` feature on `libpulse-binding` in `Cargo.toml`
- [x] 1.2 Add `serde_json` to `Cargo.toml` dependencies for JSON parsing of codec responses
- [x] 1.3 Verify project compiles after dependency changes

## 2. Data Model & Types

- [x] 2.1 Add `CodecInfo` struct with `name: String` and `description: String` to `src/audio/mod.rs`
- [x] 2.2 Add `active_codec: Option<String>` and `codecs: Vec<CodecInfo>` fields to `CardInfo` in `src/audio/mod.rs`
- [x] 2.3 Add `AudioCommand::SetCardCodec(u32, String)` variant to `src/audio/mod.rs`

## 3. PulseAudio Backend — Codec Discovery

- [x] 3.1 Implement `list_card_codecs` helper in `src/audio/pulse.rs` using `send_message_to_object` to send `list-codecs` to `/card/<name>/bluez`
- [x] 3.2 Implement `get_active_codec` helper in `src/audio/pulse.rs` using `send_message_to_object` to send `get-codec` to `/card/<name>/bluez`
- [x] 3.3 Parse JSON array response for `list-codecs` into `Vec<CodecInfo>` using `serde_json`
- [x] 3.4 Parse JSON string response for `get-codec` into `Option<String>`
- [x] 3.5 Integrate codec discovery into `build_audio_state` — call helpers for cards whose name starts with `bluez_`
- [x] 3.6 Handle `send_message_to_object` failures gracefully (empty codec list, no UI shown)

## 4. PulseAudio Backend — Codec Switching

- [x] 4.1 Add `execute_command` arm for `AudioCommand::SetCardCodec` in `src/audio/pulse.rs`
- [x] 4.2 Send `switch-codec` message to `/card/<name>/bluez` with JSON-quoted codec name as parameter
- [x] 4.3 Trigger PA state rebuild after codec switch completes

## 5. UI — Card Row Update

- [x] 5.1 Add `codec_dropdown: Entity<Dropdown>` field to `CardRow` in `src/ui/audio/card_row.rs`
- [x] 5.2 Create codec dropdown in `CardRow::new` when `card.codecs` is non-empty
- [x] 5.3 Subscribe to codec dropdown events and emit `AudioCommand::SetCardCodec` + wakeup on selection
- [x] 5.4 Update `CardRow::update_from_card` to re-sync codec dropdown items and active selection on every state refresh (external codec changes must be reflected)
- [x] 5.5 Change card row render layout from `v_flex()` to `h_flex()` for profile + codec side-by-side
- [x] 5.6 Conditionally render codec dropdown only when codecs are non-empty (keep profile always visible)

## 6. Integration & Verification

- [x] 6.1 Wire `ConfigurationPage` to pass full `CardInfo` (including codec fields) through to `CardRow::new` and `update_from_card`
- [x] 6.2 Run `cargo build` and fix any compilation errors
- [x] 6.3 Run `cargo clippy` and fix warnings
- [x] 6.4 Run `cargo fmt`
- [x] 6.5 Verify profile dropdown still works for non-Bluetooth cards
- [x] 6.6 Verify codec dropdown appears and functions for Bluetooth cards (if test hardware available)
