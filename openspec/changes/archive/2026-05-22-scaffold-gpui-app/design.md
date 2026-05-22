## Context

This is a greenfield GPUI application. The rss-reader reference (`../gpui_test/crates/rss-reader/`) demonstrates the correct `gpui-unofficial` / `gpui-platform-gpui-unofficial` crate ecosystem at v1.2.7. The zed repository (`../zed/`) establishes Rust best practices for project structure (workspace layout, toolchain pinning, edition 2024, rustfmt config). The scaffold must be minimal — a single binary crate that opens a window and renders centered text.

## Goals / Non-Goals

**Goals:**
- Produce a compilable, runnable Rust binary that opens a GPUI window
- Display "bludio" as white text centered on a black background
- Use exact same `gpui-unofficial` / `gpui-platform-gpui-unofficial` v1.2.7 dependencies as the rss-reader reference
- Follow zed's toolchain and formatting conventions (Rust 1.95.0, edition 2024, style_edition 2024)
- Serve as the foundation for all future feature additions

**Non-Goals:**
- No workspace monorepo (single binary crate is fine for now; add a workspace later if multiple crates are needed)
- No theming system, no user input, no settings
- No CLI argument parsing beyond what GPUI provides
- No CI/CD, packaging, or distribution configuration

## Decisions

1. **Single binary crate at repo root (not a workspace)**
   - *Rationale*: We only have one crate. Zed uses a workspace because it has 200+ crates. A workspace can be introduced later when needed. The rss-reader is also a standalone crate (no workspace).
   - *Alternative*: A workspace Cargo.toml at root with `crates/bludio/` — over-engineered for a single binary.

2. **`gpui-unofficial` v1.2.7 and `gpui-platform-gpui-unofficial` v1.2.7**
   - *Rationale*: These are the exact versions used in the rss-reader reference. Pin to minor version to avoid breaking changes.
   - *Platform features*: `font-kit`, `x11`, `wayland` — same as rss-reader, covers Linux rendering backends plus font loading.

3. **`application().run()` pattern for app entry**
   - *Rationale*: This is the standard GPUI bootstrap used by both the rss-reader reference and gpui examples. The `gpui_platform_gpui_unofficial::application()` function initializes the platform layer, then `run()` starts the event loop with a callback receiving `&mut App`.

4. **Minimal centered-text render via `div()` + `flex()`**
   - *Rationale*: GPUI's declarative `Render` trait approach uses `div().flex().items_center().justify_center().size_full().bg(...)`. Simple and idiomatic. No need for custom rendering or text measurement for a static label.

5. **`rust-toolchain.toml` pinning Rust 1.95.0**
   - *Rationale*: Zed pins 1.95.0. Pinning prevents CI/contributor drift. We match zed exactly so we can lift patterns (like edition 2024 features) without compatibility issues.

6. **Edition 2024 and style_edition 2024 in rustfmt.toml**
   - *Rationale*: Zed uses these. Edition 2024 is required by `gpui-unofficial` v1.2.7 (the rss-reader Cargo.toml specifies `edition = "2024"`).

7. **No `build.rs` for now**
   - *Rationale*: The rss-reader doesn't use one. Zed's build.rs is for macOS/Linux/Windows platform-specific concerns (code signing, icon embedding, conpty download). None apply to a simple scaffold.

## Risks / Trade-offs

- **[Risk] `gpui-unofficial` is a third-party crate with no stability guarantees** → The pinned version v1.2.7 has been verified working via the rss-reader. We lock to a minor version in Cargo.toml.
- **[Risk] The scaffold relies on system GPU drivers (Vulkan/Metal)** → GPUI warns on software rendering. Acceptable for development; CI and packaging concerns are non-goals.

## Open Questions

- What window size should the scaffold use? (Defer: use the rss-reader's 1100×700 as a sensible default that looks intentional.)
