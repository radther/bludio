//! Theme 1 — black/white light template theme.
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
const ALMOST_WHITE: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.9,
    a: 1.0,
};
const WHITE: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 1.0,
    a: 1.0,
};

// ── Constructor ────────────────────────────────────────────────────────────

/// Build the full Theme 1 [`Theme`] from the black/white template palette.
pub(crate) fn extreme_high_contrast() -> Theme {
    Theme {
        id: "extreme-high-contrast",
        appearance: Appearance::Light,
        font_family: "Noto Sans".into(),
        text_styles: Default::default(),
        colors: ThemeColors {
            // ── Background levels ──────────────────────────────────────────
            background: WHITE,
            surface: ALMOST_WHITE,
            background_hover: ALMOST_WHITE,
            element_background: ALMOST_WHITE,
            element_hover: WHITE,
            input_background: ALMOST_WHITE,

            // ── Text colors ────────────────────────────────────────────────
            text: BLACK,
            text_secondary: BLACK,
            text_muted: BLACK,

            text_colored_button: WHITE,

            // ── Borders ────────────────────────────────────────────────────
            border: BLACK,

            // ── Accent / Interactive ───────────────────────────────────────
            bluetooth_accent: BLACK,
            audio_accent: BLACK,
            dev_accent: BLACK,

            // ── Status colors ──────────────────────────────────────────────
            danger: BLACK,
            warning: BLACK,
            success: BLACK,
        },
    }
}
