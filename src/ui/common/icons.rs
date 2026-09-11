//! Lucide icon helpers for GPUI.
//!
//! Icons are embedded into the binary via `rust-embed` and loaded through
//! gpui's `AssetSource` at runtime.

use gpui::{Styled, px, svg};

/// Standard icon size for inline UI icons (matches text line-height at body size).
const ICON_SIZE: f32 = 16.0;

/// Create an SVG icon element from the embedded assets.
fn icon(name: &str) -> gpui::Svg {
    svg()
        .path(format!("icons/{}.svg", name))
        .w(px(ICON_SIZE))
        .h(px(ICON_SIZE))
}

pub fn bluetooth() -> gpui::Svg {
    icon("radar")
}

pub fn audio_output() -> gpui::Svg {
    icon("speaker")
}

pub fn audio_input() -> gpui::Svg {
    icon("mic")
}

pub fn audio_card() -> gpui::Svg {
    icon("form")
}

#[cfg(debug_assertions)]
pub fn text_field_test() -> gpui::Svg {
    icon("test-tube-diagonal")
}

pub fn refresh_ccw() -> gpui::Svg {
    icon("refresh-ccw")
}

pub fn chevron_down() -> gpui::Svg {
    icon("chevron-down")
}

pub fn bolt() -> gpui::Svg {
    icon("bolt")
}

pub fn check() -> gpui::Svg {
    icon("check")
}

pub fn merge() -> gpui::Svg {
    icon("merge")
}

pub fn power() -> gpui::Svg {
    icon("power")
}
