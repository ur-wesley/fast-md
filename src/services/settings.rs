use crate::types::AppSettings;
use eyre::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const APP_CONFIG_DIR_NAME: &str = "fast-md";
const SETTINGS_FILE_NAME: &str = "settings.json";

/// Resolve the platform-specific standard configuration directory for fast-md.
#[must_use]
pub fn get_settings_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            return PathBuf::from(appdata).join(APP_CONFIG_DIR_NAME);
        }
        if let Ok(userprofile) = env::var("USERPROFILE") {
            return PathBuf::from(userprofile)
                .join("AppData")
                .join("Roaming")
                .join(APP_CONFIG_DIR_NAME);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(APP_CONFIG_DIR_NAME);
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg_config).join(APP_CONFIG_DIR_NAME);
        }
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(".config").join(APP_CONFIG_DIR_NAME);
        }
    }

    // Fallback to local working directory
    env::current_dir().map_or_else(
        |_| PathBuf::from(".").join(APP_CONFIG_DIR_NAME),
        |cwd| cwd.join(format!(".{APP_CONFIG_DIR_NAME}")),
    )
}

/// Retrieve the absolute path to the persistent `settings.json` file.
#[must_use]
pub fn get_settings_file_path() -> PathBuf {
    get_settings_dir().join(SETTINGS_FILE_NAME)
}

/// Load settings from disk. If the file does not exist, writes default settings to disk and returns them.
/// If parsing fails, logs the error and gracefully returns the default settings.
#[must_use]
pub fn load_settings() -> AppSettings {
    let path = get_settings_file_path();
    load_settings_from_path(&path)
}

/// Load settings from a specific path.
#[must_use]
pub fn load_settings_from_path(path: &Path) -> AppSettings {
    if !path.exists() {
        let defaults = AppSettings::default();
        let _ = save_settings_to_path(path, &defaults);
        return defaults;
    }

    fs::read_to_string(path).map_or_else(
        |_| AppSettings::default(),
        |content| serde_json::from_str::<AppSettings>(&content).unwrap_or_default(),
    )
}

/// Save settings safely to the standard persistent location.
pub fn save_settings(settings: &AppSettings) -> Result<()> {
    let path = get_settings_file_path();
    save_settings_to_path(&path, settings)
}

/// Save settings safely to a specified file path.
pub fn save_settings_to_path(path: &Path, settings: &AppSettings) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory at {}", parent.display()))?;
        }
    }

    let json_content = serde_json::to_string_pretty(settings)
        .with_context(|| "Failed to serialize AppSettings to JSON")?;

    fs::write(path, json_content)
        .with_context(|| format!("Failed to write settings to {}", path.display()))?;

    Ok(())
}

/// Open the `settings.json` file in the operating system's default text editor.
pub fn open_settings_in_editor() {
    let path = get_settings_file_path();
    if !path.exists() {
        let defaults = AppSettings::default();
        let _ = save_settings_to_path(&path, &defaults);
    }

    let path_str = path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path_str])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&path_str).spawn();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(&path_str).spawn();
    }
}

/// Reveal the configuration folder containing `settings.json` in the OS file explorer.
pub fn reveal_settings_folder() {
    let dir = get_settings_dir();
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }

    let dir_str = dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(&dir_str).spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&dir_str).spawn();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(&dir_str).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AppTheme, SidebarTab};

    #[test]
    fn test_settings_save_and_load_roundtrip() {
        let temp_dir = env::temp_dir().join(format!("fast_md_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
        let settings_path = temp_dir.join("test_settings.json");

        let mut initial_settings = AppSettings {
            theme: AppTheme::CatppuccinMocha,
            primary_color: Some("#123456".to_string()),
            is_full_width: true,
            zoom_level: 130,
            sidebar_tab: SidebarTab::Files,
            auto_reload: false,
            sticky_headers: true,
            font_size: 18,
            ..Default::default()
        };
        initial_settings.add_recent_file(PathBuf::from("test1.md"));
        initial_settings.add_recent_folder(PathBuf::from("/test/folder"));

        assert!(save_settings_to_path(&settings_path, &initial_settings).is_ok());
        assert!(settings_path.exists());

        let loaded = load_settings_from_path(&settings_path);
        assert_eq!(loaded, initial_settings);

        // Cleanup
        let _ = fs::remove_file(&settings_path);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_corrupted_settings_fallback() {
        let temp_dir = env::temp_dir().join(format!("fast_md_corrupt_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
        let settings_path = temp_dir.join("corrupt_settings.json");

        let _ = fs::create_dir_all(&temp_dir);
        let _ = fs::write(&settings_path, "{ invalid json content @#$$%^ }");

        let loaded = load_settings_from_path(&settings_path);
        assert_eq!(loaded, AppSettings::default());

        let _ = fs::remove_file(&settings_path);
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
