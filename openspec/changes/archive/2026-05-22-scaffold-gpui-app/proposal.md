## Why

The project is currently empty. We need a minimal GPUI application scaffold as the foundation for iterative feature development. This scaffold establishes a working window with the app name rendered on screen, proving the rendering pipeline, crate dependency chain, and build system are correctly wired end-to-end before we add any real functionality.

## What Changes

- Create a Rust binary crate `bludio` with `Cargo.toml` depending on `gpui-unofficial` v1.2.7 and `gpui-platform-gpui-unofficial` v1.2.7 (same versions as the rss-reader reference)
- Add a `main.rs` entry point that opens a GPUI window displaying the centered text "bludio" in white on a black background
- Apply `rust-toolchain.toml` (Rust 1.95.0, edition 2024) and `rustfmt.toml` (edition 2024, style_edition 2024) matching zed best practices
- Add a `.gitignore` covering Rust build artifacts

## Capabilities

### New Capabilities

- `app-shell`: A minimal GPUI application that opens a window and renders centered white text ("bludio") on a black background using the gpui-unofficial crate ecosystem. This forms the visual shell that all future features will compose into.

### Modified Capabilities

<!-- No existing capabilities to modify — greenfield project. -->

## Impact

- New crate `bludio` at workspace root
- Dependencies: `gpui-unofficial` v1.2.7, `gpui-platform-gpui-unofficial` v1.2.7 (with `font-kit`, `x11`, `wayland` features)
- Build toolchain: Rust 1.95.0, edition 2024
- No existing code affected (greenfield)
