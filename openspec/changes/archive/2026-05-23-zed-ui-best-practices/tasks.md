## 1. Create src/ui/ module with stack, ext, and mod.rs

- [x] 1.1 Create `src/ui/mod.rs` with module declarations and re-exports for `h_flex`, `v_flex`, `StyledExt`, and all component modules
- [x] 1.2 Create `src/ui/ext.rs` with `StyledExt` trait providing `.h_flex()`, `.v_flex()`, and all 6 `debug_bg_*()` helpers, plus blanket impl
- [x] 1.3 Create `src/ui/stack.rs` with `pub fn h_flex() -> Div` and `pub fn v_flex() -> Div` free functions
- [x] 1.4 Run `cargo check` to verify the new module compiles

## 2. Split main.rs into main.rs + app.rs

- [x] 2.1 Create `src/app.rs` with `Page` enum, `BludioApp` struct, `BludioApp::new()`, `impl Render`, `impl Focusable`, `run_discovery()`, `devices_changed()`
- [x] 2.2 Move all entity/render/discovery code from `main.rs` into `app.rs`, updating imports to use `crate::ui::*` and `crate::bluetooth::*`
- [x] 2.3 Strip `main.rs` down to: `mod app; mod ui;`, `TOKIO` lazy static, `tokio_task()`, `fn main()`, `application().run()`, window creation
- [x] 2.4 Run `cargo check` to verify the split compiles

## 3. Move UI components into src/ui/

- [x] 3.1 Move `src/icons.rs` → `src/ui/icons.rs`, update `mod` declarations
- [x] 3.2 Move `src/tab_bar.rs` → `src/ui/tab_bar.rs`, update references
- [x] 3.3 Move `src/tooltip.rs` → `src/ui/tooltip.rs`, update references
- [x] 3.4 Run `cargo check` to verify moved files compile with correct imports

## 4. Merge device_list.rs + header into bluetooth_page.rs

- [x] 4.1 Create `src/ui/bluetooth_page.rs` based on `src/device_list.rs` content
- [x] 4.2 Move `scan_button()`, `error_banner()`, `loading_indicator()` from `src/main.rs` (now `app.rs`) into `bluetooth_page.rs`
- [x] 4.3 Create `bluetooth_page_view()` function that composes the full page: header (scan button) + error/loading states + device list
- [x] 4.4 Update `app.rs` render to call `bluetooth_page::bluetooth_page_view(...)` instead of inlining the header and device list
- [x] 4.5 Delete old `src/device_list.rs`
- [x] 4.6 Run `cargo check` to verify the bluetooth page compiles

## 5. Migrate all UI to h_flex() / v_flex() helpers

- [x] 5.1 In `src/app.rs`, replace all `div().flex().flex_row()` → `h_flex()` and `div().flex().flex_col()` → `v_flex()`
- [x] 5.2 In `src/ui/bluetooth_page.rs`, replace all `div().flex().flex_row()` → `h_flex()` and `div().flex().flex_col()` → `v_flex()`
- [x] 5.3 In `src/ui/tab_bar.rs`, replace `div().flex().flex_col()` → `v_flex()`
- [x] 5.4 Remove now-redundant `.items_center()` calls that `h_flex()` already sets
- [x] 5.5 Run `cargo check` to verify all replacements compile

## 6. Change component return types to Div

- [x] 6.1 Change `scan_button()` return type `impl IntoElement` → `Stateful<Div>`
- [x] 6.2 Change `error_banner()` return type `impl IntoElement` → `Div`
- [x] 6.3 Change `loading_indicator()` return type `impl IntoElement` → `Div`
- [x] 6.4 Change `bluetooth_page_view()` return type `impl IntoElement` → `Div`
- [x] 6.5 Change `device_list_view()` return type `impl IntoElement` → `Stateful<Div>`
- [x] 6.6 Change `device_info()` return type `impl IntoElement` → `Div`
- [x] 6.7 Change `device_buttons()` return type `impl IntoElement` → `Div`
- [x] 6.8 Change `action_btn()` return type `impl IntoElement` → `Stateful<Div>` (uses `.on_mouse_up()`)
- [x] 6.9 Change `tab_bar_view()` return type `impl IntoElement` → `Div`
- [x] 6.10 Change `TooltipLabel::render()` return type to `impl IntoElement` (must match `Render` trait signature; `Div` would be a refining impl trait)
- [x] 6.11 Run `cargo check` and fix any type errors from concrete return types

## 7. Verify and clean up

- [x] 7.1 Run `cargo clippy` and fix any warnings
- [x] 7.2 Run `cargo fmt` to ensure consistent formatting
- [x] 7.3 Run `cargo build` for a clean release build
- [x] 7.4 Remove any stale empty files left behind by moves
- [x] 7.5 Manual visual inspection: verify all imports are clean and module structure matches design
