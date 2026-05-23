## Why

The Bludio codebase currently uses raw `div()` with explicit `.flex()`, `.flex_row()`, `.flex_col()` calls throughout the UI. The Zed codebase (our reference for GPUI patterns) provides cleaner `h_flex()` and `v_flex()` builder functions plus a `StyledExt` trait that makes these available as method chains. Adopting these patterns improves readability, reduces boilerplate, and aligns Bludio with Zed's established conventions.

Additionally, `src/main.rs` has grown to ~470 lines — it handles the Tokio bridge, Bluetooth init, the `BludioApp` entity, all render logic, discovery orchestration, and even page-specific sub-components like the scan button header. Following Zed's pattern, we should split the app entity from the entry point and make pages self-contained rather than leaking their internals into the composition root.

## What Changes

- **Split `main.rs`** into a thin entry point and `src/app.rs` for the app entity, matching Zed's `main.rs` → `zed.rs` pattern:
  - `src/main.rs` — `fn main()`, Tokio bridge (`tokio_task`, `TOKIO`), window creation, `application().run()` bootstrap
  - `src/app.rs` — `Page` enum, `BludioApp` struct, `BludioApp::new()`, `Render`, `Focusable`, `run_discovery()`, `devices_changed()`

- **Make pages self-contained** — the Bluetooth page header (scan button, error banner, loading indicator) currently lives in `main.rs` but belongs with the device list. Merge them:
  - Rename `src/device_list.rs` → `src/ui/bluetooth_page.rs` and absorb `scan_button()`, `error_banner()`, `loading_indicator()` from main

- **Reorganize UI components** into `src/ui/` module:
  - `src/ui/stack.rs` — `h_flex()`, `v_flex()` free functions (returning `Div`)
  - `src/ui/ext.rs` — `StyledExt` trait with `.h_flex()`, `.v_flex()`, and all 6 `debug_bg_*()` helpers
  - `src/ui/bluetooth_page.rs` — moved from `src/device_list.rs`, now includes header sub-components
  - `src/ui/tab_bar.rs` — moved from `src/tab_bar.rs`
  - `src/ui/tooltip.rs` — moved from `src/tooltip.rs`
  - `src/ui/icons.rs` — moved from `src/icons.rs`
  - `src/ui/mod.rs` — module declarations and re-exports

- **Add `h_flex()` / `v_flex()`** free functions and `StyledExt` trait, matching Zed's `crates/ui/src/components/stack.rs` and `crates/ui/src/traits/styled_ext.rs`

- **Add `debug_bg_*()` helpers** (red, green, blue, yellow, cyan, magenta) to `StyledExt` — standalone `hsla()` calls useful for visually debugging layout during development

- **Migrate all UI code** from `div().flex().flex_row()` → `h_flex()` and `div().flex().flex_col()` → `v_flex()`

- **Return `Div` from component functions** instead of `impl IntoElement`, matching Zed's convention

## Capabilities

### New Capabilities
<!-- No new user-facing capabilities — this is a code organization and pattern alignment change. -->

### Modified Capabilities
<!-- No spec-level behavior changes. All existing specs remain unchanged as the observable behavior is identical. -->

## Impact

- Source files: `src/main.rs` (shrinks), `src/app.rs` (new), `src/device_list.rs` → `src/ui/bluetooth_page.rs` (renamed + expanded), `src/tab_bar.rs`, `src/tooltip.rs`, `src/icons.rs` (moved to `src/ui/`)
- `src/actions.rs` and `src/bluetooth/` are unaffected
- No dependency changes — uses existing `gpui` and `gpui-unofficial` APIs
- No breaking changes to public API (the crate has no external consumers)
- Build and test commands remain unchanged (`cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`)
