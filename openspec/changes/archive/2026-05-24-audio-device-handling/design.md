## Context

Bludio is a GPUI-based Bluetooth device manager with a left-mounted tab bar and page-based navigation. Audio device control is the next major feature area. The system runs PipeWire 1.6.5 with PulseAudio compatibility, meaning `libpulse` calls work transparently through the PipeWire-Pulse compatibility layer.

**Current architecture:**
- `BludioApp` owns `BluetoothState` and an `active_page: Page` enum driving page rendering
- BT operations run on a global `LazyLock<tokio::runtime::Runtime>` via `tokio_task()`
- UI follows the pattern: `cx.spawn` → `tokio_task` → `rx.await` → `this.update(cx, ...)` → `cx.notify()`
- Page views are free functions returning elements (no Entity/View for simple components)
- Device list pattern from `bluetooth_page.rs`: header + scrollable rows with info and action buttons

**PulseAudio architecture:**
- `libpulse-binding` uses callback-driven API with its own `Mainloop` (not Tokio-compatible)
- Must run PA mainloop on a dedicated std thread
- Communication: `std::sync::mpsc` channels between PA thread and GPUI async tasks
- Sinks (outputs), Sources (inputs), Cards (hardware with profiles), ServerInfo (defaults)
- Subscription events for reactive updates

**Constraints:**
- Must compile on Linux with `gpui_platform_gpui_unofficial`
- Rust toolchain pinned to 1.95.0
- `libpulse-dev` system package required at build time
- No new GPUI crate dependencies — uses existing GPUI primitives
- Follow Zed's UI patterns (not gpui examples)

## Goals / Non-Goals

**Goals:**
- List all PulseAudio output devices (sinks) with name, volume, mute state, profile, and default status
- List all PulseAudio input devices (sources) with name, volume, mute state, and default status
- Per-device volume control via clickable bar (set volume percentage) and mute toggle button
- Switch card profiles (e.g., A2DP ↔ HSP/HFP) for output devices via dropdown selector
- Per-device mute toggle button
- Volume text field for precise numeric volume entry alongside the clickable volume bar
- Set a device as the system default sink or source
- Two new tabs in the tab bar: "Output Devices" and "Input Devices" (replacing the "Page 2" placeholder)
- Reactive updates: UI reflects external volume/property changes via PA subscription events
- Follow the same GPUI patterns as the existing Bluetooth page

**Non-Goals:**
- Bluetooth codec switching (PipeWire-specific `send_message_to_object`) — handled later
- Playback/recording stream management (which apps are playing/recording audio) — separate feature
- Drag-to-adjust volume slider (click-to-set + text field is sufficient for functional MVP)
- Volume debouncing (send volume updates immediately — debounce added later if needed)
- Fancy custom dropdown widget (native-looking text-based dropdown is sufficient for functional MVP)
- Configurable audio device naming or aliases
- Multiple audio server connections (single PA context)
- Port switching (e.g., speaker vs headphone jack) — profile switching only

## Decisions

### Decision 1: PulseAudio integration — dedicated std thread with channels

**Choice:** Run `libpulse-binding`'s `Mainloop` on a dedicated `std::thread`, communicating with GPUI via `std::sync::mpsc` channels. The PA thread owns all PA objects (context, introspection, subscription) and stays alive for the app lifetime.

**Rationale:** `libpulse-binding`'s `Mainloop` is not `Send` and requires polling on its owning thread. This is the canonical pattern for Rust applications using `libpulse-binding` — a background thread iterates the mainloop while channels carry commands and state snapshots to/from the UI thread. This mirrors how `pulsectl` and other Rust PA consumers work. It also cleanly separates PA's callback-driven API from GPUI's async executor.

**Alternatives considered:**
- `libpulse-sys` raw FFI + Tokio integration — significantly more complex, manual unsafe code, no benefit over channels for this use case
- Native `pipewire` crate — would bypass PulseAudio compatibility, doesn't cover non-PipeWire setups, and adds a heavy GObject dependency
- `pulsectl` crate — synchronous API only, would block GPUI's render thread on every operation; not suitable for an interactive GUI
- PA mainloop on Tokio via `tokio::spawn_blocking` — adds Tokio dependency for PA operations while PA already has its own mainloop; unnecessary coupling

### Decision 2: Shared audio page component for outputs and inputs

**Choice:** A single `audio_page_view()` free function in `src/ui/audio_page.rs` that takes a `DeviceKind` enum (`Output` / `Input`) and renders the appropriate device list. Both pages share the same row layout pattern (name + volume controls + default toggle), with output rows additionally showing profile/codec info.

**Rationale:** Output and input device pages are structurally identical — a scrollable list of rows with per-device controls. The only difference is the data source (sinks vs sources) and whether profile info is shown. A shared component avoids duplication while keeping the code simple. This follows the established `bluetooth_page_view()` pattern of a single free function returning an element.

**Alternatives considered:**
- Two separate page functions — more code duplication for near-identical layouts
- Trait-based generic device row — over-engineering for two very similar variants; an enum parameter is simpler

### Decision 3: Volume control — clickable bar + text field input

**Choice:** A `volume_bar()` free function in `src/ui/components/volume_slider.rs` that renders a horizontal bar (filled portion + unfilled portion). Clicking at a position on the bar sets the volume to that percentage. A `TextField` entity (`src/ui/components/text_field.rs`) alongside the bar accepts direct numeric input (e.g., typing "75" sets volume to 75%). A separate mute button toggles mute state. The text field doubles as the percentage display — it shows the current volume as a bare number (without a "%" suffix).

**Note on text field implementation:** The original design planned a simplified div-based text input (`track_focus` + `on_key_down`) to avoid the full GPUI `EntityInputHandler` pattern. During implementation this approach proved insufficient — managing focus lifecycle, keyboard interactions, and IME input without `EntityInputHandler` led to edge cases around focus tracking and platform text input. The implementation therefore uses a full `EntityInputHandler`-based `TextField` entity with cursor blinking, selection, clipboard, and IME support. The entity emits `TextFieldEvent`s (`Confirmed`, `Cancelled`) for parent components to handle. While heavier than the original simplified plan, this provides robust text input that works correctly across all input methods.

**Rationale:** The clickable bar provides visual feedback and quick approximate setting. The text field allows precise numeric entry — essential for users who want exact values. Both controls update each other: clicking the bar updates the text field, and entering a number updates the bar.

**Alternatives considered:**
- Full drag slider — Zed uses this pattern but it requires managing `ElementId`, mouse capture, and drag state; too complex for initial implementation
- Click bar only (no text field) — no way to enter precise values, rejected per user feedback
- +/- buttons only — tedious for large volume changes
- Text field only — less visual, harder to quickly gauge current level

### Decision 4: Audio state on BludioApp, updated via channel-driven background loop

**Choice:** An `AudioState` struct held on `BludioApp` (like `bt_state`). A background `std::thread` runs the PA mainloop and sends state snapshots via `std::sync::mpsc::Sender<AudioState>`. In `BludioApp::new`, a `cx.spawn` task receives from the channel and calls `this.update(cx, |this, cx| { this.audio_state = new_state; cx.notify(); })`. Commands (volume change, profile switch, default device) are sent to the PA thread via a separate `std::sync::mpsc::Sender<AudioCommand>`.

**Rationale:** Follows the established Bluetooth pattern — state on `BludioApp`, background updates via channels. The Bluetooth monitor already uses `tokio::sync::mpsc` for signal-driven updates; the PA thread uses `std::sync::mpsc` because it runs outside Tokio. This keeps the GPUI integration surface consistent.

**Alternatives considered:**
- Separate `Entity<AudioManager>` — adds entity lifecycle complexity with no benefit since only one audio manager exists
- Inline PA calls from GPUI callbacks — PA operations block, would freeze the UI

### Decision 5: Card profile mapping — cards linked to sinks/sources by card index

**Choice:** PA cards have an `index` field. When listing sinks/sources, each has an `owner_module` and `card` index that links back to the owning card. We build a `HashMap<u32, CardInfo>` from the card list, then for each sink/source, look up its card by index to find available profiles and the active profile.

**Rationale:** PulseAudio's data model links sinks/sources to cards via index. This is how pavucontrol does it — fetch cards first, then cross-reference. The profile information (available profiles, active profile) lives on the card, not the sink/source.

### Decision 6: Profile/Codec selection — simple stateful dropdown using GPUI primitives

**Choice:** Profile and codec options are presented in a simple dropdown built from GPUI primitives (`div`, conditional rendering, mouse events). The dropdown tracks its open/closed state on a per-card-index basis (e.g., `open_profile_dropdown: Option<u32>` on `BludioApp`). Clicking the trigger toggles the dropdown; clicking an option selects it and closes.

**Rationale:** Zed's UI crate provides `DropdownMenu`, `ContextMenu`, `PopoverMenu`, and `PopoverMenuHandle` components, but Bludio uses `gpui-unofficial` directly without Zed's UI component library — those components aren't available. However, the pattern is straightforward to replicate with raw GPUI:
1. A trigger `div` shows the currently selected profile name
2. On click, toggle `open_profile_dropdown` state and call `cx.notify()`
3. A conditional `.when(open, |el| el.child(options_list))` renders the option list below the trigger
4. Each option is a clickable `div` that sends the profile command and closes the dropdown
5. Click-outside-to-close is deferred to a follow-up (options can always be chosen to close)

**Floating (non-inline) expansion:** The expanded options list uses GPUI's `deferred(anchored(...))` pattern to float above the device list rather than pushing content down:
- `anchored()` positions the options list at a fixed window coordinate (calculated from the trigger element's position), with `.snap_to_window()` to keep it within bounds
- `deferred()` renders it in a separate draw pass so it paints on top of the list rows below
- The trigger element remains inline in the row; only the expanded options overlay floats
- This matches how Zed's tooltips (`crates/gpui/src/elements/anchored.rs`), popovers, and dropdowns (`crates/ui/src/components/dropdown_menu.rs`) all float above content

GPUI reference: `crates/gpui/src/elements/anchored.rs` provides `anchored()` with `.position()`, `.anchor()`, `.snap_to_window()`, and `.position_mode()`. `crates/gpui/src/elements/deferred.rs` provides `deferred()` with `.priority()` for paint ordering. Both are core GPUI elements available in `gpui-unofficial`.

Zed's approach (crates/ui/src/components/dropdown_menu.rs + popover_menu.rs + context_menu.rs) handles focus management, keyboard navigation, submenus, and anchor positioning — overkill for our 2-6 profile options. A simple stateful div-based dropdown is sufficient and keeps the implementation lightweight.

**Alternatives considered:**
- Importing Zed's `ui` crate — would pull in dozens of transitive dependencies, icon sets, and theme infrastructure not needed for Bludio
- Inline clickable text for each profile option — doesn't scale beyond 2-3 options, clutters the row
- Radio buttons — same space concern as inline text
- `PopoverMenu` in gpui-unofficial — gpui has no built-in popover; Zed builds its own on top of GPUI primitives

### Decision 7: Module structure

**Choice — Audio backend:**
- `src/audio/mod.rs` — `AudioState`, `SinkInfo`, `SourceInfo`, `CardInfo`, `ProfileInfo` data types, `AudioCommand` enum, `DeviceKind` enum
- `src/audio/pulse.rs` — PulseAudio thread, connection, listing, subscription, command execution

**Choice — UI components:**
- `src/ui/components/mod.rs` — Reusable component module declarations
- `src/ui/components/volume_slider.rs` — `volume_bar()` free function (clickable volume bar)
- `src/ui/components/text_field.rs` — `TextField` entity with `EntityInputHandler` for numeric text input
- `src/ui/components/dropdown.rs` — `Dropdown` entity with `deferred(anchored(...))` floating menu for profile selection

**Choice — Audio page (entity-based):**
- `src/ui/audio/mod.rs` — Audio page module declarations
- `src/ui/audio/audio_page.rs` — `AudioPage` entity: owns a list of `Entity<AudioDeviceRow>` and coordinates state sync from `AudioState`
- `src/ui/audio/device_row.rs` — `AudioDeviceRow` entity: one row per device, owns its own `TextField` and `Dropdown` entities, renders volume bar + controls inline

**Rationale:** The audio backend follows the `src/bluetooth/` module structure (types in dedicated files, backend logic split by concern). The UI uses GPUI entities (`AudioPage`, `AudioDeviceRow`) rather than free functions — the entity pattern provides proper ownership of child entities (`TextField`, `Dropdown`) and their subscriptions, which the original free-function design could not easily handle. UI components that are reusable across pages live in `src/ui/components/`. Audio-specific pages live in `src/ui/audio/`.

## Risks / Trade-offs

- **[Risk] `libpulse-binding` may have breaking changes or API gaps.** → **Mitigation:** Pinning a specific version in `Cargo.toml`. The crate is stable and widely used (pavucontrol-rs, pulsectl, etc.).

- **[Risk] PA thread panic kills audio functionality silently.** → **Mitigation:** Catch panics in the thread with `std::panic::catch_unwind`, send error state back to UI via the channel. Implement reconnection logic if the PA context disconnects.

- **[Risk] `libpulse-dev` must be installed at build time.** → **Mitigation:** Document the system dependency. This is standard for Rust projects using PulseAudio (same as GTK/LV2/etc. requiring dev headers). Consider a feature flag to make audio support optional in the future.

- **[Trade-off] Profile switching affects ALL sinks/sources on the card.** Changing a card profile (e.g., A2DP → HSP/HFP) transforms both output and input simultaneously. This is inherent to Bluetooth audio profiles and matches pavucontrol behavior. Users need to understand this coupling.

- **[Trade-off] No reconnection on PA restart.** If PulseAudio/PipeWire restarts, the PA context becomes invalid. The current design doesn't handle reconnection — the user must restart Bludio. Acceptable for initial implementation; reconnect logic can be added later.

- **[Trade-off] Volume bar is click-to-set, not drag.** Users can't smoothly drag the volume slider. Acceptable for functional MVP — can be upgraded to a full drag slider in a follow-up change.

## Open Questions

_(None — all questions resolved.)

## GPUI Research Reference

This section consolidates findings from the Zed editor codebase. The implementing AI should not need to re-search Zed for these patterns.

### Floating elements: `deferred()` + `anchored()`

GPUI provides two core primitives for rendering elements that float above the normal layout flow. Both are in `gpui-unofficial` since they're core GPUI, not Zed's UI crate.

**`deferred()`** — `crates/gpui/src/elements/deferred.rs`
```rust
// Renders child in a separate draw pass, painting on top of regular content.
// Higher priority = drawn on top of lower-priority deferred elements.
deferred(options_list).priority(10)

// Key API:
// - deferred(child: impl IntoElement) -> Deferred
// - .priority(usize) — stacking order (higher = on top)
// - .with_priority(usize) — same, different name
```

**`anchored()`** — `crates/gpui/src/elements/anchored.rs`
```rust
// Positions children at a fixed coordinate relative to window or parent.
// Does NOT participate in parent flex flow — won't push content down.
// Auto-fits to stay within window bounds.
anchored()
    .position(point(x_px, y_px))          // window coordinates of trigger
    .anchor(Anchor::BottomLeft)            // which corner attaches to position
    .position_mode(AnchoredPositionMode::Window)  // relative to window
    .snap_to_window()                      // clamp to stay on-screen
    .child(options_list)

// Anchor variants: TopLeft, TopRight, BottomLeft, BottomRight, etc.
// FitMode: SwitchAnchor (default, flips side), SnapToWindow (clamps)
```

**Combined pattern for floating dropdown:**
```rust
.when(open, |el| el.child(
    deferred(
        anchored()
            .position(trigger_screen_pos)
            .anchor(Anchor::BottomLeft)
            .snap_to_window()
            .child(v_flex().children(options))
    ).priority(10)
))
```

The trigger stays inline in the device row. The options list floats *above* content at the trigger's screen position. No layout shift.

### Text input approaches

**Our approach — full `EntityInputHandler`-based `TextField` entity** (`src/ui/components/text_field.rs`, ~550 lines)

During implementation the simplified div-based approach proved insufficient. The implementation uses a complete `EntityInputHandler`-based `TextField` entity with:
- `EntityInputHandler` trait impl for IME and platform text input
- Custom `Element` with `request_layout`/`prepaint`/`paint` for text rendering, selection, and blinking cursor
- Clipboard support (copy/cut/paste via Ctrl+C/V/X)
- Cursor movement (Home, End, Ctrl+A, Shift+arrows for selection)
- Blinking cursor with epoch-based timer management
- Emits `TextFieldEvent::Confirmed(String)` on Enter and `TextFieldEvent::Cancelled` on Escape
- `filter_char()` for restricting input (digits only for volume)

The entity is reusable — it's also used in the `TextFieldTestPage` for debugging UI.

### Focus management (via TextField entity)

The `TextField` entity handles focus internally — no external focus management needed for text input:
- TextField acquires focus via mouse click or `window.focus(&focus_handle, cx)`
- Blinking cursor starts/stops based on focus edge detection (tracked via `was_focused`)
- `EntityInputHandler` routes platform text input to the focused field
- The parent (`AudioDeviceRow`) subscribes to `TextFieldEvent`s via `cx.subscribe()`

For custom non-TextField focus needs:
- `cx.focus_handle()` — creates a `FocusHandle` for the current entity
- `div().track_focus(&handle)` — makes a div participate in focus tracking
- `window.focus(&handle, cx)` — programmatically focus an element
- `handle.is_focused(window)` — check if currently focused

### Zed component references (for patterns, NOT importable)

These are in Zed's `ui` crate which we can't import, but their implementation patterns inform our simplified versions:

| Component | Zed file | Our equivalent |
|---|---|---|
| `DropdownMenu` | `crates/ui/src/components/dropdown_menu.rs` | Stateful div + `deferred(anchored(...))` |
| `PopoverMenu` | `crates/ui/src/components/popover_menu.rs` | Not needed; `anchored()` handles positioning |
| `ContextMenu` | (context_menu.rs) | `v_flex()` with clickable option divs |
| `Disclosure` | `crates/ui/src/components/disclosure.rs` | Simple chevron icon toggle (expand/collapse) |
| `Button`/`ButtonLike` | `crates/ui/src/components/button/` | Our own `action_btn()` pattern from `bluetooth_page.rs` |

### Event handling summary

```rust
// Mouse clicks
div().on_mouse_up(MouseButton::Left, cx.listener(|this, event: &MouseUpEvent, window, cx| { ... }))

// Keyboard
div().on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| { ... }))
// event.keystroke.key: String like "a", "backspace", "enter", "escape", "0"–"9"

// Focus
div().track_focus(&focus_handle)
div().on_blur(cx.listener(|this, window, cx| { ... }))  // focus lost

// Hover
div().hover(|el| el.bg(hover_color))
div().cursor(CursorStyle::PointingHand)
```
