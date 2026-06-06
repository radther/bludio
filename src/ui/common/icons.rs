//! Lucide icon helpers for GPUI.
//!
//! Icons are loaded from standard data directories or the crate directory
//! during development.

use gpui::{SharedString, Styled, px, svg};
use std::path::Path;

/// Standard icon size for inline UI icons (matches text line-height at body size).
const ICON_SIZE: f32 = 16.0;

/// Resolve the path to an icon SVG, checking install locations and falling back
/// to the crate directory for development builds.
fn resolve_icon_path(name: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        format!("{}/.local/share/bludio/icons/{}.svg", home, name),
        format!("/usr/share/bludio/icons/{}.svg", name),
        format!("{}/icons/{}.svg", env!("CARGO_MANIFEST_DIR"), name),
    ];
    for path in &candidates {
        if Path::new(path).exists() {
            return path.clone();
        }
    }
    // Fallback to the first candidate (will show an error in the UI if missing)
    candidates[0].clone()
}

/// Create an SVG icon element from the local icons directory.
fn icon(name: &str) -> gpui::Svg {
    let path = resolve_icon_path(name);
    svg()
        .external_path(SharedString::from(path))
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
