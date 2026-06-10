//! JSON theme loader: parses `.json` theme files into `Theme` structs.
//!
//! Supports hex color notation (`#RRGGBB`) and semantic color mapping.

use std::collections::HashMap;

use gpui::Hsla;
use serde::Deserialize;

use super::types::{Appearance, Theme, ThemeColors};

// ── JSON schema ────────────────────────────────────────────────────────────

/// Raw JSON structure for a theme file.
#[derive(Deserialize, Debug)]
struct ThemeJson {
    id: String,
    display_name: String,
    colors: HashMap<String, String>,
    theme: HashMap<String, String>,
}

// ── Hex color parsing ──────────────────────────────────────────────────────

/// Parse a `#RRGGBB` hex string into an `Hsla` with alpha 1.0.
fn parse_hex_color(hex: &str) -> Result<Hsla, String> {
    let hex = hex.trim();
    if !hex.starts_with('#') || hex.len() != 7 {
        return Err(format!("Invalid hex color: {hex}"));
    }
    let r = u8::from_str_radix(&hex[1..3], 16).map_err(|e| format!("Invalid hex red: {e}"))?;
    let g = u8::from_str_radix(&hex[3..5], 16).map_err(|e| format!("Invalid hex green: {e}"))?;
    let b = u8::from_str_radix(&hex[5..7], 16).map_err(|e| format!("Invalid hex blue: {e}"))?;

    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) / 2.0;
    let s = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    Ok(Hsla {
        h: h / 360.0,
        s,
        l,
        a: 1.0,
    })
}

// ── Required semantic tokens ─────────────────────────────────────────────────

const REQUIRED_TOKENS: &[&str] = &[
    "background",
    "surface",
    "background_hover",
    "element_background",
    "element_hover",
    "input_background",
    "text",
    "text_secondary",
    "text_muted",
    "text_colored_button",
    "border",
    "bluetooth_accent",
    "audio_accent",
    "dev_accent",
    "danger",
    "warning",
    "success",
];

// ── Public API ───────────────────────────────────────────────────────────────

/// Load a theme from a JSON string and an appearance.
///
/// Validates that all required semantic tokens are present and that every
/// palette reference resolves to a color defined in the `colors` dictionary.
pub(crate) fn load_theme_from_json(content: &str, appearance: Appearance) -> Result<Theme, String> {
    let raw: ThemeJson =
        serde_json::from_str(content).map_err(|e| format!("JSON parse error: {e}"))?;

    // Parse all palette colors
    let mut palette: HashMap<String, Hsla> = HashMap::with_capacity(raw.colors.len());
    for (name, hex) in &raw.colors {
        let color = parse_hex_color(hex).map_err(|e| format!("Color '{name}': {e}"))?;
        palette.insert(name.clone(), color);
    }

    // Validate required semantic tokens
    for token in REQUIRED_TOKENS {
        if !raw.theme.contains_key(*token) {
            return Err(format!("Missing required semantic token: {token}"));
        }
    }

    // Validate no unknown tokens
    for token in raw.theme.keys() {
        if !REQUIRED_TOKENS.contains(&token.as_str()) {
            return Err(format!("Unknown semantic token: {token}"));
        }
    }

    // Resolve semantic mappings
    let resolve = |token: &str| -> Result<Hsla, String> {
        let color_name = raw
            .theme
            .get(token)
            .ok_or_else(|| format!("Missing token: {token}"))?;
        palette
            .get(color_name)
            .copied()
            .ok_or_else(|| format!("Token '{token}' references unknown color '{color_name}'"))
    };

    let colors = ThemeColors {
        background: resolve("background")?,
        surface: resolve("surface")?,
        background_hover: resolve("background_hover")?,
        element_background: resolve("element_background")?,
        element_hover: resolve("element_hover")?,
        input_background: resolve("input_background")?,
        text: resolve("text")?,
        text_secondary: resolve("text_secondary")?,
        text_muted: resolve("text_muted")?,
        text_colored_button: resolve("text_colored_button")?,
        border: resolve("border")?,
        bluetooth_accent: resolve("bluetooth_accent")?,
        audio_accent: resolve("audio_accent")?,
        dev_accent: resolve("dev_accent")?,
        danger: resolve("danger")?,
        warning: resolve("warning")?,
        success: resolve("success")?,
    };

    Ok(Theme {
        id: raw.id,
        display_name: raw.display_name,
        appearance,
        colors,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_theme_json() -> String {
        r##"{
            "id": "test-theme",
            "display_name": "Test Theme",
            "colors": {
                "base": "#faf4ed",
                "accent": "#ea9d34"
            },
            "theme": {
                "background": "base",
                "surface": "base",
                "background_hover": "base",
                "element_background": "base",
                "element_hover": "base",
                "input_background": "base",
                "text": "accent",
                "text_secondary": "accent",
                "text_muted": "accent",
                "text_colored_button": "base",
                "border": "accent",
                "bluetooth_accent": "accent",
                "audio_accent": "accent",
                "dev_accent": "accent",
                "danger": "accent",
                "warning": "accent",
                "success": "accent"
            }
        }"##
        .to_string()
    }

    #[test]
    fn test_load_valid_theme() {
        let theme = load_theme_from_json(&valid_theme_json(), Appearance::Light).unwrap();
        assert_eq!(theme.id, "test-theme");
        assert_eq!(theme.display_name, "Test Theme");
        assert_eq!(theme.appearance, Appearance::Light);
    }

    #[test]
    fn test_invalid_hex() {
        let mut json = valid_theme_json();
        json = json.replace("#faf4ed", "not-a-color");
        let result = load_theme_from_json(&json, Appearance::Light);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid hex color"));
    }

    #[test]
    fn test_missing_palette_reference() {
        let mut json = valid_theme_json();
        json = json.replace("\"background\": \"base\"", "\"background\": \"missing\"");
        let result = load_theme_from_json(&json, Appearance::Light);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown color"));
    }

    #[test]
    fn test_missing_semantic_token() {
        let mut json = valid_theme_json();
        json = json.replace("\"background\": \"base\",", "");
        let result = load_theme_from_json(&json, Appearance::Light);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Missing required semantic token")
        );
    }

    #[test]
    fn test_unknown_semantic_token() {
        let mut json = valid_theme_json();
        // Insert an unknown token inside the `theme` dictionary
        json = json.replace(
            "\"success\": \"accent\"",
            "\"success\": \"accent\",\n                \"unknown_token\": \"base\"",
        );
        let result = load_theme_from_json(&json, Appearance::Light);
        assert!(result.is_err(), "Expected error, got: {result:?}");
        assert!(result.unwrap_err().contains("Unknown semantic token"));
    }

    #[test]
    fn test_hex_color_parsing() {
        let hsla = parse_hex_color("#ff0000").unwrap();
        assert!((hsla.h - 0.0 / 360.0).abs() < 0.01);
        assert!((hsla.s - 1.0).abs() < 0.01);
        assert!((hsla.l - 0.5).abs() < 0.01);
        assert!((hsla.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hex_white() {
        let hsla = parse_hex_color("#ffffff").unwrap();
        assert!((hsla.l - 1.0).abs() < 0.01);
        assert!(hsla.s < 0.01);
    }

    #[test]
    fn test_hex_black() {
        let hsla = parse_hex_color("#000000").unwrap();
        assert!((hsla.l - 0.0).abs() < 0.01);
        assert!(hsla.s < 0.01);
    }
}
