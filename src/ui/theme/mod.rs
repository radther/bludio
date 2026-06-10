//! Theme system: color tokens, font configuration, text styles, and global state.
//!
//! Each theme variant lives in its own file (e.g., `rose_pine_dawn_theme.rs`)
//! with the source palette constants at the top and a `Theme` constructor below.
//!
//! Core types (`Theme`, `ThemeColors`, `TextStyleSet`, `Appearance`) are
//! defined in `types.rs`. The active theme is stored in `GlobalSettings`
//! (see `src/settings.rs`) and accessed via the re-exported `theme(cx)`.

mod extreme_high_contrast_dark_theme;
mod extreme_high_contrast_theme;
mod high_contrast_dark_theme;
mod high_contrast_light_theme;
mod registry;
mod rose_pine_dawn_theme;
mod rose_pine_theme;
mod theme2_theme;
mod theme2dark_theme;
mod theme3_theme;
mod theme3dark_theme;
mod theme4_theme;
mod theme4dark_theme;
mod theme5_theme;
mod theme5dark_theme;
mod types;

pub(crate) use crate::backend::settings::{ThemeMode, theme};
pub(crate) use extreme_high_contrast_dark_theme::extreme_high_contrast_dark;
pub(crate) use extreme_high_contrast_theme::extreme_high_contrast;
pub(crate) use high_contrast_dark_theme::high_contrast_dark;
pub(crate) use high_contrast_light_theme::high_contrast_light;
pub(crate) use registry::{dark_theme_ids, light_theme_ids, theme_for_id};
pub(crate) use rose_pine_dawn_theme::rose_pine_dawn;
pub(crate) use rose_pine_theme::rose_pine;
pub(crate) use theme2_theme::theme2;
pub(crate) use theme2dark_theme::theme2_dark;
pub(crate) use theme3_theme::theme3;
pub(crate) use theme3dark_theme::theme3_dark;
pub(crate) use theme4_theme::theme4;
pub(crate) use theme4dark_theme::theme4_dark;
pub(crate) use theme5_theme::theme5;
pub(crate) use theme5dark_theme::theme5_dark;
pub(crate) use types::*;
