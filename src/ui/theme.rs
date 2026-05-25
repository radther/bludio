//! Theme system: color tokens, font configuration, text styles, and global state.
//!
//! Stores semantic design tokens and provides convenience accessors for
//! any UI component to reference theme values instead of hardcoded colors.
//!
//! ## Architecture
//!
//! ```text
//! Theme
//! ├── appearance: Appearance (Light | Dark)
//! ├── font_family: SharedString ("Noto Sans")
//! ├── colors: ThemeColors (semantic color tokens)
//! └── text_styles: TextStyleSet (size + weight per role)
//! ```
//!
//! The active theme is stored as a GPUI global (`GlobalTheme`) and accessed
//! via the free functions `theme(cx)` and `set_theme(theme, cx)`.

use gpui::{App, BorrowAppContext, FontWeight, Global, Hsla, SharedString, hsla};
use std::sync::Arc;

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
    /// Dropdown/context menu background.
    pub menu_background: Hsla,

    // ── Text colors ────────────────────────────────────────────────────
    /// Primary body text.
    pub text: Hsla,
    /// Secondary/subdued text (descriptions, labels).
    pub text_secondary: Hsla,
    /// Muted/placeholder text in inputs.
    pub text_placeholder: Hsla,

    // ── Borders ────────────────────────────────────────────────────────
    /// Primary border (panel separators, input borders).
    pub border: Hsla,
    /// Subtle border (item separators in lists).
    pub border_subtle: Hsla,

    // ── Accent / Interactive ───────────────────────────────────────────
    /// Primary accent (buttons, highlights, active selections).
    pub accent: Hsla,
    /// Accent hover state.
    pub accent_hover: Hsla,

    // ── Status colors ──────────────────────────────────────────────────
    /// Error/danger (destructive actions, error text).
    pub danger: Hsla,
    /// Danger hover state.
    pub danger_hover: Hsla,
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
    pub heading: (gpui::AbsoluteLength, FontWeight),
    pub body_small: (gpui::AbsoluteLength, FontWeight),
    pub caption: (gpui::AbsoluteLength, FontWeight),
}

impl TextStyleSet {
    fn default() -> Self {
        Self {
            body: (gpui::rems(1.0).into(), FontWeight::NORMAL),
            heading: (gpui::rems(1.0).into(), FontWeight::BOLD),
            body_small: (gpui::rems(0.875).into(), FontWeight::NORMAL),
            caption: (gpui::rems(0.75).into(), FontWeight::MEDIUM),
        }
    }
}

// ── Theme ──────────────────────────────────────────────────────────────────

/// The full theme definition: appearance, colors, font family, and text styles.
#[derive(Clone, Debug)]
pub(crate) struct Theme {
    pub appearance: Appearance,
    pub font_family: SharedString,
    pub colors: ThemeColors,
    pub text_styles: TextStyleSet,
}

impl Theme {
    /// The dark theme (default, matches the original hardcoded look).
    pub fn dark() -> Self {
        Self {
            appearance: Appearance::Dark,
            font_family: "Noto Sans".into(),
            text_styles: TextStyleSet::default(),
            colors: ThemeColors {
                background: hsla(0.0, 0.0, 0.08, 1.0),
                surface: hsla(0.0, 0.0, 0.14, 1.0),
                sidebar: hsla(0.0, 0.0, 0.12, 1.0),
                element_background: hsla(0.0, 0.0, 0.22, 1.0),
                element_hover: hsla(0.0, 0.0, 0.30, 1.0),
                input_background: hsla(0.0, 0.0, 0.18, 1.0),
                menu_background: hsla(0.0, 0.0, 0.16, 1.0),
                text: hsla(0.0, 0.0, 0.95, 1.0),
                text_secondary: hsla(0.0, 0.0, 0.60, 1.0),
                text_placeholder: hsla(0.0, 0.0, 0.45, 1.0),
                border: hsla(0.0, 0.0, 0.25, 1.0),
                border_subtle: hsla(0.0, 0.0, 0.20, 1.0),
                accent: hsla(210.0 / 360.0, 0.7, 0.55, 1.0),
                accent_hover: hsla(210.0 / 360.0, 0.7, 0.45, 1.0),
                danger: hsla(0.0, 0.7, 0.55, 1.0),
                danger_hover: hsla(0.0, 0.7, 0.45, 1.0),
                warning: hsla(45.0 / 360.0, 0.8, 0.55, 1.0),
                success: hsla(140.0 / 360.0, 0.6, 0.50, 1.0),
                error_background: Hsla::from(gpui::rgba(0xff3c_3c33)),
                hover_overlay: hsla(0.0, 0.0, 1.0, 0.04),
                selection_background: hsla(210.0 / 360.0, 0.6, 0.5, 0.3),
                icon: hsla(0.0, 0.0, 0.90, 1.0),
                muted: hsla(0.0, 0.0, 0.40, 1.0),
                menu_border: hsla(0.0, 0.0, 0.30, 1.0),
            },
        }
    }

    /// The light theme (warm off-white background, dark text).
    pub fn light() -> Self {
        Self {
            appearance: Appearance::Light,
            font_family: "Noto Sans".into(),
            text_styles: TextStyleSet::default(),
            colors: ThemeColors {
                background: hsla(36.0 / 360.0, 0.16, 0.94, 1.0),
                surface: hsla(40.0 / 360.0, 0.08, 0.88, 1.0),
                sidebar: hsla(40.0 / 360.0, 0.08, 0.82, 1.0),
                element_background: hsla(36.0 / 360.0, 0.06, 0.72, 1.0),
                element_hover: hsla(36.0 / 360.0, 0.08, 0.64, 1.0),
                input_background: hsla(36.0 / 360.0, 0.06, 0.78, 1.0),
                menu_background: hsla(40.0 / 360.0, 0.08, 0.90, 1.0),
                text: hsla(15.0 / 360.0, 0.03, 0.31, 1.0),
                text_secondary: hsla(36.0 / 360.0, 0.02, 0.43, 1.0),
                text_placeholder: hsla(34.0 / 360.0, 0.03, 0.55, 1.0),
                border: hsla(50.0 / 360.0, 0.05, 0.76, 1.0),
                border_subtle: hsla(50.0 / 360.0, 0.03, 0.82, 1.0),
                accent: hsla(210.0 / 360.0, 0.7, 0.45, 1.0),
                accent_hover: hsla(210.0 / 360.0, 0.7, 0.38, 1.0),
                danger: hsla(0.0, 0.7, 0.48, 1.0),
                danger_hover: hsla(0.0, 0.7, 0.40, 1.0),
                warning: hsla(45.0 / 360.0, 0.8, 0.45, 1.0),
                success: hsla(140.0 / 360.0, 0.5, 0.40, 1.0),
                error_background: Hsla::from(gpui::rgba(0xffe0_e0e0)),
                hover_overlay: hsla(0.0, 0.0, 0.0, 0.04),
                selection_background: hsla(210.0 / 360.0, 0.6, 0.5, 0.15),
                icon: hsla(0.0, 0.0, 0.30, 1.0),
                muted: hsla(0.0, 0.0, 0.55, 1.0),
                menu_border: hsla(50.0 / 360.0, 0.05, 0.76, 1.0),
            },
        }
    }
}

// ── Global theme ───────────────────────────────────────────────────────────

/// Wrapper that stores the active theme as a GPUI global.
///
/// Stored as `Arc<Theme>` so async tasks can cheaply clone and hold a
/// reference without lifetime issues.
pub(crate) struct GlobalTheme {
    theme: Arc<Theme>,
}

impl Global for GlobalTheme {}

impl GlobalTheme {
    /// Create a new global theme wrapper.
    pub fn new(theme: Theme) -> Self {
        Self {
            theme: Arc::new(theme),
        }
    }
}

// ── Public accessors ──────────────────────────────────────────────────────

/// Retrieve the current active theme.
///
/// Panics if `GlobalTheme` has not been initialized (set via `set_theme` or
/// via `cx.set_global(GlobalTheme::new(...))`).
pub(crate) fn theme(cx: &App) -> &Arc<Theme> {
    &cx.global::<GlobalTheme>().theme
}

/// Replace the active theme.
///
/// The UI will re-render automatically via GPUI's global observation.
pub(crate) fn set_theme(new_theme: Theme, cx: &mut App) {
    cx.update_global::<GlobalTheme, _>(|global, _cx| {
        global.theme = Arc::new(new_theme);
    });
}
