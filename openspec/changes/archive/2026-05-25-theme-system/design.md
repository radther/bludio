## Context

Bludio currently hardcodes colors and font values directly in each UI component's render method. For example:

- `hsla(0.0, 0.0, 0.14, 1.0)` is repeated across multiple files for surface/header backgrounds
- `hsla(210.0/360.0, 0.7, 0.55, 1.0)` is the accent color, repeated everywhere
- `hsla(0.0, 0.0, 0.6, 1.0)` is the secondary text color
- Font weights like `FontWeight::BOLD`, `FontWeight::MEDIUM` are applied ad-hoc

This creates maintenance issues: changing a color means finding every usage, inconsistencies creep in, and there's no way to switch between light/dark modes.

**Reference implementations:**

- **Zed's theme system** (`crates/theme/`): `ThemeColors` struct with ~100 semantic color fields, a `GlobalTheme` stored as a GPUI global, accessed via `cx.theme()`. Zed uses `window.text_style()` (a resolved `TextStyle` struct) for font context. Very heavyweight — includes syntax highlighting, editor-specific colors, icon themes, etc.

- **rss-reader example** (`gpui_test/crates/rss-reader/`): A simple `Theme` struct with ~15 color fields + font configuration. Passed through as `&Theme` to every render. Light theme via `Theme::light()`. Uses "Noto Sans" font.

**Bludio's approach** should be between these two extremes: more structured than rss-reader's flat struct, but much simpler than Zed's full theme engine.

## Goals / Non-Goals

**Goals:**
- Centralize all colors into a `ThemeColors` struct with semantic names
- Centralize font configuration (family, weight scale, size scale) in the theme
- Provide a `Theme` struct holding colors + font config + dark/light mode flag
- Store the active theme as a GPUI global, accessible from any `Window` or `App` context
- Provide two presets: dark (current look) and light (warm light variant)
- Provide a `text_style(style)` convenience method on `Styled` that applies font family, weight, and size in one call
- Convert all existing hardcoded values to theme references
- Demo theme switching from `DevTestPage`

**Non-Goals:**
- JSON theme serialization/deserialization (future work)
- User-customizable themes (future work)
- Icon theming
- Syntax highlighting themes (not applicable)
- Re-styling components beyond color/font migration (e.g., layout changes, new visual design)
- Hot-reloading themes from files

## Decisions

### Decision 1: Global theme via `cx.set_global` + convenience accessors

**Choice:** Store `Arc<Theme>` as a GPUI global (`GlobalTheme` struct), with a `pub(crate) fn theme(cx: &App) -> &Arc<Theme>` free function in `theme.rs`.

**Alternatives considered:**
- *Pass `&Theme` through render args* (like rss-reader): Simple but requires threading through every render call and every component constructor. Violates the existing pattern where components don't receive a theme parameter.
- *Store in a `static`/`LazyLock`*: Works but can't be swapped at runtime without unsafe code or atomics.

**Rationale:** GPUI globals are the standard pattern (Zed uses `GlobalTheme`), they support runtime mutation via `cx.set_global()`, and they're accessible from any context without threading. The `Arc` wrapper allows cheap cloning when needed for async contexts.

### Decision 2: Theme struct structure

**Choice:** A `Theme` struct with appearance, font family, colors, and text styles:

```rust
pub struct Theme {
    pub appearance: Appearance,  // Light or Dark
    pub font_family: SharedString,  // "Noto Sans"
    pub colors: ThemeColors,
    pub text_styles: TextStyleSet,
}
```

`ThemeColors` has 24 fields covering background levels, text levels, accent, status colors, borders. `TextStyleSet` has named fields (`body`, `heading`, `body_small`, `caption`) each mapping to a `(size, weight)` tuple. `FontConfig` was initially planned but removed as redundant — `font_family` is set on the root element via GPUI's cascade, and text size/weight live in `TextStyleSet`.

**Rationale:** Each concern has a clear home. The nested access pattern (`theme.colors.surface`, `theme.text_styles.heading`) is self-documenting and symmetric.

### Decision 3: TextStyle convenience method

**Choice:** A `TextStyleSet` struct with named fields (`body`, `heading`, `body_small`, `caption`) each holding a `(AbsoluteLength, FontWeight)` tuple. An extension trait method `.styled(tuple)` on any `Styled` element applies both size and weight in one call. Font family is inherited from the root element via GPUI's cascade — `.styled()` only sets size and weight.

Call sites follow the same pattern as colors:
```rust
let colors = &theme.colors;
let text_styles = &theme.text_styles;
div().bg(colors.surface).styled(text_styles.heading)
```

**How Zed handles this:** Zed uses `window.text_style()` which returns a resolved `TextStyle` struct, and `window.with_text_style()` for scoped overrides. Bludio's approach is simpler — direct field access from the theme, no cascading inheritance needed.

**Rationale:** The `TextStyleSet` lives in `Theme`, so different themes can define different size scales and weight mappings. The field-access pattern is symmetric with `ThemeColors`, making the API predictable. An earlier design used a `TextStyle` enum with `.text_style(Body)` syntax, but this clashed with GPUI's built-in `text_style()` method and required hardcoded values that couldn't vary per theme.

### Decision 4: Font delivery

**Choice:** Bundle the Noto Sans **variable font** (VF) with the application. Copy `NotoSans.ttf` and `NotoSans-Italic.ttf` from `~/.local/share/fonts/noto-vf/` into a `fonts/` directory at project root, embed with `include_bytes!`, and load at startup via `cx.text_system().add_fonts()`.

**Alternatives considered:**
- *System font*: Unpredictable across Linux distros; defeats the purpose of consistent theming.
- *Download at runtime*: Adds network dependency; undesirable for a system utility.
- *Individual weight files (Regular, Medium, SemiBold, Bold)*: Larger total size (~2MB for 4 files) vs single VF (~2MB for normal, ~2.3MB for italic). The VF provides infinite weight/font-style variation, matching the flexibility of the rss-reader example which uses `FontWeight::EXTRA_BOLD` etc.

**Rationale:** The variable font covers all weights and both upright/italic styles in two files, matching "Noto Sans" usage from the rss-reader example. Using `include_bytes!` embeds the fonts directly in the binary — no runtime file I/O and font is always available.

### Decision 5: Interactive component styling

**Choice:** Interactive components read theme colors at render time rather than storing them. The Dropdown component reads from the global theme in its `RenderOnce::render()` method — no stored color fields, no builder methods for overrides. The Slider (which is an ephemeral `RenderOnce` reconstructed each frame) stores colors from the theme at construction time in `Slider::render()`, but reads fresh values each frame.

**Rationale:** Reading from the global at render time eliminates stale-state bugs when the theme switches. The Dropdown's previous approach of storing defaults in `new()` caused it to show dark-theme colors even after switching to light mode. The builder API (`accent()`, `bg()`, etc.) was removed as unnecessary — components that need per-instance color overrides can be revisited when that use case arises.

### Decision 6: Theme switching mechanism

**Choice:** `DevTestPage` gets two buttons ("Dark" / "Light") that call `set_theme(Theme::dark(), cx)` or `set_theme(Theme::light(), cx)`. `set_theme()` calls `cx.update_global()`, which triggers automatic UI re-render via GPUI's global observation — no explicit `cx.notify()` or `cx.refresh_windows()` needed.

**Rationale:** This matches Zed's approach where updating the theme global is sufficient. GPUI detects the global change and re-renders all windows. No persistence, no settings file, no OS integration (those are non-goals).

## Risks / Trade-offs

- **[Risk] Theme change doesn't propagate to all elements** → If `update_global` doesn't trigger re-renders for some children, stale colors could linger. **Mitigation:** GPUI's global observation system walks the entity tree when a global changes, so all windows re-render. This was verified by testing the Dark/Light toggle on DevTestPage — all pages update correctly. If gaps appear with future component types, per-entity `cx.notify()` can be added as a fallback.

- **[Risk] Font bundling increases binary size** → Noto Sans VF normal is ~2MB, italic ~2.3MB — total ~4.3MB. **Mitigation:** Acceptable for a desktop app. If size becomes an issue, subset to Latin-1 (VF subsetting brings it under 1MB total).

- **[Trade-off] ThemeColors has fewer fields than Zed** → We might miss a color that's needed later. **Mitigation:** We can always add fields; the struct is extensible. Starting lean avoids premature abstraction.
