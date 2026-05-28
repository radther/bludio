## Context

Device actions in `bluetooth/device.rs` currently log errors to stderr and return `()`. The UI has no mechanism to display errors — users see no feedback when connect/disconnect/forget fails. The `PairAndTrust` flow has partial error surfacing via `PairingStatus::Failed` on the device row, but this is specific to pairing and doesn't apply to other actions.

The tab bar already uses a `cx.spawn` frame-loop animation for its sliding indicator. The RSS reader example (`../gpui_test/crates/rss-reader/`) uses GPUI's `with_animation` API for declarative element animations.

## Goals / Non-Goals

**Goals:**
- Reusable error banner component that can be used across any page
- Smooth slide-up animation using GPUI's `with_animation` API
- Human-readable error messages from BlueZ D-Bus errors
- Banner auto-dismisses after a timeout or on user click

**Non-Goals:**
- Service-level error handling (Bluetooth daemon crash, PulseAudio down) — future work
- Toast notification system — this is page-local, not app-global
- Error history or logging beyond stderr

## Decisions

### 1. Animation API: `with_animation` over `cx.spawn` frame loop

The tab bar uses a `cx.spawn` frame loop for its indicator animation because it needs to interpolate between discrete positions (tab indices) with a generation counter to handle rapid re-selection.

For the error banner, `with_animation` is better because:
- It's declarative — the animation is part of the element tree, not a side effect
- It handles show/hide transitions naturally (animate in, animate out)
- It matches the RSS reader pattern the user referenced
- No need for generation counters — the animation is per-element, not per-entity

The banner will animate from off-screen (below viewport) to its final position.

### 2. Banner placement: Shared component, rendered at page level

The banner is a shared component in `src/ui/components/error_banner.rs`. Each page that wants error display manages its own `error_banner_text` state and renders the banner at the bottom of its layout. This keeps the component reusable while letting each page own its error lifecycle.

### 3. Error message extraction from BlueZ errors

`bluer::Error` wraps D-Bus errors. The key error variants to map:
- `org.bluez.Error.NotAvailable` → "Device is out of range or not responding"
- `org.bluez.Error.NotReady` → "Bluetooth adapter is not ready"
- `org.bluez.Error.AlreadyConnected` → "Device is already connected"
- `org.bluez.Error.NotConnected` → "Device is not connected"
- `org.bluez.Error.InProgress` → "Another operation is in progress"
- `org.bluez.Error.Failed` → Generic "Operation failed" (with original error string appended)
- `org.bluez.Error.AuthenticationTimeout` → "Pairing timed out — try again"
- `org.bluez.Error.AuthenticationRejected` → "Pairing was rejected by the device"
- `org.bluez.Error.AuthenticationCanceled` → "Pairing was canceled"

We'll parse the error string for known D-Bus error names and provide friendly messages, falling back to a generic message with the raw error.

### 4. Banner state management

The banner state (visible, message, animation phase) lives in the consuming page as simple fields. No separate Entity needed — it's a visual component, not an interactive one. The page's `render()` method conditionally includes the banner element.

State fields (per page that uses the banner):
- `error_banner_text: Option<String>` — None = hidden, Some = visible with message
- `error_banner_generation: u64` — for animation keying (new error = new animation)

Auto-dismiss: A spawned timer sets `error_banner_text = None` after ~5 seconds.

The component function signature: `error_banner(text: &str, generation: u64, on_dismiss: impl Fn() + 'static) -> gpui::Div` (returns concrete `Div` type, matching `page_header` pattern)

## Risks / Trade-offs

- **Animation interruption**: If a new error arrives while the banner is animating out, the generation counter ensures the new animation starts fresh. The old animation task checks the generation and aborts.
- **Error message accuracy**: BlueZ error strings vary across versions. The friendly message mapping may not cover all cases — the fallback includes the raw error for debugging.
- **Banner overlap**: If the banner is at the bottom and the device list is scrollable, the banner may overlap the last device. Solution: add bottom padding to the scroll area when the banner is visible.

## Open Questions

- Should clicking the banner dismiss it immediately, or just let the auto-dismiss handle it? → Let user click to dismiss (feels more responsive).
