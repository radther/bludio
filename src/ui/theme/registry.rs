//! Theme registry: static map of theme IDs to theme constructors.
//!
//! Provides O(1) lookup by stable string ID and categorized lists
//! for light/dark theme dropdowns.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;

use super::types::Theme;
use super::{high_contrast_dark, high_contrast_light, rose_pine, rose_pine_dawn};

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
    m
});

/// IDs of all themes registered as light variants.
static LIGHT_THEME_IDS: &[&str] = &["rose-pine-dawn", "high-contrast-light"];

/// IDs of all themes registered as dark variants.
static DARK_THEME_IDS: &[&str] = &["rose-pine", "high-contrast-dark"];

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
