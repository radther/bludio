//! Lucide icon helpers for GPUI.
//!
//! Icons are loaded from `icons/<name>.svg` relative to the crate directory.

use gpui::{SharedString, Styled, px, svg};

/// Standard icon size for inline UI icons (matches text line-height at body size).
const ICON_SIZE: f32 = 16.0;

/// Create an SVG icon element from the local icons directory.
fn icon(name: &str) -> gpui::Svg {
    let path = format!("{}/icons/{}.svg", env!("CARGO_MANIFEST_DIR"), name);
    svg()
        .external_path(SharedString::from(path))
        .w(px(ICON_SIZE))
        .h(px(ICON_SIZE))
}

/// Bluetooth icon (bolt — Lucide's lightning bolt approximates the Bluetooth rune).
pub fn bluetooth() -> gpui::Svg {
    icon("radar")
}

/// Audio output icon (speaker — Lucide's volume-2 approximates a speaker).
pub fn audio_output() -> gpui::Svg {
    icon("speaker")
}

/// Audio input icon (mic — Lucide's mic approximates a microphone).
pub fn audio_input() -> gpui::Svg {
    icon("mic")
}

/// Audio card configuration icon (settings gear).
pub fn audio_card() -> gpui::Svg {
    icon("form")
}

/// Text field test page icon.
pub fn text_field_test() -> gpui::Svg {
    icon("test-tube-diagonal")
}
