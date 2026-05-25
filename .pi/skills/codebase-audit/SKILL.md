---
name: codebase-audit
description: Run an in-depth codebase audit. Full scan or compare (diff against main). Produces a severity-ranked report with issues and fix proposals.
license: MIT
compatibility: Bludio project
metadata:
  author: bludio
  version: "1.0"
---

Run a comprehensive audit of the Bludio codebase, producing an in-depth report of issues broken down by severity level, each with a description of why it's flagged, and a proposal for how to fix it.

**Input**: Optionally specify a mode — "full" for a complete scan, "compare" to audit only the diff between the current branch and `main`. If omitted, defaults to "full".

**Output**: A single markdown document detailing the audit results, with YAML frontmatter. The output is written to `audit-x-YYYY-MM-DD.md` in `./openspec/audit/` where `x` is the next audit iteration in the sequence, e.g. `audit-5-YYYY-MM-DD.md`.

---

## Modes

### Full Scan

Audit every `*.rs` file under `src/`. Run static analysis tools first, then walk through each audit dimension (sections A–E below) against the entire codebase.

### Compare Mode

Run `git diff main...HEAD -- '*.rs'` and audit only the changed code. Focus on issues introduced by the diff — regressions, new violations, broken patterns. Cross-reference unchanged code only when needed to establish "this breaks a pattern used everywhere else." Start with:

```bash
git diff main...HEAD -- '*.rs'
```

Then read the changed files and audit them against the dimensions below.

---

## Before You Start

1. **Read `AGENTS.md`** — it documents the project's conventions, known limitations, and patterns. Treat it as guidance, not gospel. A pattern being documented doesn't make it immune to scrutiny — if a convention outlined in AGENTS.md is causing real problems (bloat, bugs, confusing code), flag it as a NOTE or SUGGESTION and propose updating AGENTS.md as part of the fix.

2. **Run static analysis** — gather data first, don't block on it:
   ```bash
   cargo clippy -- -D warnings 2>&1 || true
   cargo fmt --check 2>&1 || true
   ```
   Include the raw output at the bottom of the report.

3. **Locate the reference codebases** — ground truth for correct patterns:
   - `../zed/` — primary source for GPUI usage. The way Zed does something IS the GPUI standard pattern.
   - `../gpui-component/` — common GPUI components. Reference when Zed doesn't have an equivalent.

---

## Audit Dimensions

Each dimension covers a specific category of concern. For each, inspect the code and flag issues.

### A. Rust Best Practices & Safety

Check for:

- **Unsafe code** — every `unsafe` block must have a doc comment explaining why it's sound. In the PA backend (`pulse.rs`), `unsafe impl Send for PaWakeup` and the `pa_mainloop_wakeup` call MUST be justified. Flag any new unsafe without documentation, or existing unsafe whose justification is unclear or missing.

- **Naming conventions** —
  - Modules/files: `snake_case`
  - Functions: `snake_case`
  - Structs/enums: `PascalCase`
  - Section comments: `// ── Section Name ──` (Unicode box-drawing)
  - Module doc comments: file-level only (`//!`)

- **Separation of concerns** —
  - `audio/` handles PulseAudio (data types + PA thread)
  - `bluetooth/` handles BlueZ D-Bus (agent, device, discovery, monitor, properties)
  - `ui/` handles all UI (pages, components, theme)
  - No cross-contamination — bluetooth code must not import UI types, UI must not reach into PA internals.

- **Error handling** —
  - Flag `let _ =` that swallows errors without at least an `eprintln!`
  - Flag `.unwrap()` / `.expect()` in non-init code (runtime panics)
  - The `crate::tokio_task` pattern returns `oneshot::Receiver<T>` — callers must handle both the `RecvError` (task cancelled) and the inner `Result` from the Tokio task. Missing either arm is a bug.
  - `Result` must be propagated with `?` or handled explicitly.
  - `.ok()` usage should have a comment or be obviously intentional.

- **Clippy attributes** —
  - `#[allow(clippy::...)]` must have an explanatory comment.
  - `AudioCommand`'s `#[allow(clippy::enum_variant_names)]` — intentional (the `Set` prefix maps to PA operations). Ensure the comment is present.
  - `volume_f64_to_pa`'s `#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]` — intentional (PA volumes fit in u32). Ensure comment.
  - Flag any clippy allow that lacks an explanatory comment.

### B. GPUI Usage (Zed as Reference)

Check against both `../zed/` (primary) and `../gpui-component/` (secondary):

- **Layout conventions** —
  - Use `h_flex()` / `v_flex()` free functions, never raw `div().flex().flex_row()`
  - `h_flex()` defaults to `items_center()` — don't add it redundantly
  - Override with `.items_start()` or `.items_stretch()` when needed
  - `v_flex()` has no items alignment default — add `.items_center()` etc. as needed

- **Entity ownership** —
  - Entities own their children; app never stores a grandchild's entity
  - Page manages rows, not individual text fields
  - Each level creates its children, passes down clones (`cmd_tx`, `PaWakeup`), and subscribes via `cx.subscribe_in()`
  - Never drill through multiple entity levels

- **Window passing** —
  - Async tasks / subscriptions needing `&mut Window` must use `*_in` APIs: `cx.spawn_in()`, `cx.subscribe_in()`, `this.update_in()`
  - Use `cx.subscribe()` (non-`_in`) only when window access isn't needed
  - Never use boolean flags as workarounds for window access

- **EventEmitter** —
  - Child→parent: `cx.emit()` + `cx.subscribe_in()` / `cx.subscribe()`
  - No custom `drain_events()` polling or `on_event()` callbacks
  - All entities that emit events must impl `EventEmitter<EventType>`
  - Every emit must have a corresponding subscribe on the parent side

- **Component interactivity** —
  - Every interactive element MUST have `.id()`. Without it, hover, click states, and rendering won't behave correctly.
  - `cx.notify()` must be called after state changes to trigger re-renders.
  - Flag interactive elements missing an id, and state changes lacking `cx.notify()`.

- **Return types** —
  - Builder chains with `.id()`, `.hover()`, `.on_*()` return `Stateful<Div>`
  - Pure styling functions return `Div`
  - Trait impls (`Render::render`) keep `impl IntoElement`
  - Flag functions that advertise a return type that doesn't match what the builder chain actually produces.

- **RenderOnce pattern** —
  - For complex rendering (TextField, Dropdown, Slider): Entity → RenderOnce → IntoElement
  - Entity holds state + logic; RenderOnce handles canvas/paint
  - Compare with `../gpui-component/` components for reference
  - Flag complex rendering logic embedded directly in `Render::render` that should be extracted into a `RenderOnce`.

- **Canvas usage** —
  - `canvas(prepare_fn, paint_fn)` for text rendering, bounds capture
  - Text paint goes through `window.text_system().shape_line()` (not raw skia)
  - Flag direct skia usage when `shape_line` would suffice.

- **Theme access** —
  - `crate::ui::theme::theme(cx)` to get current theme (never hardcode colors)
  - Global theme stored as `Arc<Theme>` via `cx.set_global(GlobalTheme::new(...))`
  - Async tasks that need theme values must clone the `Arc` before spawning
  - Flag hardcoded colors and `theme(cx)` inside async blocks (lifetime issues).

- **Shared constructors** —
  - Two constructors sharing logic → private `new_impl` with params struct
  - Example: `AudioDeviceRow::new_sink` / `new_source` → `new_impl(RowParams { ... })`
  - Flag duplicated constructor logic.

- **Focus handling** —
  - Entities expose their focus handle via `Focusable::focus_handle`
  - `window.focus(&handle, cx)` to programmatically transfer focus
  - `track_focus(&handle)` on containers to receive keyboard events
  - Flag focus-able elements that don't implement `Focusable`, or focus calls missing `_in` variants when inside async.

### C. Dead Code & Future-Proofing

Check for:

- **Dead code WITH justification** — intentional dead code is acceptable if documented: the `debug_bg_*` methods in `StyledExt` are debugging helpers. AGENTS.md documents `trusted` and `CardInfo::name` as "kept for future use." `#[allow(dead_code)]` is acceptable as long as a comment explains why it's kept. Flag `#[allow(dead_code)]` that lacks any justification comment.

- **Dead code WITHOUT justification** — candidates for removal: `#[allow(dead_code)]` on items that are never called and have no doc comment explaining why they exist. Functions, methods, enum variants, and struct fields that are declared but never referenced. Distinguish "intentionally kept" from "forgotten."

- **Subscription fields** — subscriptions (the `Subscription` type) must be stored in a field to stay alive — if dropped, they cancel immediately. The preferred convention is to name the field with a `_` prefix (e.g., `_tab_bar_sub`), which suppresses Rust's unused-field warning *without* needing `#[allow(dead_code)]`. The `_` prefix alone is sufficient and cleaner than adding the attribute.
  - Flag subscription fields that lack `_` and rely on `#[allow(dead_code)]` instead — rename them.
  - Flag entities that subscribe to child events but don't store the `Subscription` at all (silent bug — the subscription cancels immediately).

- **Unused function parameters** —
  - `_cx` / `_window` / `_app` prefix is only justified when the parameter is required by a trait or closure signature (e.g., `cx.new(|cx| ...)` forces a `Context` param even if unused).
  - Flag `_`-prefixed parameters in functions that could simply omit the parameter.
  - Flag unused parameters that lack the `_` prefix entirely.

- **"For future use" fields** —
  - Fields kept only with comments like "stored for potential future use" should be flagged. Add them when they're needed, not before. Carrying unnecessary state bloats the struct and confuses readers.

### D. General Programming Principles (Language-Agnostic Readability)

Check for:

- **File organization** — can a non-Rust programmer find the right file?
  - Module tree maps to features: `audio/` → sound, `bluetooth/` → bluetooth, `ui/` → visuals, `ui/components/` → reusable widgets
  - File names describe their content clearly
  - Flag any file whose name doesn't match what's inside it.

- **Naming clarity** —
  - Variable names that describe *what* not *how*
  - Type names that communicate intent
  - Function names that say what they do: `sync_rows`, `update_from_sink`, `resolve_display_name`
  - Flag names that are generic when they could be specific, or names that are misleading.

- **Comment quality** —
  - Module-level doc comments explain the module's purpose
  - Section comments (`// ──`) visually separate logical blocks
  - Inline comments for non-obvious logic
  - Flag: missing module doc comments, unexplained "magic" numbers, functions whose purpose is unclear from the name alone.

- **Control flow clarity** —
  - Long functions (> 50 lines) broken into named helpers
  - Error paths handled early (guard clauses) rather than deeply indented
  - Nested conditionals beyond 3 levels deep are a red flag
  - Flag functions that mix levels of abstraction or require holding too much state in your head.

- **Internal consistency** —
  - If a pattern exists in 3+ places, it should be applied consistently
  - Flag similar things done differently without a clear reason
  - Example: every page uses a command channel for child→app communication, but one directly mutates app state instead → inconsistent.
  - Example: every entity implements `Focusable`, but one doesn't → needs a documented reason.

### E. Bludio-Specific Patterns

These patterns exist in the current codebase. New code must follow them. Flag deviations.

- **Channel-based command dispatch** —
  - UI→backend goes through channels, not direct method calls.
  - Audio: `tokio::sync::mpsc::UnboundedSender<AudioCommand>` → PA thread
  - Bluetooth: `futures::channel::mpsc::UnboundedSender<BluetoothPageCommand>` → Tokio task handler
  - Volume commands use a HashMap dedup in `pulse.rs` to prevent backlog during rapid drag — this is intentional. Flag volume paths that bypass it.
  - Flag attempts to call backend functions directly from UI code.

- **GPUI + Tokio bridge** —
  - `crate::TOKIO` global runtime for all BlueZ D-Bus work
  - `crate::tokio_task()` → `oneshot::Receiver`
  - Pattern: `cx.spawn_in(window, async { TOKIO.enter(); tokio_task(...).await; ... }).detach()`
  - Flag: Tokio work spawned directly on GPUI's executor without `tokio_task()`
  - Flag: missing `TOKIO.enter()` before `tokio::sync::mpsc::recv()` inside GPUI async blocks.

- **Async update flow** —
  1. `cx.spawn_in(window, ...).detach()` — spawn with window access
  2. `TOKIO.enter()` — enter tokio context
  3. `tokio_task(...)` — ship work to tokio
  4. `rx.await` — receive
  5. `this.update_in(cx, |this, window, cx| { ...; cx.notify(); })` — apply
  - Flag: missing `cx.notify()` after state update in async callback
  - Flag: blocking the GPUI thread with `.await` outside of `cx.spawn`.

- **PA backend conventions** —
  - `Rc<RefCell<T>>` for mutable data shared with C callbacks (single-threaded)
  - `PaWakeup` is `Clone + Copy`, stored as `Option<PaWakeup>` when init may fail
  - `DoneFlag` pattern (`Rc<RefCell<bool>>` + `spin_until`) for sync callback completion — blocks the PA thread only, never GPUI
  - Flag: `Mutex` or `RwLock` where `RefCell` would suffice
  - Flag: any blocking on the GPUI thread.

- **Blueman-parity device filtering** —
  - Name resolution priority: broadcast name → alias → MAC fallback
  - Alias that equals the raw MAC address → treated as "no alias set"
  - Unpaired + no RSSI → hidden during scan
  - Flag changes to `resolve_display_name` or `build_device` that would cause the device list to behave differently from blueman.

- **Theme access pattern** —
  - Sync code: `crate::ui::theme::theme(cx)` borrows the `Arc` from global
  - Async code: clone the `Arc` before spawning — `let theme = cx.global::<GlobalTheme>().theme.clone();`
  - Flag calling `theme(cx)` (which borrows `App`) inside an async block.

- **Placeholder / TODO debt** —
  - Track TODO comments and placeholder implementations
  - Flag new TODOs without a tracking issue or plan
  - Flag stale TODOs that are clearly old and unaddressed (e.g., > 3 months with no action).

---

## Severity Levels

```
┌─────────────┬──────────────────────────────────────────────────────┐
│ CRITICAL    │ Unsafe without docs, memory safety, data loss,       │
│             │ crash-on-start, security issues. Fix immediately.    │
├─────────────┼──────────────────────────────────────────────────────┤
│ WARNING     │ Violates project convention, error-prone pattern,    │
│             │ missing cx.notify(), wrong API usage, logic bug.     │
├─────────────┼──────────────────────────────────────────────────────┤
│ SUGGESTION  │ Dead code to remove, redundant attributes, minor     │
│             │ style nits, cleanup opportunities.                   │
├─────────────┼──────────────────────────────────────────────────────┤
│ NOTE        │ Observations worth being aware of — tradeoffs,       │
│             │ design decisions, patterns to watch. Not issues.     │
├─────────────┼──────────────────────────────────────────────────────┤
│ POSITIVE    │ Things done well — patterns worth keeping and        │
│             │ replicating in new code.                             │
└─────────────┴──────────────────────────────────────────────────────┘
```

---

## Report Format

Produce the report as a single markdown document with YAML frontmatter. The report should be written to `audit-x-YYYY-MM-DD.md` in `./openspec/audit/` where `x` is the next audit iteration in the sequence, e.g. `audit-5-YYYY-MM-DD.md`.

```markdown
---
status: reported
date: YYYY-MM-DD
mode: full | compare (branch → main)
files_reviewed: N
tooling:
  clippy: clean | (output)
  fmt: clean | (output)
---

# Codebase Audit Report

(If the report is consolidated from multiple audit runs, add a `consolidated_from` field listing the source file paths.
If comparing against a specific base rather than main, add a `base` field.)

## Summary

X critical, Y warnings, Z suggestions. Brief one-paragraph overview.

## Critical Issues

### [CRITICAL-1] Issue title

**File:** `src/path/to/file.rs`
**Line:** 42 (or range: 42–58)
**What:** Description — what the code does and why it's wrong.
**Why:** The impact — what could go wrong, what principle it violates.
**Fix:** Concrete proposal. Replace X with Y. Add a comment. Remove this block.

(repeat for each critical)

## Warnings

(same format as Critical, with WARNING-N prefix)

## Suggestions

(same format, with SUGGESTION-N prefix)

## Notes

Observations, tradeoffs, patterns worth knowing about. Not actionable issues.

## Positive Findings

What's done well — specific modules, functions, or patterns that exemplify good practices.


```

---

## Operating Guidelines

- **Read before flagging.** Read the actual code at each flagged location before opening an issue. Don't pattern-match from memory or guess.
- **Documented intent is guidance, not immunity.** AGENTS.md documents patterns discovered so far — it's a living document. If a documented convention is actively worsening the codebase, flag it and propose updating AGENTS.md. Conventions that serve the project well should be reinforced; conventions that don't should be questioned. This audit skill itself is under the same scrutiny — if its dimensions are bloated, unclear, or flagging things that shouldn't be flagged, propose improvements to it as well.
- **Missing or stale justifications are always fair game.** If a code comment or `#[allow(...)]` explains a deviation, verify the explanation still holds. Flag if the justification is missing, stale, or wrong.
- **Propose concrete fixes.** Every issue must include a specific proposal: "replace X with Y", "add a comment here explaining Z", "store the Subscription or remove the subscribe call." Avoid vague notes.
- **Cite Zed for GPUI issues.** When flagging a GPUI usage problem, cite the specific Zed file or pattern that shows the correct way. This makes the report actionable and educational.
- **Positive findings matter.** The report should help the team know what's working well so new code can follow good examples. Be specific: "see how `AudioDeviceRow` handles XYZ via the `RowParams` pattern."
- **Compare mode stays scoped.** Only flag issues visible in the diff. You may reference unchanged code to support a "this breaks an established pattern" argument, but the issue itself must be in the changed code.
