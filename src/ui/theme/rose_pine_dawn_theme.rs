//! Rose Pine Dawn — a light, warm theme.
//!
//! The private constants below are the full Rose Pine Dawn palette, kept for
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

// ── Rose Pine Dawn palette ─────────────────────────────────────────────────

const BASE: Hsla = hsl!(32.0, 0.57, 0.95);
const SURFACE: Hsla = hsl!(35.0, 1.00, 0.98);
const OVERLAY: Hsla = hsl!(33.0, 0.43, 0.91);
const MUTED: Hsla = hsl!(257.0, 0.09, 0.61);
const SUBTLE: Hsla = hsl!(248.0, 0.12, 0.52);
const TEXT: Hsla = hsl!(248.0, 0.19, 0.40);
const LOVE: Hsla = hsl!(343.0, 0.35, 0.55);
const GOLD: Hsla = hsl!(35.0, 0.81, 0.56);
const ROSE: Hsla = hsl!(3.0, 0.53, 0.67);
const PINE: Hsla = hsl!(197.0, 0.53, 0.34);
const FOAM: Hsla = hsl!(189.0, 0.30, 0.48);
const IRIS: Hsla = hsl!(268.0, 0.21, 0.57);
const HIGHLIGHT_LOW: Hsla = hsl!(25.0, 0.35, 0.93);
const HIGHLIGHT_MED: Hsla = hsl!(10.0, 0.09, 0.86);
const HIGHLIGHT_HIGH: Hsla = hsl!(315.0, 0.04, 0.80);

// ── Constructor ────────────────────────────────────────────────────────────

/// Build the full Rose Pine Dawn [`Theme`] by mapping palette colors to
/// semantic color tokens.
pub(crate) fn rose_pine_dawn() -> Theme {
    Theme {
        appearance: Appearance::Light,
        font_family: "Noto Sans".into(),
        text_styles: Default::default(),
        colors: ThemeColors {
            // ── Background levels ──────────────────────────────────────────
            background: BASE,
            surface: SURFACE,
            sidebar: OVERLAY,

            element_background: HIGHLIGHT_MED,
            element_hover: HIGHLIGHT_HIGH,
            input_background: HIGHLIGHT_LOW,

            // ── Text colors ────────────────────────────────────────────────
            text: TEXT,
            text_secondary: SUBTLE,
            text_placeholder: MUTED,

            text_colored_button: BASE,

            // ── Borders ────────────────────────────────────────────────────
            border: HIGHLIGHT_HIGH,
            border_subtle: HIGHLIGHT_MED,

            // ── Accent / Interactive ───────────────────────────────────────
            bluetooth_accent: FOAM,
            audio_accent: IRIS,
            dev_accent: PINE,

            // ── Status colors ──────────────────────────────────────────────
            danger: LOVE,
            // Darker love for hover.
            warning: GOLD,
            success: PINE,

            // ── Misc ───────────────────────────────────────────────────────
            error_background: Hsla {
                h: 343.0 / 360.0,
                s: 0.35,
                l: 0.55,
                a: 0.12,
            },
            // Love at 12% opacity — light enough to sit behind text.
            hover_overlay: Hsla {
                h: 25.0 / 360.0,
                s: 0.35,
                l: 0.93,
                a: 0.50,
            },
            // Highlight Low at 50%.
            selection_background: Hsla {
                h: 268.0 / 360.0,
                s: 0.21,
                l: 0.57,
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
