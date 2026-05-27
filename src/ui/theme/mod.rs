//! Theme system: color tokens, font configuration, text styles, and global state.
//!
//! Each theme variant lives in its own file (e.g., `rose_pine_dawn_theme.rs`)
//! with the source palette constants at the top and a `Theme` constructor below.
//!
//! Core types (`Theme`, `ThemeColors`, `TextStyleSet`, `Appearance`,
//! `GlobalTheme`) and accessors (`theme()`, `set_theme()`) are defined in
//! `types.rs`.

mod rose_pine_dawn_theme;
mod rose_pine_theme;
mod types;

#[allow(unused_imports)]
pub(crate) use rose_pine_dawn_theme::rose_pine_dawn;
#[allow(unused_imports)]
pub(crate) use rose_pine_theme::rose_pine;
pub(crate) use types::*;
