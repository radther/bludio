//! Theme system: color tokens, font configuration, text styles, and global state.
//!
//! Stores semantic design tokens and provides convenience accessors for
//! any UI component to reference theme values instead of hardcoded colors.
//!
//! ## Architecture
//!
//! ```text
//! Theme
//! ├── id: &'static str (stable identifier for registry lookups)
//! ├── appearance: Appearance (Light | Dark)
//! ├── font_family: SharedString ("Noto Sans")
//! ├── colors: ThemeColors (semantic color tokens)
//! └── text_styles: TextStyleSet (size + weight per role)
//! ```
//!
//! The active theme is stored in `GlobalSettings` (see `src/settings.rs`) and
//! accessed via `crate::settings::theme(cx)`. Runtime changes go through
//! `crate::settings::update_settings()`.

use gpui::{FontWeight, Hsla, SharedString};

// ── Appearance ─────────────────────────────────────────────────────────────

/// Whether the theme is light or dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Appearance {
    Light,
    Dark,
}

// ── ThemeColors ────────────────────────────────────────────────────────────

/// Semantic color tokens used by all UI components.
///
/// Each field represents a semantic role (e.g., "surface background", "accent
/// color", "danger text"). Components reference these by name, never by raw
/// HSLA values.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ThemeColors {
    // ── Background levels ──────────────────────────────────────────────
    /// Main app background (darkest).
    pub background: Hsla,
    /// Panel/header background.
    pub surface: Hsla,
    /// Sidebar/tab bar background.
    pub sidebar: Hsla,

    /// Button, input, element background.
    pub element_background: Hsla,
    /// Button/element hover state.
    pub element_hover: Hsla,

    /// Text input field background.
    pub input_background: Hsla,

    // ── Text colors ────────────────────────────────────────────────────
    /// Primary body text.
    pub text: Hsla,
    /// Secondary/subdued text (descriptions, labels).
    pub text_secondary: Hsla,
    /// Muted/placeholder text in inputs.
    pub text_placeholder: Hsla,
    /// Text color for colored buttons.
    pub text_colored_button: Hsla,

    // ── Borders ────────────────────────────────────────────────────────
    /// Primary border (panel separators, input borders).
    pub border: Hsla,
    /// Subtle border (item separators in lists).
    pub border_subtle: Hsla,

    // ── Accent / Interactive ───────────────────────────────────────────
    /// Bluetooth accent color.
    pub bluetooth_accent: Hsla,
    /// Audio accent color.
    pub audio_accent: Hsla,
    /// Developer accent color.
    pub dev_accent: Hsla,

    // ── Status colors ──────────────────────────────────────────────────
    /// Error/danger (destructive actions, error text).
    pub danger: Hsla,
    /// Warning (cautionary states).
    pub warning: Hsla,
    /// Success (connected, completed states).
    pub success: Hsla,

    // ── Misc ───────────────────────────────────────────────────────────
    /// Error banner / notification background.
    pub error_background: Hsla,
    /// Row/item hover overlay (typically a semi-transparent white/black).
    pub hover_overlay: Hsla,
    /// Selection highlight background (semi-transparent accent).
    pub selection_background: Hsla,
    /// Default icon fill color.
    pub icon: Hsla,
    /// Muted state text color (e.g., muted volume).
    pub muted: Hsla,

    // ── Menu ───────────────────────────────────────────────────────────
    /// Dropdown/context menu border.
    pub menu_border: Hsla,
}

// ── TextStyleSet ───────────────────────────────────────────────────────────

/// Maps each text role to a font size and weight.
///
/// Stored in [`Theme`] so different themes can define different
/// size scales and weight mappings. Call sites access fields directly:
///
/// ```ignore
/// let text_styles = &theme.text_styles;
/// div().styled(text_styles.heading)
/// ```
#[derive(Clone, Debug)]
pub(crate) struct TextStyleSet {
    pub body: (gpui::AbsoluteLength, FontWeight),
    pub body2: (gpui::AbsoluteLength, FontWeight),
    pub heading: (gpui::AbsoluteLength, FontWeight),
    pub caption: (gpui::AbsoluteLength, FontWeight),
}

impl Default for TextStyleSet {
    fn default() -> Self {
        Self {
            body: (gpui::rems(1.0).into(), FontWeight::MEDIUM),
            body2: (gpui::rems(0.875).into(), FontWeight::MEDIUM),
            heading: (gpui::rems(1.5).into(), FontWeight::EXTRA_BOLD),
            caption: (gpui::rems(0.75).into(), FontWeight::BOLD),
        }
    }
}

// ── Theme ──────────────────────────────────────────────────────────────────

/// The full theme definition: appearance, colors, font family, and text styles.
#[derive(Clone, Debug)]
pub(crate) struct Theme {
    /// Stable ID set at construction time. Backing store for the [`Theme::id()`]
    /// accessor; read via the method rather than directly.
    pub id: &'static str,
    pub appearance: Appearance,
    pub font_family: SharedString,
    pub colors: ThemeColors,
    pub text_styles: TextStyleSet,
}

impl Theme {
    /// Stable string identifier for this theme variant.
    ///
    /// This is the canonical way to read a theme's ID (the backing field is
    /// public only so constructors in sibling modules can set it).
    #[allow(dead_code)]
    pub(crate) fn id(&self) -> &'static str {
        self.id
    }
}

// ── Theme accessors ────────────────────────────────────────────────────────
//
// The active theme is stored in `GlobalSettings` (defined in `src/settings.rs`)
// and accessed via `crate::settings::theme(cx)`. The functions below are
// re-exported from `src/ui/theme/mod.rs` for backward compatibility with
// existing call sites.
//
// To change the theme at runtime, call `crate::settings::update_settings()`
// which updates the settings struct, recomputes the theme, persists, and
// triggers re-render.
