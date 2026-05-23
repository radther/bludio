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
    icon("bolt")
}

/// Placeholder page icon (plus — generic add/new page indicator).
pub fn page_placeholder() -> gpui::Svg {
    icon("plus")
}
