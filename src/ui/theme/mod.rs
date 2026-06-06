//! Theme system: color tokens, font configuration, text styles, and global state.
//!
//! Each theme variant lives in its own file (e.g., `rose_pine_dawn_theme.rs`)
//! with the source palette constants at the top and a `Theme` constructor below.
//!
//! Core types (`Theme`, `ThemeColors`, `TextStyleSet`, `Appearance`) are
//! defined in `types.rs`. The active theme is stored in `GlobalSettings`
//! (see `src/settings.rs`) and accessed via the re-exported `theme(cx)`.

mod high_contrast_dark_theme;
mod high_contrast_light_theme;
mod registry;
mod rose_pine_dawn_theme;
mod rose_pine_theme;
mod types;

pub(crate) use crate::settings::{ThemeMode, theme, update_settings};
pub(crate) use high_contrast_dark_theme::high_contrast_dark;
pub(crate) use high_contrast_light_theme::high_contrast_light;
pub(crate) use registry::{dark_theme_ids, light_theme_ids, theme_for_id};
pub(crate) use rose_pine_dawn_theme::rose_pine_dawn;
pub(crate) use rose_pine_theme::rose_pine;
pub(crate) use types::*;
