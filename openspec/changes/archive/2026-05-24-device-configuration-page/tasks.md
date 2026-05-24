## 1. CardInfo enrichment (PA backend + audio types)

- [x] 1.1 Add `description: Option<String>` field to `CardInfo` in `src/audio/mod.rs` — stores the human-readable name from `device.description` proplist
- [x] 1.2 In `src/audio/pulse.rs`, read `ci.proplist.get_str(properties::DEVICE_DESCRIPTION)` in the card listing callback and populate `CardInfo.description`, falling back to `None` when not set

## 2. Card row entity

- [x] 2.1 Create `src/ui/audio/card_row.rs` with `CardRow` entity following the `AudioDeviceRow` pattern: owns an `Entity<Dropdown>` for profile selection, stores `cmd_tx` and `wakeup` for sending `SetCardProfile` commands
- [x] 2.2 Implement `CardRow::new()` constructor that takes `CardInfo`, `cmd_tx`, `wakeup`, `window`, `cx` — creates a `Dropdown` from profiles with the active profile selected, stores description for display
- [x] 2.3 Implement `CardRow::update_from_card()` to sync display name (description → name fallback) and profile dropdown items/selection from fresh `CardInfo`
- [x] 2.4 Add `cx.subscribe_in()` on the dropdown to handle `DropdownEvent::Selected` — extract profile name, send `AudioCommand::SetCardProfile(card_index, profile)` via `cmd_tx`, wake the PA loop
- [x] 2.5 Implement `Render` for `CardRow`: show card description label (or PA name fallback) + profile dropdown in a horizontal row, matching the visual style of `AudioDeviceRow` rows (same height, border, hover, padding)
- [x] 2.6 Implement `Focusable` for `CardRow`

## 3. Configuration page entity

- [x] 3.1 Create `src/ui/audio/configuration_page.rs` with `ConfigurationPage` entity following the `AudioPage` pattern: owns `Vec<Entity<CardRow>>`, `cmd_tx`, `wakeup`, `connected`, `error`
- [x] 3.2 Implement `ConfigurationPage::new()` constructor taking `cmd_tx`, `wakeup`, `cx`
- [x] 3.3 Implement `ConfigurationPage::sync_cards(&mut self, state: &AudioState, window, cx)` — create/update/remove `CardRow` entities based on `state.cards`, only proceed when `wakeup` is `Some`
- [x] 3.4 Implement `Render` for `ConfigurationPage`: show "Configuration" title header, error banner when `self.error` is set, "Connecting..." when not connected, empty state when no cards, scrollable list of card rows when connected with cards

## 4. Wire into BludioApp

- [x] 4.1 Add `pub mod card_row;` and `pub mod configuration_page;` to `src/ui/audio/mod.rs`
- [x] 4.2 Add `Configuration` variant to `Page` enum in `src/app.rs`
- [x] 4.3 Add `configuration_page: Entity<ConfigurationPage>` field to `BludioApp`
- [x] 4.4 Instantiate `ConfigurationPage` in `BludioApp::new()`, passing `audio_cmd_tx` and `pa_wakeup`
- [x] 4.5 Add Configuration page sync call in `spawn_audio_state_loop`: `this.configuration_page.update(cx, |page, cx| page.sync_cards(&state, window, cx));`
- [x] 4.6 Add Configuration tab to the tabs array (after Input Devices) with `icons::audio_card` icon and mapping `Page::Configuration` to `active_tab_index`
- [x] 4.7 Add `Page::Configuration => self.configuration_page.clone().into_any_element()` in the render match

## 5. Icons

- [x] 5.1 Add `audio_card` icon to `src/ui/icons.rs` (use a Unicode character or SVG path — a card/wrench icon, e.g., `⚙` or a gear-like symbol)

## 6. Verification

- [x] 6.1 Run `cargo build` — ensure no compilation errors
- [x] 6.2 Run `cargo clippy` — ensure no warnings
- [x] 6.3 Run `cargo fmt` — ensure formatting is consistent
- [x] 6.4 Run `cargo test` — ensure existing tests pass (no regressions)
