//! High Contrast Dark — accessibility-optimized dark theme.
//!
//! Maximizes luminance contrast: near-black backgrounds, near-white text,
//! and saturated accent colors for clear distinction.

use gpui::Hsla;

use super::types::{Appearance, Theme, ThemeColors};

// ── High Contrast Dark palette ────────────────────────────────────────────

const WHITE: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 1.0,
    a: 1.0,
};
const BLACK: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.0,
    a: 1.0,
};
const NEAR_BLACK: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.08,
    a: 1.0,
};
const DARK_GRAY: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.20,
    a: 1.0,
};
const LIGHT_GRAY: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.70,
    a: 1.0,
};
const CYAN: Hsla = Hsla {
    h: 180.0 / 360.0,
    s: 1.0,
    l: 0.55,
    a: 1.0,
};
const GREEN: Hsla = Hsla {
    h: 120.0 / 360.0,
    s: 1.0,
    l: 0.55,
    a: 1.0,
};
const RED: Hsla = Hsla {
    h: 0.0 / 360.0,
    s: 1.0,
    l: 0.55,
    a: 1.0,
};
const YELLOW: Hsla = Hsla {
    h: 60.0 / 360.0,
    s: 1.0,
    l: 0.55,
    a: 1.0,
};

// ── Constructor ────────────────────────────────────────────────────────────

/// Build the full High Contrast Dark [`Theme`].
pub(crate) fn high_contrast_dark() -> Theme {
    Theme {
        id: "high-contrast-dark",
        appearance: Appearance::Dark,
        font_family: "Noto Sans".into(),
        text_styles: Default::default(),
        colors: ThemeColors {
            // ── Background levels ──────────────────────────────────────────
            background: BLACK,
            surface: NEAR_BLACK,
            sidebar: DARK_GRAY,

            element_background: DARK_GRAY,
            element_hover: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.30,
                a: 1.0,
            },
            input_background: NEAR_BLACK,

            // ── Text colors ────────────────────────────────────────────────
            text: WHITE,
            text_secondary: LIGHT_GRAY,
            text_placeholder: LIGHT_GRAY,
            text_colored_button: BLACK,

            // ── Borders ────────────────────────────────────────────────────
            border: WHITE,
            border_subtle: LIGHT_GRAY,

            // ── Accent / Interactive ───────────────────────────────────────
            bluetooth_accent: CYAN,
            audio_accent: Hsla {
                h: 270.0 / 360.0,
                s: 1.0,
                l: 0.65,
                a: 1.0,
            },
            dev_accent: GREEN,

            // ── Status colors ──────────────────────────────────────────────
            danger: RED,
            warning: YELLOW,
            success: GREEN,

            // ── Misc ───────────────────────────────────────────────────────
            error_background: Hsla {
                h: 0.0,
                s: 1.0,
                l: 0.55,
                a: 0.15,
            },
            hover_overlay: Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.15,
            },
            selection_background: Hsla {
                h: 180.0 / 360.0,
                s: 1.0,
                l: 0.55,
                a: 0.25,
            },
            icon: LIGHT_GRAY,
            muted: LIGHT_GRAY,

            // ── Menu ───────────────────────────────────────────────────────
            menu_border: WHITE,
        },
    }
}
