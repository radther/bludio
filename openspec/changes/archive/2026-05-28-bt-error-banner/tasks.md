## 1. Error Banner Component (shared)

- [x] 1.1 Create `src/ui/components/error_banner.rs` with `error_banner(text, generation, on_dismiss)` function signature
- [x] 1.2 Implement slide-up animation using GPUI's `with_animation` API (animate from off-screen to visible)
- [x] 1.3 Style banner with tab bar background color, full width, page-header-matching padding
- [x] 1.4 Add click-to-dismiss handler that invokes the `on_dismiss` callback
- [x] 1.5 Register module in `src/ui/components/mod.rs`

## 2. Bluetooth Page Integration

- [x] 2.1 Add `error_banner_text: Option<String>` and `error_banner_generation: u64` fields to `BluetoothPage`
- [x] 2.2 Add `show_error(&mut self, msg: String, cx)` and `clear_error(&mut self, cx)` methods
- [x] 2.3 Implement auto-dismiss timer (spawn task that clears error after 5 seconds)
- [x] 2.4 Render the error banner at the bottom of the Bluetooth page's render method
- [x] 2.5 Add bottom padding to device list scroll area when banner is visible

## 3. Device Action Error Surfacing

- [x] 3.1 Change `execute_device_action` return type from `()` to `Result<(), String>`
- [x] 3.2 Map BlueZ errors to human-readable messages (NotAvailable → "out of range", etc.)
- [x] 3.3 Update app.rs command handler to pass errors to BluetoothPage's `show_error`
- [x] 3.4 Remove TODO comment from device.rs

## 4. Polish

- [x] 4.1 Test rapid error arrival (new error replaces current banner, timer resets)
- [x] 4.2 Test animation interruption (error during slide-out restarts slide-in)
- [x] 4.3 Verify banner doesn't overlap critical UI elements
