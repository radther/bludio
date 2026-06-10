//! Theme 1 Dark — black/white dark template theme.
//!
//! Template palette for manual customization. Only pure black and pure white
//! are used so this file is a minimal starting point.

#![allow(dead_code)]

use gpui::Hsla;

use super::types::{Appearance, Theme, ThemeColors};

// ── Template palette ──────────────────────────────────────────────────────

const BLACK: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.0,
    a: 1.0,
};
const ALMOST_BLACK: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.1,
    a: 1.0,
};
const WHITE: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 1.0,
    a: 1.0,
};

// ── Constructor ────────────────────────────────────────────────────────────

/// Build the full Theme 1 Dark [`Theme`] from the black/white template palette.
pub(crate) fn extreme_high_contrast_dark() -> Theme {
    Theme {
        id: "extreme-high-contrast-dark",
        appearance: Appearance::Dark,
        font_family: "Noto Sans".into(),
        text_styles: Default::default(),
        colors: ThemeColors {
            // ── Background levels ──────────────────────────────────────────
            background: BLACK,
            surface: ALMOST_BLACK,
            background_hover: ALMOST_BLACK,
            element_background: ALMOST_BLACK,
            element_hover: BLACK,
            input_background: ALMOST_BLACK,

            // ── Text colors ────────────────────────────────────────────────
            text: WHITE,
            text_secondary: WHITE,
            text_muted: WHITE,

            text_colored_button: BLACK,

            // ── Borders ────────────────────────────────────────────────────
            border: WHITE,

            // ── Accent / Interactive ───────────────────────────────────────
            bluetooth_accent: WHITE,
            audio_accent: WHITE,
            dev_accent: WHITE,

            // ── Status colors ──────────────────────────────────────────────
            danger: WHITE,
            warning: WHITE,
            success: WHITE,
        },
    }
}
