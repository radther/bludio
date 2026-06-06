## Context

Bludio currently provides a Settings page with theme mode toggle and per-mode theme dropdowns. All view animations use a shared `FadeInAnimationExt` trait that adds `with_fade_in_up()` to styled elements. The bundled font is Noto Sans (variable TTF), loaded at startup via `AppContext::add_fonts()`. The active theme is cached in `GlobalSettings` and accessed via `theme(cx)`.

## Goals / Non-Goals

**Goals:**
- Add an "Accessibility" section to Settings with a global "Disable Animations" toggle.
- Rename the existing theme section to "Appearance" and add a Font dropdown.
- Bundle OpenDyslexic alongside the existing Noto Sans variable font.
- Add High Contrast Light and High Contrast Dark themes.
- Ensure all new settings are persisted and restored on startup.

**Non-Goals:**
- Per-view animation control (only global on/off).
- System font discovery or arbitrary user font loading.
- Modifying existing Rose Pine theme colors.
- Adding additional accessibility features beyond animation, font, and contrast.

## Decisions

### 1. Animation suppression in `FadeInAnimationExt`
**Decision:** When `disable_animations` is true, `with_fade_in_up` returns the unanimated element instead of an `AnimationElement`.

**Rationale:** Centralizing the check in the animation extension means no view code needs to change. If we instead gated every call site, we'd have to touch every page.

**Trade-off:** We need to handle the return-type difference. The trait currently returns `AnimationElement<Self>`. Options:
- **A.** Keep the same return type but make the animation instantaneous (0ms). This preserves the type but the element still wraps in animation machinery.
- **B.** Change the return type to `impl IntoElement` or `Div`. This is a breaking API change but cleaner.
- **C.** Store `disable_animations` in a new GPUI global (`GlobalAccessibility`) and read it inside the trait method. To avoid changing the return type, we can return `AnimationElement` with a 0ms duration, which effectively skips the visual animation while keeping types intact.

**Chosen:** Option C — return `AnimationElement` with `Duration::ZERO` when disabled. The element still animates, but the effect is immediate. This avoids cascading type changes through every page.

### 2. Font storage and switching
**Decision:** Add `font_family: String` to `Settings`, default `"Noto Sans"`. The root element in `BludioApp::render` applies `.font_family(theme.font_family.clone())`.

**Rationale:** GPUI's `font_family` is inherited down the tree, so setting it once at the root is sufficient. Storing the font in `Settings` means it persists naturally via the existing `update_settings()` path.

**Alternative considered:** Store font family on the `Theme` struct. Rejected because the font is a user preference, not an intrinsic theme property — a user might want OpenDyslexic with any theme.

### 3. OpenDyslexic bundling
**Decision:** Download all four OpenDyslexic style variants (`Regular`, `Bold`, `Italic`, `Bold-Italic`), place them in `fonts/`, and load all four via `include_bytes!` in `main.rs` alongside Noto Sans.

**Rationale:** OpenDyslexic does not ship as a variable font. To preserve the font's readability features at all weights/styles — especially bold, where glyph shapes are tuned for readability — we bundle the actual style files rather than relying on GPUI's synthetic bold/italic. This matches how we already handle Noto Sans (which *is* a variable font covering all weights in two files).

### 4. High-contrast theme design
**Decision:** Create two new theme files (`high_contrast_light.rs`, `high_contrast_dark.rs`) using the same `Theme` struct but with a palette that maximizes contrast ratios (e.g., pure black/white or near-pure, saturated primary colors for accents).

**Rationale:** High contrast is about maximizing luminance difference, not just inverting colors. We'll use `#000000` / `#FFFFFF` backgrounds with `#0000FF` / `#FFFF00` accents (following WCAG high-contrast patterns).

### 5. Section headers in Settings page
**Decision:** Use a styled `div()` with the `heading` text style as a section header, placed before each group of controls. The existing controls (theme mode, light/dark dropdowns) are grouped under an "Appearance" header. New controls (disable animations) go under an "Accessibility" header.

**Rationale:** This is purely presentational and requires no new components. It matches the user's request for visual distinction.

## Risks / Trade-offs

- **[Risk]** OpenDyslexic licensing must be verified before bundling. → **Mitigation:** Use the official OFL-licensed release; include the license file in `fonts/`.
- **[Risk]** 0ms animation may still trigger GPUI animation bookkeeping. → **Mitigation:** Acceptable overhead; if it causes issues, we can revisit and change the return type.
- **[Risk]** High-contrast themes may look harsh next to the aesthetic Rose Pine themes. → **Mitigation:** This is intentional — high contrast is for accessibility, not aesthetics.
- **[Risk]** Settings file schema change means old settings files won't have the new fields. → **Mitigation:** `serde` will default missing fields via `#[serde(default)]` or `Default` impl.

## Open Questions

1. Should `disable_animations` also suppress other future animations (e.g., page transitions), or only fade-ins? — **Answer:** Any animation using `FadeInAnimationExt` will be suppressed. New animation patterns should be added to the same gating mechanism.
2. Does OpenDyslexic need both regular and bold files, or does GPUI synthesize bold from a single regular file? — **Answer:** Bundle all four style variants (Regular, Bold, Italic, Bold-Italic). OpenDyslexic does not have a variable font, and its bold/italic glyphs are designed for readability rather than algorithmically derived.
