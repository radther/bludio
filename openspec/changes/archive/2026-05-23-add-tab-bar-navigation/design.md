## Context

Bludio is a GPUI-based Bluetooth device manager. Currently all UI code lives in `src/main.rs` and `src/device_list.rs`, with a single full-screen view (header + device list). The app needs navigation infrastructure to support multiple pages as features grow. The chosen pattern is a left-mounted icon-only vertical tab bar, matching common desktop app patterns (like VS Code's activity bar or Zed's sidebar panels).

**Current architecture:**
- `BludioApp` owns `BluetoothState` and renders everything in one `Render` impl
- `device_list.rs` provides `device_list_view()` as a free function returning an element
- No navigation or page-switching infrastructure exists

**Constraints:**
- Must compile on Linux with `gpui_platform_gpui_unofficial`
- SVG icons loaded from filesystem (no embedded/font icons)
- Tokio runtime required for Bluetooth D-Bus operations
- Code must meet Zed/GPUI code quality standards

## Goals / Non-Goals

**Goals:**
- A reusable `TabBar` component that renders a vertical stack of icon-only buttons
- Icon loading via the established `svg().external_path()` pattern (same as rss-reader)
- Clean separation: tab bar component knows nothing about what pages render
- Page switching updates the main content area immediately (synchronous state change + notify)
- Visual highlight for the active tab, hover effects for all tabs
- At least two tabs: Bluetooth devices page + a placeholder page
- The left tab bar has a fixed width (~48px) matching common desktop app conventions

**Non-Goals:**
- Animations or transitions between pages (just instant swap)
- Configurable/plugin-based page registry (just a hardcoded enum for now)
- Keyboard navigation between tabs
- Collapsible tab bar
- Drag-and-drop tab reordering
- Nested tabs or sub-tabs
- Settings panel or settings page (placeholder only)

## Decisions

### Decision 1: Page enum over trait-based dynamic dispatch

**Choice:** Use a `Page` enum with a render method, not a `Page` trait with `Entity<dyn Page>`.

**Rationale:** The page count is small and known at compile time. An enum avoids boxing, keeps lifetimes simple, and lets `BludioApp` own all page-specific state directly. Traits + entities would add unnecessary indirection for two static pages. This can be refactored to trait-based when page count exceeds ~5 or needs dynamic registration.

**Alternatives considered:** Trait `Page` with `Entity<dyn PageTrait>` — rejected as over-engineering for two pages. Dynamic dispatch adds complexity without benefit at this scale.

### Decision 2: Icon loading — filesystem SVG via `external_path`, not compiled-in

**Choice:** Load SVG icons from `icons/*.svg` relative to `CARGO_MANIFEST_DIR`, using `svg().external_path()`, matching the rss-reader pattern.

**Rationale:** Proven pattern, GPUI's `svg()` element supports this natively, and it keeps icons easy to swap without recompilation (for development). No build script needed. The icons directory is distributed alongside the binary.

**Alternatives considered:**
- `include_str!()` / compile-time embedding — rejected because GPUI's `svg()` element renders via paths, not raw SVG strings
- Icon font (e.g., icomoon) — rejected, adds dependency and complexity
- GPUI `ui::IconName` enum — core GPUI icons available in Zed but may not be available in the unofficial GPUI fork

### Decision 3: TabBar as a free function, not a View/Entity

**Choice:** `TabBar` is rendered by a free function `tab_bar_view()` that takes a reference to the active page and closures for tab click callbacks, returning an element.

**Rationale:** The tab bar has no mutable state of its own — it's a pure visual component driven by props (active page, on-click callback). This follows Zed's pattern where simple UI components are just functions returning elements, not full `Entity` + `Render` impls. This keeps the component lightweight and easily testable. It mirrors `device_list_view()` already in the codebase.

**Alternatives considered:** `Entity<TabBar>` with `Render` impl — rejected because the tab bar has no independent state to manage. It would add a focus handle, entity lifecycle, and context management for no benefit.

### Decision 4: Tab bar fixed width, content area fills remaining space

**Choice:** Use GPUI's flex layout: `div().flex().flex_row()` wrapping `tab_bar()` (fixed `w(px(48.0))`) + content `div().flex_1()`.

**Rationale:** Standard GPUI flex pattern. The GPUI `flex_1()` on the content area causes it to fill the remaining width. No manual layout calculation needed.

### Decision 5: Active page state on BludioApp, not a separate model

**Choice:** `active_page: Page` field on `BludioApp` struct, mutated in-place with `cx.notify()`.

**Rationale:** `BludioApp` already owns all app state (`BluetoothState`, etc.). Adding `active_page` alongside it is the simplest approach. No need for a separate state management entity or message-passing.

## Risks / Trade-offs

- **[Risk] SVG path resolution may break on non-standard installations.** The `CARGO_MANIFEST_DIR` env var is set at compile time and points to the crate root, so `icons/` must be present relative to the binary's working directory. → **Mitigation:** Copy `icons/` next to the binary via a build script or ensure the Cargo workspace layout keeps it accessible.

- **[Risk] The `Page` enum may grow unwieldy as features are added.** → **Mitigation:** This is acceptable for now with 2-3 pages. When page count grows beyond ~5, refactor to trait-based dynamic dispatch. The tab bar's API (taking an index/u32) is already decoupled from the enum.

- **[Trade-off] No keyboard navigation.** Users must click tabs with the mouse. This is acceptable for an initial navigation implementation; keyboard support can be added later.
