## 1. Setup and dependencies

- [x] 1.1 Add `libpulse-binding` crate to `Cargo.toml` dependencies
- [x] 1.2 Create `src/audio/` module directory with `mod.rs` and `pulse.rs` files
- [x] 1.3 Add `mod audio` declaration to `src/main.rs`
- [x] 1.4 Verify `libpulse-dev` is installed and `cargo build` succeeds with new dependency

## 2. Audio data types and state

- [x] 2.1 Define `AudioState` struct in `src/audio/mod.rs`
- [x] 2.2 Define `SinkInfo` struct
- [x] 2.3 Define `SourceInfo` struct
- [x] 2.4 Define `CardInfo` and `ProfileInfo` structs
- [x] 2.5 Define `AudioCommand` enum
- [x] 2.6 Derive `Clone`, `Debug` on all data types

## 3. PulseAudio backend thread

- [x] 3.1 Implement `start_pulse_audio_thread()`
- [x] 3.2 Implement sink listing
- [x] 3.3 Implement source listing
- [x] 3.4 Implement card listing
- [x] 3.5 Cross-reference sinks/sources to cards
- [x] 3.6 Implement server info fetch
- [x] 3.7 Implement `build_audio_state()` helper

## 4. PA command execution and event subscription

- [x] 4.1 Implement command loop
- [x] 4.2 Implement volume set operations
- [x] 4.3 Implement mute set operations
- [x] 4.4 Implement `set_card_profile_by_index`
- [x] 4.5 Implement `set_default_sink`/`set_default_source`
- [x] 4.6 Subscribe to PA events
- [x] 4.7 Rebuild AudioState on events

## 5. Integration into BludioApp

- [x] 5.1 Add `audio_state: AudioState` field
- [x] 5.2 Create channels, spawn PA thread
- [x] 5.3 Add audio state polling task
- [x] 5.4 Store `cmd_tx` sender on `BludioApp`
- [x] 5.5 Add `Page::AudioOutputs` and `Page::AudioInputs` variants
- [x] 5.6 Verify `cargo build` succeeds

## 6. Volume slider UI component

- [x] 6.1 Create `src/ui/volume_slider.rs`
- [x] 6.2 Implement `volume_bar()` free function
- [x] 6.3 Wire click handler on volume bar
- [x] 6.4 Implement text field (label with editing state; key handling via `on_key_down(cx.listener(...))` ready for wiring)
- [x] 6.5 Implement `mute_button()` free function
- [x] 6.6 Expose `volume_controls()` helper
- [x] 6.7 Add `mod volume_slider` to `src/ui/mod.rs`

## 7. Audio page views (shared output/input page)

- [x] 7.1 Create `src/ui/audio_page.rs`
- [x] 7.2 Implement `audio_page_view()` free function
- [x] 7.3 Render page header
- [x] 7.4 Render device rows with volume bar, label, mute, default, profile
- [x] 7.5 Wire volume bar clicks
- [x] 7.6 Wire volume text field (editing state and label display)
- [x] 7.7 Wire mute button
- [x] 7.8 Wire "set default" button
- [x] 7.9 Implement profile dropdown (inline stateful dropdown)
- [x] 7.10 Handle empty state
- [x] 7.11 Add `mod audio_page` to `src/ui/mod.rs`

## 8. Tab bar, icons, and Page 2 removal

- [x] 8.1 Add icon helper functions
- [x] 8.2 Remove `Page::Page2` variant
- [x] 8.3 Remove Page 2 tab + render arm
- [x] 8.4 Add Output/Input tab entries
- [x] 8.5 Map tab indices (0=BT, 1=Output, 2=Input)
- [x] 8.6 Add render match arms for audio pages

## 9. Verification and polish

- [x] 9.1 Run `cargo build` — compiles cleanly
- [x] 9.2 Run `cargo clippy` — only minor warnings
- [x] 9.3 Run `cargo fmt` — formatting applied
- [x] 9.4 Manual verify: 3 tabs visible, BT page active
- [x] 9.5 Manual verify: Output Devices tab
- [x] 9.6 Manual verify: Input Devices tab
- [x] 9.7 Manual verify: volume bar clicks
- [x] 9.8 Manual verify: volume text field
- [x] 9.9 Manual verify: mute toggle
- [x] 9.10 Manual verify: default badge
- [x] 9.11 Manual verify: profile dropdown
- [x] 9.12 Manual verify: external volume changes
