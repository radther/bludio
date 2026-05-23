## Why

Bludio currently renders a single full-screen view (Bluetooth device list). There is no navigation infrastructure to switch between different app pages. A left-mounted icon tab bar provides the foundational navigation pattern needed for adding future pages (settings, device details, etc.) while following established Zed/GPUI UI patterns for sidebar navigation.

## What Changes

- Add a reusable `icons` module providing SVG icon helpers (ported from rss-reader patterns) for the Lucide icon set
- Add a `tab_bar` module implementing a left-mounted, icon-only vertical tab bar component
- Introduce a `Page` enum representing the available pages in the app (Bluetooth devices, plus a temporary placeholder page)
- Restructure `BludioApp` to hold active page state and render the tab bar + active page side-by-side
- The header (currently containing "bluetooth" title + scan button) moves into the Bluetooth page view, since it is page-specific
- Add a `Tab` struct for tab configuration (icon, tooltip, page)
- The tab bar is a reusable, self-contained component with clear separation from page rendering logic

## Capabilities

### New Capabilities
- `tab-bar-navigation`: A reusable, left-mounted icon-only vertical tab bar component that allows switching between app pages. Each tab is defined by a configuration struct (icon, tooltip, page identifier). Only one tab can be active at a time. The active tab is visually highlighted.

### Modified Capabilities
- `app-shell`: The app shell's rendering is restructured from a single full-screen view to a horizontal layout with a left tab bar (fixed width, ~48px) and a right content area that renders the active page. The app identity text "bluetooth" in the header moves to the Bluetooth page view since it is page-specific.

## Impact

- **New files**: `src/icons.rs`, `src/tab_bar.rs`
- **Modified files**: `src/main.rs` (restructure `BludioApp` struct and `Render` impl to support page-based layout)
- **Deleted files**: None
- **Dependencies**: No new crate dependencies. Uses existing GPUI primitives (`div`, `flex`, `hover`, `bg`, etc.)
- **Icons**: SVG icon files copied from `gpui_test/crates/rss-reader/icons/` into `bludio/icons/`
