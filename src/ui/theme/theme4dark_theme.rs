//! Theme 4 Dark — black/white dark template theme.
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
const WHITE: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 1.0,
    a: 1.0,
};

// ── Constructor ────────────────────────────────────────────────────────────

/// Build the full Theme 4 Dark [`Theme`] from the black/white template palette.
pub(crate) fn theme4_dark() -> Theme {
    Theme {
        id: "theme4dark",
        appearance: Appearance::Dark,
        font_family: "Noto Sans".into(),
        text_styles: Default::default(),
        colors: ThemeColors {
            // ── Background levels ──────────────────────────────────────────
            background: BLACK,
            surface: BLACK,
            background_hover: BLACK,
            element_background: BLACK,
            element_hover: BLACK,
            input_background: BLACK,

            // ── Text colors ────────────────────────────────────────────────
            text: WHITE,
            text_secondary: WHITE,
            text_muted: WHITE,

            text_colored_button: WHITE,

            // ── Borders ────────────────────────────────────────────────────
            border: WHITE,

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
