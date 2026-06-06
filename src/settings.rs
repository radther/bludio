//! Application settings: persistence, global access, and theme integration.
//!
//! Provides a single GPUI global (`GlobalSettings`) that holds the user's
//! settings and the cached active theme. Any page can read settings or the
//! active theme via free functions, and any page can mutate settings via
//! `update_settings()` which automatically persists and re-renders.

use gpui::{App, BorrowAppContext, Global};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ui::theme::{Theme, theme_for_id};

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
    pub(crate) fn new(settings: Settings) -> Self {
        let active_theme = Self::compute_theme(&settings);
        Self {
            settings,
            active_theme,
        }
    }

    /// Recompute the active theme from current settings.
    fn compute_theme(settings: &Settings) -> Arc<Theme> {
        let id = match settings.theme_mode {
            ThemeMode::Light => &settings.light_theme_id,
            ThemeMode::Dark => &settings.dark_theme_id,
        };
        let mut theme = theme_for_id(id)
            .unwrap_or_else(|| theme_for_id("rose-pine-dawn").expect("default theme must exist"));
        Arc::make_mut(&mut theme).font_family = settings.font_family.clone().into();
        theme
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
        global.active_theme = GlobalSettings::compute_theme(&global.settings);
    });

    // Fire-and-forget background save.
    let settings_to_save = cx.global::<GlobalSettings>().settings.clone();
    std::thread::spawn(move || {
        settings_to_save.save();
    });
}

// ── Config path helpers ──────────────────────────────────────────────────

fn config_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(|home| {
            std::path::PathBuf::from(home)
                .join(".config")
                .join("bludio")
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp").join("bludio"))
}

fn config_path() -> std::path::PathBuf {
    config_dir().join("settings.json")
}
