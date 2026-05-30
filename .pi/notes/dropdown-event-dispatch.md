# Dropdown Toggle vs Click-Outside: Zed's Approach vs Ours

> Written: 2026-05-30  
> Context: We needed clicking the trigger button to toggle the dropdown closed (in addition to escape and click-outside). Our original implementation had a race where the click-outside handler closed the dropdown and the trigger's `on_click` immediately reopened it.

---

## The Problem

GPUI's event dispatch during the **bubble phase** iterates listeners in **reverse registration order**:

```rust
// From zed/crates/gpui/src/window.rs
for listener in mouse_listeners.iter_mut().rev() {
    listener(event, DispatchPhase::Bubble, self, cx);
    if !cx.propagate_event { break; }
}
```

Our original dropdown had a single `absolute().size_full()` canvas registering a `MouseUpEvent` handler **after** the trigger button in the DOM. During paint, the trigger's `on_click` registered first in the listeners vec, and the canvas handler registered later. During bubble-phase reverse iteration:

1. **Canvas handler fires first** → `is_open = false`
2. **Trigger `on_click` fires second** → toggles back to `true`

Clicking the trigger to close would briefly close the dropdown then reopen it in the same frame.

---

## Zed's Approach (`PopoverMenu` / `ContextMenu`)

Zed's approach is architecturally different from ours in a few key ways.

### 1. Zed uses a custom `Element`, not `RenderOnce`

`PopoverMenu<M>` implements GPUI's `Element` trait directly, giving it full access to the three-phase lifecycle:

1. **`request_layout`** — builds trigger + menu children
2. **`prepaint`** — captures the trigger's `child_hitbox` (stored in element state)
3. **`paint`** — registers a `MouseDownEvent` handler that knows the trigger's exact hitbox

Because it's a custom `Element`, Zed can paint the trigger first, then the menu, then register a handler that references the trigger's hitbox from prepaint. Our `RenderOnce` doesn't have access to hitboxes or the paint-phase registration order in the same way.

### 2. Zed uses focus loss (blur) for outside dismissal

The `ContextMenu` inside Zed's popover takes focus when opened via `window.focus(&focus_handle, cx)`. The menu entity subscribes to `on_blur` — when anything outside takes focus, the menu emits `DismissEvent` and closes. No manual click-outside handler is needed for general "click elsewhere" dismissal.

From `../zed/crates/ui/src/components/context_menu.rs`:

```rust
let _on_blur_subscription = cx.on_blur(
    &focus_handle,
    window,
    |this: &mut ContextMenu, window, cx| {
        if let Some(ignore_until) = this.ignore_blur_until {
            if Instant::now() < ignore_until {
                return;
            } else {
                this.ignore_blur_until = None;
            }
        }

        if this.main_menu.is_none() {
            if let SubmenuState::Open(open_submenu) = &this.submenu_state {
                let submenu_focus = open_submenu.entity.read(cx).focus_handle.clone();
                if submenu_focus.contains_focused(window, cx) {
                    return;
                }
            }
        }

        this.cancel(&menu::Cancel, window, cx)
    },
);
```

### 3. Zed intercepts `MouseDown` (not `MouseUp`) on the trigger

This is the key to their toggle behavior. In `PopoverMenu::paint`, after painting the menu, Zed registers:

```rust
// From ../zed/crates/ui/src/components/popover_menu.rs
window.on_mouse_event(move |_: &MouseDownEvent, phase, window, cx| {
    if phase == DispatchPhase::Bubble && child_hitbox.is_hovered(window) {
        if let Some(menu) = menu_handle.borrow().as_ref() {
            menu.update(cx, |_, cx| cx.emit(DismissEvent)); // close menu
        }
        cx.stop_propagation(); // ← crucial
    }
})
```

#### Event dispatch order during a trigger click (while menu is open)

1. **MouseDown**: PopoverMenu's handler fires first (reverse bubble order — it registered last during menu paint). It sees the click is on the trigger, dismisses the menu, and calls `cx.stop_propagation()`.
2. The trigger button's `MouseDownEvent` handler never fires because propagation was stopped.
3. **MouseUp**: The trigger button's `MouseUpEvent` handler runs capture phase, but `pending_mouse_down` was never set (step 2 was skipped). During bubble phase, no `ClickEvent` is generated.
4. Result: the menu closes and **stays** closed. The trigger's `on_click` never runs.

When the menu is closed and you click the trigger: the PopoverMenu's `MouseDownEvent` handler fires but `menu_handle.borrow()` is `None`, so it's a no-op. Then the trigger's `on_click` fires normally and calls `show_menu()` to open it.

### 4. Why this works for Zed

- **Custom `Element`** gives hitbox access during `prepaint` (stored as `child_hitbox`)
- **`paint` phase** registration means the menu paint happens *after* the trigger paint, so the menu's handler registers *later* in the listeners vec — ensuring it fires *first* during bubble reverse iteration
- **`cx.stop_propagation()`** prevents the trigger's own handlers from running
- **Focus loss** handles general "click elsewhere" dismissal without needing manual bounds checks

---

## Our Approach (`RenderOnce` workaround)

Our dropdown is a simpler `RenderOnce` (not a custom `Element`). We don't have hitbox access or precise paint-phase control over listener registration order.

### The two-canvas solution

We split what was one canvas into **two**, with carefully ordered DOM placement:

| Canvas | DOM Placement | Size | Job |
|---|---|---|---|
| Click-outside | **First** child | `absolute()` (zero size) | Registers `MouseUpEvent` handler **before** trigger button paints |
| Bounds capture | **Last** child | `absolute().size_full()` | Captures full outer `div()` bounds **after** all siblings laid out |

```rust
// Simplified structure
div()
    .child(click_outside_canvas)   // registers handler FIRST
    .child(trigger_button)          // registers on_click SECOND (later)
    .child(menu)                    // deferred, third
    .child(bounds_capture_canvas)   // captures stable bounds LAST
```

### Why this works

During paint, listener registration order in the vec:

- Index 0: click-outside handler (registered first)
- Index 1: trigger `on_click` (registered later)

During bubble-phase reverse iteration:

1. **Trigger `on_click` fires first** → toggles `is_open = false`
2. **Click-outside handler fires second** → sees `this.is_open == false`, does nothing

When clicking genuinely outside the trigger, the trigger's `on_click` never fires (no hitbox), so only the click-outside handler runs and correctly closes the dropdown.

### Why bounds capture is at the end

If the bounds-capturing canvas lives inside the trigger button (as we briefly tried), it captures the trigger's smaller bounds. The outer `div()` bounds include padding/siblings that affect menu positioning. Moving it to the end ensures it captures the stable outer container bounds after all siblings are laid out.

---

## Key Differences

| Aspect | Zed (`PopoverMenu`) | Ours (`Dropdown`) |
|---|---|---|
| GPUI primitive | Custom `Element` | `RenderOnce` |
| Hitbox access | Yes, from `prepaint` | No |
| Trigger dismissal | `MouseDownEvent` + `stop_propagation()` | `MouseUpEvent` + registration order |
| "Click elsewhere" dismissal | Focus loss (`on_blur`) | Manual `MouseUpEvent` handler on canvas |
| Handler ordering control | Paint-phase registration | DOM ordering of canvas elements |
| Complexity | Higher (full Element impl) | Lower (RenderOnce builder) |

---

## If We Wanted to Match Zed

To adopt Zed's approach, we would need to:

1. **Convert `DropdownComponent` to a custom `Element`** implementing `request_layout`, `prepaint`, and `paint`
2. **In `prepaint`**: capture the trigger button's hitbox ID (or bounds) from element state
3. **In `paint`**: after painting the menu, register a `MouseDownEvent` handler that checks `child_hitbox.is_hovered(window)` and calls `cx.stop_propagation()`
4. **For "click elsewhere"**: either use focus-based dismissal (have the dropdown take focus and use `on_blur`) or keep the canvas-based approach

Alternatively, we could keep our `RenderOnce` but use `window.on_mouse_event` with `DispatchPhase::Capture` to intercept the event before the trigger's bubble-phase handlers run. However, capture-phase handlers run in forward order, and the trigger's own handlers also register capture-phase listeners, making the interaction harder to control without hitbox access.

---

## File References

- Our dropdown: `src/ui/components/dropdown.rs`
- Zed's PopoverMenu: `../zed/crates/ui/src/components/popover_menu.rs`
- Zed's ContextMenu: `../zed/crates/ui/src/components/context_menu.rs`
- Zed's ButtonLike (on_click + stop_propagation): `../zed/crates/ui/src/components/button/button_like.rs`
- GPUI mouse dispatch: `../zed/crates/gpui/src/window.rs` (bubble phase reverse iteration)
- GPUI div click handling: `../zed/crates/gpui/src/elements/div.rs` (MouseDown/MouseUp pairing for ClickEvent)
