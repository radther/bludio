# Bludio — GPUI Bluetooth Manager

Rust GUI application using `gpui-unofficial` (matches the version used for the Zed editor) for the UI and `bluer` (BlueZ D-Bus) for Bluetooth management.

## Build & Run

```bash
cargo build
cargo run
cargo test
cargo clippy
cargo fmt
```

Rust toolchain is pinned to **1.95.0** (see `rust-toolchain.toml`).

## Architecture

```
src/
  main.rs            — Entry point, Tokio bridge, window creation
  app.rs             — BludioApp entity, Page navigation, Render, discovery
  actions.rs         — Bluetooth action execution
  bluetooth/         — BlueZ D-Bus backend (agent, device, discovery, monitor, properties)
  ui/                — UI components and utilities (ext, stack, icons, tab_bar, tooltip, bluetooth_page)
```

## Core Patterns

### GPUI + Tokio bridge
GPUI's executor is not Tokio-based, but `bluer` requires Tokio. We spin up a global `LazyLock<tokio::runtime::Runtime>` and route all Bluetooth work through `tokio_task()` which returns a `oneshot::Receiver<T>`. All D-Bus calls happen on the Tokio runtime, results are sent back to GPUI via `cx.spawn` + `this.update()`.

### Async update flow
Pattern for any async operation that updates UI state:
1. `cx.spawn(async move |this, cx| { ... }).detach()` — spawn from GPUI context
2. `TOKIO.enter()` — enter the Tokio context on the GPUI thread (needed for mpsc::recv)
3. `tokio_task(...)` — ship work to the Tokio runtime
4. `rx.await` — receive result
5. `this.update(cx, |this, cx| { ...; cx.notify(); })` — apply to UI state

### Device filtering (blueman parity)
The display name resolution and filtering logic matches [blueman](https://github.com/blueman-project/blueman)'s default behavior:
- Broadcast name (`Name` D-Bus property) takes precedence over alias
- Alias that equals the raw MAC is treated as "no alias set" — device is hidden unless paired
- Unpaired devices without RSSI during scan are filtered out
- In-range checks via RSSI for non-paired devices during scan

## Naming Conventions
- Modules: `snake_case` files, `pub mod` / `pub(crate) mod` visibility
- Functions: `snake_case`, `pub(crate)` for cross-module internal use
- Structs/enums: `PascalCase`
- Section comments: `// ── Section Name ──` with Unicode box-drawing characters (─)
- Module doc comments at file level only (`//!`)

## Known Limitations
- Pairing agent auto-accepts all requests — only suitable for testing
- No persistent preferences or settings storage
- Only handles the default Bluetooth adapter
- `#[allow(dead_code)]` on `trusted` field — present for future use but not displayed


## GPUI best practices
- Always treat the Zed editor as the source of truth for UI conventions and patterns, it has been cloned locally and can be found at `../zed/` when needing to be referenced.
- Use the Zed editor's main source code as reference point, not the gpui examples also in that repository.
- gpui-unofficial tracks Zed's internal gpui implementation exactly, so if it works in Zed, it will work in gpui-unofficial.
- Any component with interactivity requires an id. Without it hover, click states, and rendering, won't behave correctly.
- cx.notify() must be called for the UI to reflect changes.

### Layout conventions
- Always use `h_flex()` and `v_flex()` free functions over raw `div().flex().flex_row()` / `div().flex().flex_col()`. They are more readable and match Zed's conventions.
- `h_flex()` defaults to `items_center()` (cross-axis centering). When this isn't desired, override with one of:
  - `.items_start()` — align children to the top/left
  - `.items_stretch()` — restore default flex cross-axis stretch (use for root containers where children should fill the cross-axis)
- `v_flex()` doesn't set any items alignment — add `.items_center()` etc. as needed.

### Debug helpers
- `StyledExt` in `src/ui/ext.rs` provides `debug_bg_red()`, `debug_bg_green()`, `debug_bg_blue()`, `debug_bg_yellow()`, `debug_bg_cyan()`, `debug_bg_magenta()` — chain these on any element during debugging to visualize its bounds. Remove after debugging.

### Return types
- Functions whose builder chain includes `.id()`, `.hover()`, or `.on_*()` return `Stateful<Div>` (gpui transitions the builder type). Pure styling functions return `Div`. Trait impls (`Render::render`) keep `impl IntoElement` to match the trait signature.

---

## Agent Notes

When working in this codebase, **keep this file in mind** — it documents the conventions and patterns discovered so far. After completing and archiving an OpenSpec change (i.e., when the implementation has settled), review this file and suggest additions if you noticed new conventions, patterns, or important context that should be captured here. Do not propose updates mid-implementation; wait until the code is stable and the change is archived.

### On diverging from the plan
OpenSpec artifacts (design.md, tasks.md) are a starting point — they describe intent, not ground truth. When implementation reveals that the plan was wrong about a specific detail (e.g., gpui's actual return types don't match what the design assumed), accept the correction and move forward. Don't loop trying to make reality fit a mistaken assumption. Minor deviations like concrete type changes (`Div` vs `Stateful<Div>`) are expected and don't require design updates unless the divergence is significant. When in doubt, correct the code, note the deviation, and update the design doc after the fact.
