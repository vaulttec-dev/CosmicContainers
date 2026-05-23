//! User preferences persisted under `~/.config/{APP_ID}/settings.conf`.

use std::fs;
use std::path::PathBuf;

use crate::APP_ID;

/// Default font size for the running-container count on the panel badge.
pub const DEFAULT_PANEL_COUNT_FONT_SIZE: u16 = 14;

pub const MIN_PANEL_COUNT_FONT_SIZE: u16 = 10;
pub const MAX_PANEL_COUNT_FONT_SIZE: u16 = 28;

const KEY_PANEL_COUNT_FONT_SIZE: &str = "panel_count_font_size";

/// Persisted applet preferences.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserSettings {
    pub panel_count_font_size: u16,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            panel_count_font_size: DEFAULT_PANEL_COUNT_FONT_SIZE,
        }
    }
}

impl UserSettings {
    pub fn clamp_panel_count_font_size(size: u16) -> u16 {
        size.clamp(MIN_PANEL_COUNT_FONT_SIZE, MAX_PANEL_COUNT_FONT_SIZE)
    }

    pub fn set_panel_count_font_size(&mut self, size: u16) {
        self.panel_count_font_size = Self::clamp_panel_count_font_size(size);
    }
}

/// Load settings from disk, falling back to defaults on missing or invalid files.
pub fn load() -> UserSettings {
    migrate_legacy_config();
    let Some(path) = config_path() else {
        return UserSettings::default();
    };
    let Ok(contents) = fs::read_to_string(path) else {
        return UserSettings::default();
    };
    parse(&contents).unwrap_or_default()
}

pub fn save(settings: &UserSettings) -> Result<(), String> {
    let path = config_path().ok_or_else(|| "HOME is not set".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir {}: {e}", parent.display()))?;
    }
    let contents = format!(
        "# {APP_ID} user settings\n{KEY_PANEL_COUNT_FONT_SIZE}={}\n",
        settings.panel_count_font_size
    );
    fs::write(&path, contents).map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

fn config_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join(APP_ID)
            .join("settings.conf")
    })
}

fn legacy_config_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("linux-pop-extension")
            .join("settings.conf")
    })
}

/// Copy legacy settings from the pre-Flatpak config directory when needed.
fn migrate_legacy_config() {
    let Some(new_path) = config_path() else {
        return;
    };
    if new_path.exists() {
        return;
    }
    let Some(legacy_path) = legacy_config_path() else {
        return;
    };
    if !legacy_path.exists() {
        return;
    }
    if let Some(parent) = new_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::copy(&legacy_path, &new_path);
}

fn parse(contents: &str) -> Option<UserSettings> {
    let mut settings = UserSettings::default();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=')?;
        if key.trim() == KEY_PANEL_COUNT_FONT_SIZE {
            let size: u16 = value.trim().parse().ok()?;
            settings.set_panel_count_font_size(size);
        }
    }
    Some(settings)
}

#[cfg(test)]
mod tests {
    use super::{MAX_PANEL_COUNT_FONT_SIZE, MIN_PANEL_COUNT_FONT_SIZE, UserSettings, parse};

    #[test]
    fn clamp_panel_count_font_size_bounds() {
        assert_eq!(
            UserSettings::clamp_panel_count_font_size(4),
            MIN_PANEL_COUNT_FONT_SIZE
        );
        assert_eq!(
            UserSettings::clamp_panel_count_font_size(99),
            MAX_PANEL_COUNT_FONT_SIZE
        );
        assert_eq!(UserSettings::clamp_panel_count_font_size(16), 16);
    }

    #[test]
    fn parse_settings_file() {
        let settings = parse("panel_count_font_size=20\n").expect("parse");
        assert_eq!(settings.panel_count_font_size, 20);
    }
}
