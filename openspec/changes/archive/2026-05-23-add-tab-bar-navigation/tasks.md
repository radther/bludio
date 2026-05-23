## 1. Icons module

- [x] 1.1 Copy SVG icon files from `gpui_test/crates/rss-reader/icons/` into `bludio/icons/`
- [x] 1.2 Create `src/icons.rs` with icon helper functions for Bluetooth and placeholder page icons, following the `svg().external_path()` pattern from rss-reader's `icons.rs`

## 2. Tab bar component

- [x] 2.1 Create `src/tab_bar.rs` with `Tab` configuration struct (icon function pointer, tooltip string, page identifier)
- [x] 2.2 Implement `tab_bar_view()` free function that renders a vertical column of icon-only buttons, with active-tab highlighting and hover effects
- [x] 2.3 Add `mod tab_bar` and `mod icons` and `mod tooltip` declarations to `src/main.rs`

## 3. Page navigation model

- [x] 3.1 Define a `Page` enum in `src/main.rs` with variants for Bluetooth devices and the placeholder page
- [x] 3.2 Add `active_page: Page` field to `BludioApp` struct, initialized to the Bluetooth devices page

## 4. App shell restructure

- [x] 4.1 Refactor `BludioApp::render` to produce a horizontal flex layout: left tab bar (fixed 48px width) + right content area (flex-1)
- [x] 4.2 Move the current header ("bluetooth" title + scan button) from the global `render` into the Bluetooth page content rendering branch
- [x] 4.3 Implement content area branching: match on `active_page` to render the Bluetooth page or the placeholder "Page 2" centered text

## 5. Tab click handling

- [x] 5.1 Wire tab `on_mouse_up` callbacks to set `self.active_page` to the corresponding `Page` variant and call `cx.notify()`
- [x] 5.2 Ensure clicking the already-active tab is a no-op (early return in handler)

## 6. Verification

- [x] 6.1 Run `cargo build` and fix any compilation errors
- [x] 6.2 Manually verify: app opens with Bluetooth page active and tab bar visible on the left
- [x] 6.3 Manually verify: clicking the second tab switches to "Page 2" placeholder and back
- [x] 6.4 Manually verify: active tab is visually highlighted, inactive tabs show hover effects
