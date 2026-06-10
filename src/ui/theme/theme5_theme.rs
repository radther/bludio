//! Theme 5 — black/white light template theme.
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

/// Build the full Theme 5 [`Theme`] from the black/white template palette.
pub(crate) fn theme5() -> Theme {
    Theme {
        id: "theme5",
        appearance: Appearance::Light,
        font_family: "Noto Sans".into(),
        text_styles: Default::default(),
        colors: ThemeColors {
            // ── Background levels ──────────────────────────────────────────
            background: WHITE,
            surface: WHITE,
            background_hover: WHITE,
            element_background: WHITE,
            element_hover: WHITE,
            input_background: WHITE,

            // ── Text colors ────────────────────────────────────────────────
            text: BLACK,
            text_secondary: BLACK,
            text_muted: BLACK,

            text_colored_button: BLACK,

            // ── Borders ────────────────────────────────────────────────────
            border: BLACK,

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
