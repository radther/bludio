//! High Contrast Light — accessibility-optimized light theme.
//!
//! Maximizes luminance contrast: near-white backgrounds, near-black text,
//! and saturated accent colors for clear distinction.

use gpui::Hsla;

use super::types::{Appearance, Theme, ThemeColors};

// ── High Contrast Light palette ───────────────────────────────────────────

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
const DARK_GRAY: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.15,
    a: 1.0,
};
const MED_GRAY: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.40,
    a: 1.0,
};
const LIGHT_GRAY: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.85,
    a: 1.0,
};
const BLUE: Hsla = Hsla {
    h: 240.0 / 360.0,
    s: 1.0,
    l: 0.5,
    a: 1.0,
};
const GREEN: Hsla = Hsla {
    h: 120.0 / 360.0,
    s: 1.0,
    l: 0.35,
    a: 1.0,
};
const RED: Hsla = Hsla {
    h: 0.0 / 360.0,
    s: 1.0,
    l: 0.4,
    a: 1.0,
};
const YELLOW: Hsla = Hsla {
    h: 60.0 / 360.0,
    s: 1.0,
    l: 0.5,
    a: 1.0,
};

// ── Constructor ────────────────────────────────────────────────────────────

/// Build the full High Contrast Light [`Theme`].
pub(crate) fn high_contrast_light() -> Theme {
    Theme {
        id: "high-contrast-light",
        appearance: Appearance::Light,
        font_family: "Noto Sans".into(),
        text_styles: Default::default(),
        colors: ThemeColors {
            // ── Background levels ──────────────────────────────────────────
            background: WHITE,
            surface: WHITE,
            sidebar: LIGHT_GRAY,

            element_background: LIGHT_GRAY,
            element_hover: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.75,
                a: 1.0,
            },
            input_background: WHITE,

            // ── Text colors ────────────────────────────────────────────────
            text: BLACK,
            text_secondary: DARK_GRAY,
            text_placeholder: MED_GRAY,
            text_colored_button: WHITE,

            // ── Borders ────────────────────────────────────────────────────
            border: BLACK,
            border_subtle: DARK_GRAY,

            // ── Accent / Interactive ───────────────────────────────────────
            bluetooth_accent: BLUE,
            audio_accent: Hsla {
                h: 270.0 / 360.0,
                s: 1.0,
                l: 0.4,
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
                l: 0.4,
                a: 0.12,
            },
            hover_overlay: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.10,
            },
            selection_background: Hsla {
                h: 240.0 / 360.0,
                s: 1.0,
                l: 0.5,
                a: 0.20,
            },
            icon: DARK_GRAY,
            muted: MED_GRAY,

            // ── Menu ───────────────────────────────────────────────────────
            menu_border: BLACK,
        },
    }
}
