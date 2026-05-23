## Context

Bludio currently builds UI with raw GPUI builder calls like `div().flex().flex_row().items_center()`. Zed's `crates/ui` library provides two layers of convenience:

1. **`StyledExt` trait** (`crates/ui/src/traits/styled_ext.rs`) — extension methods on any `Styled` type. We'll adopt `h_flex`, `v_flex`, and the 6 debug color helpers (`debug_bg_red`, etc.). We skip elevation/border methods since Bludio doesn't have Zed's theme system yet.

2. **Free functions** (`crates/ui/src/components/stack.rs`) — `pub fn h_flex() -> Div` and `pub fn v_flex() -> Div` for concise container creation.

Additionally, `src/main.rs` is approaching 500 lines — far beyond a thin entry point. Zed splits its entry point: `main.rs` handles CLI + platform boot, while `zed.rs` holds app entity code. We'll split Bludio similarly before it grows further.

Finally, pages should be self-contained: the Bluetooth header (scan button, error banner, loading indicator) currently lives in `main.rs`'s render block but belongs with the device list in the bluetooth page module.

## Goals / Non-Goals

**Goals:**
- Split `main.rs` into thin entry point (`src/main.rs`) and app entity (`src/app.rs`)
- Make pages self-contained: move scan button, error banner, loading indicator into the bluetooth page module
- Rename `device_list.rs` → `bluetooth_page.rs` to reflect its expanded scope
- Create `src/ui/` module with sub-modules for all UI components
- Add `h_flex()`, `v_flex()` free functions and `StyledExt` trait with `h_flex`, `v_flex`, and all 6 `debug_bg_*()` helpers
- Migrate from `div().flex().flex_row()` → `h_flex()` and `div().flex().flex_col()` → `v_flex()`
- Change component function return types from `impl IntoElement` → `Div`
- Keep `main.rs` at crate root (it contains `fn main()`)

**Non-Goals:**
- Adopting Zed's full component system (`#[derive(IntoElement)]`, `RenderOnce`, `Component` trait)
- Adding `elevation_*` or `border_*` methods from Zed's `StyledExt` — require theme system we don't have yet
- Moving `actions.rs` or `bluetooth/` — they're not UI components
- Changing observable behavior of any UI element

## Decisions

### Decision 1: Split main.rs → main.rs + app.rs

Following Zed's `main.rs` / `zed.rs` pattern:

| Before | After | Contents |
|--------|-------|----------|
| `src/main.rs` (~470 lines) | `src/main.rs` (~60 lines) | `TOKIO` lazy static, `tokio_task()`, `fn main()`, `application().run()`, window creation |
| (inline) | `src/app.rs` (~470 lines) | `Page` enum, `BludioApp` struct, `BludioApp::new()`, `impl Render`, `impl Focusable`, `run_discovery()`, `devices_changed()` |

`main.rs` declares `mod app;` and imports `BludioApp`, `Page`, `run_discovery`. The render method composes pages via `bluetooth_page::bluetooth_page_view()` instead of inlining the header.

**Alternative**: Keep main.rs monolithic. Rejected — it's already too long and will grow.

### Decision 2: Self-contained pages — bluetooth_page.rs absorbs header

The Bluetooth page currently has its header (`scan_button`, `error_banner`, `loading_indicator`) defined as free functions in `main.rs`. These are page-specific, not app-wide. They move into the bluetooth page module.

| Before | After |
|--------|-------|
| `src/device_list.rs` — `device_list_view()`, `device_info()`, `device_buttons()`, `action_btn()` | `src/ui/bluetooth_page.rs` — `bluetooth_page_view()`, `device_list_view()`, `device_info()`, `device_buttons()`, `action_btn()`, `scan_button()`, `error_banner()`, `loading_indicator()` |

`bluetooth_page_view()` is the new top-level entry point — it returns the complete page including header, error/loading states, and device list. `app.rs` render just calls `bluetooth_page::bluetooth_page_view(...)` for `Page::BluetoothDevices`.

### Decision 3: Module structure

```
src/
  main.rs              — entry point, tokio bridge, window creation
  app.rs               — BludioApp entity, Page enum, Render, discovery
  actions.rs           — unchanged (bluetooth action execution)
  bluetooth/           — unchanged (agent, device, discovery, monitor, properties, mod)
  ui/
    mod.rs             — module declarations, re-exports
    ext.rs             — StyledExt trait (h_flex, v_flex, debug_bg_*)
    stack.rs           — h_flex(), v_flex() free functions
    icons.rs           — SVG icon helpers
    tab_bar.rs         — vertical icon tab bar
    tooltip.rs         — tooltip label component
    bluetooth_page.rs  — full Bluetooth page (header + device list)
```

### Decision 4: Copy debug helpers from StyledExt

All 6 `debug_bg_*()` methods are standalone `hsla()` calls with no theme dependency:

```rust
fn debug_bg_red(self) -> Self    { self.bg(hsla(0. / 360., 1., 0.5, 1.)) }
fn debug_bg_green(self) -> Self  { self.bg(hsla(120. / 360., 1., 0.5, 1.)) }
fn debug_bg_blue(self) -> Self   { self.bg(hsla(240. / 360., 1., 0.5, 1.)) }
fn debug_bg_yellow(self) -> Self { self.bg(hsla(60. / 360., 1., 0.5, 1.)) }
fn debug_bg_cyan(self) -> Self   { self.bg(hsla(160. / 360., 1., 0.5, 1.)) }
fn debug_bg_magenta(self) -> Self { self.bg(hsla(300. / 360., 1., 0.5, 1.)) }
```

These are useful during UI development for visualizing layout boxes. They compile immediately — no `cx` needed, no theme dependency. Full saturation, 50% lightness, full opacity.

### Decision 5: Return `Div` instead of `impl IntoElement`

All component functions return concrete `Div` or `Stateful<Div>`:

- `scan_button(...) → Stateful<Div>` (`.id()`, `.hover()`, `.on_mouse_up()` transition the builder)
- `error_banner(...) → Div`
- `loading_indicator(...) → Div`
- `bluetooth_page_view(...) → Div`
- `device_list_view(...) → Stateful<Div>` (`.id("device-list")` on outer chain)
- `device_info(...) → Div`
- `device_buttons(...) → Div`
- `action_btn(...) → Stateful<Div>` (uses `.on_mouse_up()`)
- `tab_bar_view(...) → Div`
- `TooltipLabel::render(...) → impl IntoElement` (must match `Render` trait signature; concrete `Div` would be a refining impl trait)

**Rule**: Functions with `.id()`, `.hover()`, or `.on_*()` on their main builder chain return `Stateful<Div>`. Pure styling functions return `Div`. Trait impls retain `impl IntoElement` to match the trait contract.

## Risks / Trade-offs

- **[Risk]** Splitting main.rs may introduce import loops or visibility issues.
  → **Mitigation**: `app.rs` imports from `ui::*` and `bluetooth::*`; `main.rs` only imports from `app` and `ui`. No circular dependency possible.

- **[Risk]** Moving `scan_button` into bluetooth_page.rs requires the discovery callback to call `run_discovery` from app.rs.
  → **Mitigation**: `bluetooth_page::scan_button()` takes a `gpui::WeakEntity<BludioApp>` and spawns discovery via `cx.spawn`, matching the existing pattern. No new coupling.

- **[Trade-off]** `h_flex()` is opinionated about `items_center()` — matches Zed's default but may not suit every container.
  → **Mitigation**: `h_flex()` returns `Div` so callers can override with `.items_start()` or `.items_stretch()`. The root layout uses `.items_stretch()` to restore default cross-axis stretch behavior; inner containers benefit from `items_center()`.
