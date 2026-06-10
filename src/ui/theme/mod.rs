//! Theme system: color tokens, font configuration, text styles, and global state.
//!
//! Themes are loaded from JSON files at startup: built-in themes are embedded
//! via `include_str!`, and user themes are scanned from
//! `~/.config/bludio/themes/`.
//!
//! Core types (`Theme`, `ThemeColors`, `TextStyleSet`, `Appearance`) are
//! defined in `types.rs`. The active theme is stored in `GlobalSettings`
//! (see `src/settings.rs`) and accessed via the re-exported `theme(cx)`.

mod loader;
mod registry;
mod types;

pub(crate) use crate::backend::settings::{ThemeMode, theme};
pub(crate) use registry::{dark_theme_ids, light_theme_ids, theme_display_name, theme_for_id};
pub(crate) use types::*;
