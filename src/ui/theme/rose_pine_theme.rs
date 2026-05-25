//! Rose Pine — the original dark theme.
//!
//! The private constants below are the full Rose Pine palette, kept for
//! reference when selecting which colors to map to semantic tokens. Not every
//! color is used; the module-level `#![allow(dead_code)]` suppresses warnings.

#![allow(dead_code)]

use gpui::Hsla;

use super::theme::{Appearance, Theme, ThemeColors};

// ── Convenience macro ──────────────────────────────────────────────────────

/// Construct an [`Hsla`] from degree-based hue, saturation, and lightness.
///
/// All values are in the range 0.0–1.0 except `h` which is in degrees
/// (0–360). Alpha is always 1.0 (fully opaque).
macro_rules! hsl {
    ($h:expr, $s:expr, $l:expr) => {
        Hsla {
            h: ($h) / 360.0,
            s: ($s),
            l: ($l),
            a: 1.0,
        }
    };
}

// ── Rose Pine palette ──────────────────────────────────────────────────────

const BASE: Hsla = hsl!(249.0, 0.22, 0.12);
const SURFACE: Hsla = hsl!(247.0, 0.23, 0.15);
const OVERLAY: Hsla = hsl!(248.0, 0.25, 0.18);
const MUTED: Hsla = hsl!(249.0, 0.12, 0.47);
const SUBTLE: Hsla = hsl!(248.0, 0.15, 0.61);
const TEXT: Hsla = hsl!(245.0, 0.50, 0.91);
const LOVE: Hsla = hsl!(343.0, 0.76, 0.68);
const GOLD: Hsla = hsl!(35.0, 0.88, 0.72);
const ROSE: Hsla = hsl!(2.0, 0.55, 0.83);
const PINE: Hsla = hsl!(197.0, 0.49, 0.38);
const FOAM: Hsla = hsl!(189.0, 0.43, 0.73);
const IRIS: Hsla = hsl!(267.0, 0.57, 0.78);
const HIGHLIGHT_LOW: Hsla = hsl!(244.0, 0.18, 0.15);
const HIGHLIGHT_MED: Hsla = hsl!(249.0, 0.15, 0.28);
const HIGHLIGHT_HIGH: Hsla = hsl!(248.0, 0.13, 0.36);

// ── Constructor ────────────────────────────────────────────────────────────

/// Build the full Rose Pine [`Theme`] by mapping palette colors to semantic
/// color tokens.
pub(crate) fn rose_pine() -> Theme {
    Theme {
        appearance: Appearance::Dark,
        font_family: "Noto Sans".into(),
        text_styles: Default::default(),
        colors: ThemeColors {
            // ── Background levels ──────────────────────────────────────────
            background: BASE,
            surface: SURFACE,
            sidebar: OVERLAY,
            element_background: HIGHLIGHT_MED,
            element_hover: HIGHLIGHT_HIGH,
            element_selected: HIGHLIGHT_MED,
            input_background: HIGHLIGHT_LOW,
            menu_background: OVERLAY,

            // ── Text colors ────────────────────────────────────────────────
            text: TEXT,
            text_secondary: SUBTLE,
            text_placeholder: MUTED,

            // ── Borders ────────────────────────────────────────────────────
            border: HIGHLIGHT_HIGH,
            border_subtle: HIGHLIGHT_MED,

            // ── Accent / Interactive ───────────────────────────────────────
            accent: IRIS,
            accent_hover: hsl!(267.0, 0.57, 0.70),
            // Lighter iris for hover.

            // ── Status colors ──────────────────────────────────────────────
            danger: LOVE,
            danger_hover: hsl!(343.0, 0.76, 0.60),
            // Darker love for hover.
            warning: GOLD,
            success: PINE,

            // ── Misc ───────────────────────────────────────────────────────
            error_background: Hsla {
                h: 343.0 / 360.0,
                s: 0.76,
                l: 0.68,
                a: 0.15,
            },
            // Love at 15% opacity — dark enough for a dark bg.
            hover_overlay: Hsla {
                h: 244.0 / 360.0,
                s: 0.18,
                l: 0.15,
                a: 0.50,
            },
            // Highlight Low at 50%.
            selection_background: Hsla {
                h: 267.0 / 360.0,
                s: 0.57,
                l: 0.78,
                a: 0.15,
            },
            // Iris at 15%.
            icon: SUBTLE,
            muted: MUTED,

            // ── Menu ───────────────────────────────────────────────────────
            menu_border: HIGHLIGHT_HIGH,
        },
    }
}
