## 1. Project Setup

- [x] 1.1 Create `rust-toolchain.toml` pinning Rust 1.95.0 with `rustfmt` and `clippy` components
- [x] 1.2 Create `rustfmt.toml` with `edition = "2024"` and `style_edition = "2024"`
- [x] 1.3 Create `.gitignore` covering `/target`, `.DS_Store`, and IDE files
- [x] 1.4 Create `Cargo.toml` with package name `bludio`, edition 2024, dependencies on `gpui-unofficial` v1.2.7 and `gpui-platform-gpui-unofficial` v1.2.7 (features: `font-kit`, `x11`, `wayland`)

## 2. Application Shell

- [x] 2.1 Create `src/main.rs` with a `BludioApp` struct implementing `gpui::Render`
- [x] 2.2 Implement `Render` trait to produce a full-size black-background `div` with centered white "bludio" text
- [x] 2.3 Implement `main()` using `gpui_platform_gpui_unofficial::application().run()` to open a 1100×700 window centered on the display, holding a `BludioApp` instance

## 3. Verification

- [x] 3.1 Run `cargo build` and confirm zero errors
- [x] 3.2 Run `cargo fmt --check` and confirm formatting compliance
- [x] 3.3 Run `cargo clippy -- -D warnings` and confirm zero warnings
