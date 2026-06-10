//! Theme registry: static map of theme IDs to theme constructors.
//!
//! Provides O(1) lookup by stable string ID and categorized lists
//! for light/dark theme dropdowns.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;

use super::types::Theme;
use super::{
    extreme_high_contrast, extreme_high_contrast_dark, high_contrast_dark, high_contrast_light,
    rose_pine, rose_pine_dawn, theme2, theme2_dark, theme3, theme3_dark, theme4, theme4_dark,
    theme5, theme5_dark,
};

// ── Type alias ─────────────────────────────────────────────────────────────

type ThemeCtor = fn() -> Arc<Theme>;

// ── Theme constructors (function pointers for the registry) ────────────────

fn make_rose_pine() -> Arc<Theme> {
    Arc::new(rose_pine())
}

fn make_rose_pine_dawn() -> Arc<Theme> {
    Arc::new(rose_pine_dawn())
}

fn make_high_contrast_light() -> Arc<Theme> {
    Arc::new(high_contrast_light())
}

fn make_high_contrast_dark() -> Arc<Theme> {
    Arc::new(high_contrast_dark())
}

fn make_extreme_high_contrast() -> Arc<Theme> {
    Arc::new(extreme_high_contrast())
}

fn make_extreme_high_contrast_dark() -> Arc<Theme> {
    Arc::new(extreme_high_contrast_dark())
}

fn make_theme2() -> Arc<Theme> {
    Arc::new(theme2())
}

fn make_theme2_dark() -> Arc<Theme> {
    Arc::new(theme2_dark())
}

fn make_theme3() -> Arc<Theme> {
    Arc::new(theme3())
}

fn make_theme3_dark() -> Arc<Theme> {
    Arc::new(theme3_dark())
}

fn make_theme4() -> Arc<Theme> {
    Arc::new(theme4())
}

fn make_theme4_dark() -> Arc<Theme> {
    Arc::new(theme4_dark())
}

fn make_theme5() -> Arc<Theme> {
    Arc::new(theme5())
}

fn make_theme5_dark() -> Arc<Theme> {
    Arc::new(theme5_dark())
}

// ── Registry ───────────────────────────────────────────────────────────────

/// Static map of theme ID to constructor function returning `Arc<Theme>`.
static REGISTRY: LazyLock<HashMap<&str, ThemeCtor>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("rose-pine", make_rose_pine as fn() -> Arc<Theme>);
    m.insert("rose-pine-dawn", make_rose_pine_dawn as fn() -> Arc<Theme>);
    m.insert(
        "high-contrast-light",
        make_high_contrast_light as fn() -> Arc<Theme>,
    );
    m.insert(
        "high-contrast-dark",
        make_high_contrast_dark as fn() -> Arc<Theme>,
    );
    m.insert(
        "extreme-high-contrast",
        make_extreme_high_contrast as fn() -> Arc<Theme>,
    );
    m.insert(
        "extreme-high-contrast-dark",
        make_extreme_high_contrast_dark as fn() -> Arc<Theme>,
    );
    m.insert("theme2", make_theme2 as fn() -> Arc<Theme>);
    m.insert("theme2dark", make_theme2_dark as fn() -> Arc<Theme>);
    m.insert("theme3", make_theme3 as fn() -> Arc<Theme>);
    m.insert("theme3dark", make_theme3_dark as fn() -> Arc<Theme>);
    m.insert("theme4", make_theme4 as fn() -> Arc<Theme>);
    m.insert("theme4dark", make_theme4_dark as fn() -> Arc<Theme>);
    m.insert("theme5", make_theme5 as fn() -> Arc<Theme>);
    m.insert("theme5dark", make_theme5_dark as fn() -> Arc<Theme>);
    m
});

/// IDs of all themes registered as light variants.
static LIGHT_THEME_IDS: &[&str] = &[
    "rose-pine-dawn",
    "high-contrast-light",
    "extreme-high-contrast",
    "theme2",
    "theme3",
    "theme4",
    "theme5",
];

/// IDs of all themes registered as dark variants.
static DARK_THEME_IDS: &[&str] = &[
    "rose-pine",
    "high-contrast-dark",
    "extreme-high-contrast-dark",
    "theme2dark",
    "theme3dark",
    "theme4dark",
    "theme5dark",
];

// ── Public helpers ─────────────────────────────────────────────────────────

/// Look up a theme constructor by its stable string ID.
/// Returns `None` if the ID is not known.
pub(crate) fn theme_for_id(id: &str) -> Option<Arc<Theme>> {
    REGISTRY.get(id).map(|ctor| ctor())
}

/// All registered light theme IDs.
pub(crate) fn light_theme_ids() -> &'static [&'static str] {
    LIGHT_THEME_IDS
}

/// All registered dark theme IDs.
pub(crate) fn dark_theme_ids() -> &'static [&'static str] {
    DARK_THEME_IDS
}
