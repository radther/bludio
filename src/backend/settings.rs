//! Application settings: persistence, global access, and theme integration.
//!
//! Provides a single GPUI global (`GlobalSettings`) that holds the user's
//! settings and the cached active theme. Any page can read settings or the
//! active theme via free functions, and any page can mutate settings via
//! `update_settings()` which automatically persists and re-renders.

use gpui::{App, BorrowAppContext, Global};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ui::theme::{TextStyleSet, Theme, theme_for_id};

// ── Theme mode ─────────────────────────────────────────────────────────────

/// Light or dark theme preference.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ThemeMode {
    #[default]
    Light,
    Dark,
}

// ── Settings struct ────────────────────────────────────────────────────────

/// User-facing settings persisted to disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub theme_mode: ThemeMode,
    pub light_theme_id: String,
    pub dark_theme_id: String,
    pub disable_animations: bool,
    pub font_family: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Light,
            light_theme_id: "rose-pine-dawn".to_string(),
            dark_theme_id: "rose-pine".to_string(),
            disable_animations: false,
            font_family: "Noto Sans".to_string(),
        }
    }
}

impl Settings {
    /// Load settings from `~/.config/bludio/settings.json`.
    /// Falls back to defaults if the file is missing or malformed.
    pub(crate) fn load() -> Self {
        let path = config_path();
        if let Ok(contents) = std::fs::read_to_string(&path)
            && let Ok(settings) = serde_json::from_str(&contents)
        {
            return settings;
        }
        Self::default()
    }

    /// Save settings to `~/.config/bludio/settings.json`.
    pub(crate) fn save(&self) {
        let dir = config_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("[settings] Failed to create config dir: {e}");
            return;
        }

        let path = dir.join("settings.json");
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    eprintln!("[settings] Failed to write settings: {e}");
                }
            }
            Err(e) => eprintln!("[settings] Failed to serialize settings: {e}"),
        }
    }
}

// ── Global settings ────────────────────────────────────────────────────────

/// GPUI global holding the active settings and cached theme.
///
/// The active theme is derived from `theme_mode` + the appropriate theme ID
/// and is recomputed automatically whenever settings change.
pub(crate) struct GlobalSettings {
    pub settings: Settings,
    pub active_theme: Arc<Theme>,
}

impl Global for GlobalSettings {}

impl GlobalSettings {
    /// Initialize from loaded settings, computing the active theme.
    pub(crate) fn new(mut settings: Settings) -> Self {
        let active_theme = Self::compute_theme(&mut settings);
        Self {
            settings,
            active_theme,
        }
    }

    /// Recompute the active theme from current settings.
    /// Falls back to default themes if the saved ID is not in the registry.
    fn compute_theme(settings: &mut Settings) -> Arc<Theme> {
        let id = match settings.theme_mode {
            ThemeMode::Light => &settings.light_theme_id,
            ThemeMode::Dark => &settings.dark_theme_id,
        };
        if let Some(theme) = theme_for_id(id) {
            return theme;
        }
        // Fallback: update the settings with the default theme ID
        let fallback_id = match settings.theme_mode {
            ThemeMode::Light => "rose-pine-dawn",
            ThemeMode::Dark => "rose-pine",
        };
        match settings.theme_mode {
            ThemeMode::Light => settings.light_theme_id = fallback_id.to_string(),
            ThemeMode::Dark => settings.dark_theme_id = fallback_id.to_string(),
        }
        theme_for_id(fallback_id).expect("default theme must exist")
    }
}

// ── Public accessors ─────────────────────────────────────────────────────

/// Retrieve the current active theme.
///
/// Panics if `GlobalSettings` has not been initialized.
pub(crate) fn theme(cx: &App) -> &Arc<Theme> {
    &cx.global::<GlobalSettings>().active_theme
}

/// Retrieve the current settings.
///
/// Panics if `GlobalSettings` has not been initialized.
pub(crate) fn settings(cx: &App) -> &Settings {
    &cx.global::<GlobalSettings>().settings
}

/// Retrieve the current text styles.
///
/// Panics if `GlobalSettings` has not been initialized.
///
/// Today this returns the default set. When text styles become a user
/// setting, this will read from the settings global instead.
pub(crate) fn text_styles(_cx: &App) -> &'static TextStyleSet {
    static DEFAULT: std::sync::LazyLock<TextStyleSet> =
        std::sync::LazyLock::new(TextStyleSet::default);
    &DEFAULT
}

/// Background save channel — a single thread handles all settings writes.
static SETTINGS_SAVE_TX: std::sync::LazyLock<std::sync::mpsc::Sender<Settings>> =
    std::sync::LazyLock::new(|| {
        let (tx, rx) = std::sync::mpsc::channel::<Settings>();
        std::thread::spawn(move || {
            while let Ok(settings) = rx.recv() {
                settings.save();
            }
        });
        tx
    });

/// Update settings, recompute the active theme, persist to disk, and trigger
/// a UI re-render.
///
/// This is the single entry point for all settings mutations.
pub(crate) fn update_settings<F>(f: F, cx: &mut App)
where
    F: FnOnce(&mut Settings),
{
    cx.update_global::<GlobalSettings, _>(|global, _cx| {
        f(&mut global.settings);
        global.active_theme = GlobalSettings::compute_theme(&mut global.settings);
    });

    let settings_to_save = cx.global::<GlobalSettings>().settings.clone();
    let _ = SETTINGS_SAVE_TX.send(settings_to_save);
}

// ── Config path helpers ──────────────────────────────────────────────────

fn config_dir() -> std::path::PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| {
            std::env::var("HOME").map(|home| {
                std::path::PathBuf::from(home)
                    .join(".config")
                    .join("bludio")
            })
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp").join("bludio"))
}

fn config_path() -> std::path::PathBuf {
    config_dir().join("settings.json")
}
