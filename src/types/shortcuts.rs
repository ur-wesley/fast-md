use serde::{Deserialize, Serialize};

/// Identifies an application action that can be bound to a keyboard shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutAction {
    Save,
    SaveAs,
    OpenFile,
    OpenFolder,
    NewTab,
    CloseTab,
    ToggleSidebar,
    ToggleZen,
    CycleMode,
    Find,
    FormatDocument,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    ToggleSettings,
}

impl ShortcutAction {
    /// Return all supported shortcut actions.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Save,
            Self::SaveAs,
            Self::OpenFile,
            Self::OpenFolder,
            Self::NewTab,
            Self::CloseTab,
            Self::ToggleSidebar,
            Self::ToggleZen,
            Self::CycleMode,
            Self::Find,
            Self::FormatDocument,
            Self::ZoomIn,
            Self::ZoomOut,
            Self::ResetZoom,
            Self::ToggleSettings,
        ]
    }

    /// Return the category of the shortcut action for settings grouping.
    #[must_use]
    pub const fn category(self) -> ShortcutCategory {
        match self {
            Self::Save | Self::SaveAs | Self::OpenFile | Self::OpenFolder | Self::NewTab | Self::CloseTab => {
                ShortcutCategory::FileAndTabs
            }
            Self::ToggleSidebar | Self::ToggleZen | Self::CycleMode => ShortcutCategory::LayoutAndModes,
            Self::Find | Self::FormatDocument => ShortcutCategory::EditorAndSearch,
            Self::ZoomIn | Self::ZoomOut | Self::ResetZoom | Self::ToggleSettings => {
                ShortcutCategory::ViewAndPreferences
            }
        }
    }

    /// Default string key combination for this action.
    #[must_use]
    pub const fn default_binding(self) -> &'static str {
        match self {
            Self::Save => "Ctrl+S",
            Self::SaveAs => "Ctrl+Shift+S",
            Self::OpenFile => "Ctrl+O",
            Self::OpenFolder => "Ctrl+Shift+O",
            Self::NewTab => "Ctrl+T",
            Self::CloseTab => "Ctrl+W",
            Self::ToggleSidebar => "Ctrl+B",
            Self::ToggleZen => "Ctrl+Shift+F",
            Self::CycleMode => "Ctrl+E",
            Self::Find => "Ctrl+F",
            Self::FormatDocument => "Shift+Alt+F",
            Self::ZoomIn => "Ctrl+=",
            Self::ZoomOut => "Ctrl+-",
            Self::ResetZoom => "Ctrl+0",
            Self::ToggleSettings => "Ctrl+,",
        }
    }
}

/// Category grouping for shortcut actions in preferences UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutCategory {
    FileAndTabs,
    LayoutAndModes,
    EditorAndSearch,
    ViewAndPreferences,
}

/// Parsed representation of a key combination.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortcutKey {
    pub ctrl: bool,
    pub meta: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: String,
}

impl ShortcutKey {
    /// Parse a human-readable shortcut string (e.g. "Ctrl+Shift+S", "Alt+Shift+F", "Escape", "Cmd+O").
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut ctrl = false;
        let mut meta = false;
        let mut alt = false;
        let mut shift = false;
        let mut key_part = None;

        // Split by '+' while respecting '+' as the key itself (e.g. "Ctrl++" or "Ctrl+Shift++")
        let parts: Vec<&str> = trimmed.split('+').collect();
        let mut i = 0;
        while i < parts.len() {
            let part = parts[i];
            if part.is_empty() {
                // If consecutive pluses resulted in an empty part (e.g. "Ctrl++"), the key is "+"
                key_part = Some("+".to_string());
                i += 1;
                continue;
            }

            match part.trim().to_ascii_lowercase().as_str() {
                "ctrl" | "control" => ctrl = true,
                "meta" | "cmd" | "command" | "win" | "super" => meta = true,
                "alt" | "opt" | "option" => alt = true,
                "shift" => shift = true,
                other => {
                    key_part = Some(normalize_key_name(other));
                }
            }
            i += 1;
        }

        key_part.map(|key| Self {
            ctrl,
            meta,
            alt,
            shift,
            key,
        })
    }

    /// Check if this shortcut combination matches incoming key event parameters.
    #[must_use]
    pub fn matches(&self, raw_key: &str, is_ctrl: bool, is_alt: bool, is_shift: bool, is_meta: bool) -> bool {
        // Match modifier states
        let ctrl_match = self.ctrl == (is_ctrl || (self.meta && is_meta));
        let alt_match = self.alt == is_alt;
        let shift_match = self.shift == is_shift;
        let meta_match = if self.meta {
            is_meta || is_ctrl
        } else {
            // If shortcut does not require meta, meta shouldn't be pressed unless it counts as ctrl on some OS
            !is_meta || is_ctrl
        };

        if !ctrl_match || !alt_match || !shift_match || !meta_match {
            return false;
        }

        let normalized_raw = normalize_key_name(raw_key);
        self.key.eq_ignore_ascii_case(&normalized_raw)
            || match_key_aliases(&self.key, &normalized_raw)
    }

    /// Format as a canonical human-readable string (e.g. "Ctrl+Shift+S").
    #[must_use]
    pub fn to_canonical_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.meta {
            parts.push("Cmd");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        let formatted_key = capitalize_key(&self.key);
        parts.push(&formatted_key);
        parts.join("+")
    }
}

/// Helper to normalize key names for comparison.
fn normalize_key_name(k: &str) -> String {
    let lower = k.trim().to_ascii_lowercase();
    match lower.as_str() {
        "esc" | "escape" => "Escape".to_string(),
        "enter" | "return" => "Enter".to_string(),
        "tab" => "Tab".to_string(),
        "space" | " " => "Space".to_string(),
        "backspace" => "Backspace".to_string(),
        "delete" | "del" => "Delete".to_string(),
        "arrowup" | "up" => "ArrowUp".to_string(),
        "arrowdown" | "down" => "ArrowDown".to_string(),
        "arrowleft" | "left" => "ArrowLeft".to_string(),
        "arrowright" | "right" => "ArrowRight".to_string(),
        "=" | "equal" => "=".to_string(),
        "+" | "plus" => "+".to_string(),
        "-" | "minus" => "-".to_string(),
        "," | "comma" => ",".to_string(),
        "." | "period" => ".".to_string(),
        "/" | "slash" => "/".to_string(),
        "\\" | "backslash" => "\\".to_string(),
        "[" | "bracketleft" => "[".to_string(),
        "]" | "bracketright" => "]".to_string(),
        ";" | "semicolon" => ";".to_string(),
        "'" | "quote" => "'".to_string(),
        "`" | "backquote" => "`".to_string(),
        other => {
            if other.starts_with('f') && other.len() <= 3 && other[1..].chars().all(|c| c.is_ascii_digit()) {
                other.to_ascii_uppercase()
            } else if other.len() == 1 {
                other.to_ascii_uppercase()
            } else {
                other.to_string()
            }
        }
    }
}

/// Check equivalent key aliases (e.g. "=" and "+", "," and "<").
fn match_key_aliases(expected: &str, actual: &str) -> bool {
    match expected {
        "=" | "+" => actual == "=" || actual == "+",
        "," | "<" => actual == "," || actual == "<",
        _ => false,
    }
}

/// Format key name for display.
fn capitalize_key(k: &str) -> String {
    if k.len() == 1 {
        k.to_ascii_uppercase()
    } else {
        k.to_string()
    }
}

/// Configurable shortcut keybindings for all application actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    #[serde(default = "default_save")]
    pub save: String,

    #[serde(default = "default_save_as")]
    pub save_as: String,

    #[serde(default = "default_open_file")]
    pub open_file: String,

    #[serde(default = "default_open_folder")]
    pub open_folder: String,

    #[serde(default = "default_new_tab")]
    pub new_tab: String,

    #[serde(default = "default_close_tab")]
    pub close_tab: String,

    #[serde(default = "default_toggle_sidebar")]
    pub toggle_sidebar: String,

    #[serde(default = "default_toggle_zen")]
    pub toggle_zen: String,

    #[serde(default = "default_cycle_mode")]
    pub cycle_mode: String,

    #[serde(default = "default_find")]
    pub find: String,

    #[serde(default = "default_format_document")]
    pub format_document: String,

    #[serde(default = "default_zoom_in")]
    pub zoom_in: String,

    #[serde(default = "default_zoom_out")]
    pub zoom_out: String,

    #[serde(default = "default_reset_zoom")]
    pub reset_zoom: String,

    #[serde(default = "default_toggle_settings")]
    pub toggle_settings: String,
}

fn default_save() -> String {
    ShortcutAction::Save.default_binding().to_string()
}
fn default_save_as() -> String {
    ShortcutAction::SaveAs.default_binding().to_string()
}
fn default_open_file() -> String {
    ShortcutAction::OpenFile.default_binding().to_string()
}
fn default_open_folder() -> String {
    ShortcutAction::OpenFolder.default_binding().to_string()
}
fn default_new_tab() -> String {
    ShortcutAction::NewTab.default_binding().to_string()
}
fn default_close_tab() -> String {
    ShortcutAction::CloseTab.default_binding().to_string()
}
fn default_toggle_sidebar() -> String {
    ShortcutAction::ToggleSidebar.default_binding().to_string()
}
fn default_toggle_zen() -> String {
    ShortcutAction::ToggleZen.default_binding().to_string()
}
fn default_cycle_mode() -> String {
    ShortcutAction::CycleMode.default_binding().to_string()
}
fn default_find() -> String {
    ShortcutAction::Find.default_binding().to_string()
}
fn default_format_document() -> String {
    ShortcutAction::FormatDocument.default_binding().to_string()
}
fn default_zoom_in() -> String {
    ShortcutAction::ZoomIn.default_binding().to_string()
}
fn default_zoom_out() -> String {
    ShortcutAction::ZoomOut.default_binding().to_string()
}
fn default_reset_zoom() -> String {
    ShortcutAction::ResetZoom.default_binding().to_string()
}
fn default_toggle_settings() -> String {
    ShortcutAction::ToggleSettings.default_binding().to_string()
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            save: default_save(),
            save_as: default_save_as(),
            open_file: default_open_file(),
            open_folder: default_open_folder(),
            new_tab: default_new_tab(),
            close_tab: default_close_tab(),
            toggle_sidebar: default_toggle_sidebar(),
            toggle_zen: default_toggle_zen(),
            cycle_mode: default_cycle_mode(),
            find: default_find(),
            format_document: default_format_document(),
            zoom_in: default_zoom_in(),
            zoom_out: default_zoom_out(),
            reset_zoom: default_reset_zoom(),
            toggle_settings: default_toggle_settings(),
        }
    }
}

impl ShortcutsConfig {
    /// Retrieve the configured binding string for a specific action.
    #[must_use]
    pub fn get_binding(&self, action: ShortcutAction) -> &str {
        match action {
            ShortcutAction::Save => &self.save,
            ShortcutAction::SaveAs => &self.save_as,
            ShortcutAction::OpenFile => &self.open_file,
            ShortcutAction::OpenFolder => &self.open_folder,
            ShortcutAction::NewTab => &self.new_tab,
            ShortcutAction::CloseTab => &self.close_tab,
            ShortcutAction::ToggleSidebar => &self.toggle_sidebar,
            ShortcutAction::ToggleZen => &self.toggle_zen,
            ShortcutAction::CycleMode => &self.cycle_mode,
            ShortcutAction::Find => &self.find,
            ShortcutAction::FormatDocument => &self.format_document,
            ShortcutAction::ZoomIn => &self.zoom_in,
            ShortcutAction::ZoomOut => &self.zoom_out,
            ShortcutAction::ResetZoom => &self.reset_zoom,
            ShortcutAction::ToggleSettings => &self.toggle_settings,
        }
    }

    /// Set a custom keybinding string for an action.
    pub fn set_binding(&mut self, action: ShortcutAction, binding: String) {
        match action {
            ShortcutAction::Save => self.save = binding,
            ShortcutAction::SaveAs => self.save_as = binding,
            ShortcutAction::OpenFile => self.open_file = binding,
            ShortcutAction::OpenFolder => self.open_folder = binding,
            ShortcutAction::NewTab => self.new_tab = binding,
            ShortcutAction::CloseTab => self.close_tab = binding,
            ShortcutAction::ToggleSidebar => self.toggle_sidebar = binding,
            ShortcutAction::ToggleZen => self.toggle_zen = binding,
            ShortcutAction::CycleMode => self.cycle_mode = binding,
            ShortcutAction::Find => self.find = binding,
            ShortcutAction::FormatDocument => self.format_document = binding,
            ShortcutAction::ZoomIn => self.zoom_in = binding,
            ShortcutAction::ZoomOut => self.zoom_out = binding,
            ShortcutAction::ResetZoom => self.reset_zoom = binding,
            ShortcutAction::ToggleSettings => self.toggle_settings = binding,
        }
    }

    /// Reset a single action back to its default keybinding.
    pub fn reset_action(&mut self, action: ShortcutAction) {
        self.set_binding(action, action.default_binding().to_string());
    }

    /// Reset all shortcuts back to default values.
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }

    /// Check if an incoming keyboard event matches any configured shortcut action.
    #[must_use]
    pub fn match_event(
        &self,
        raw_key: &str,
        is_ctrl: bool,
        is_alt: bool,
        is_shift: bool,
        is_meta: bool,
    ) -> Option<ShortcutAction> {
        // Check configured actions
        for &action in ShortcutAction::all() {
            let binding = self.get_binding(action);
            if let Some(parsed) = ShortcutKey::parse(binding) {
                if parsed.matches(raw_key, is_ctrl, is_alt, is_shift, is_meta) {
                    return Some(action);
                }
            }
        }

        // Check fallback built-in aliases (e.g. Ctrl+Shift+I for format document)
        if (is_ctrl || is_meta) && is_shift && (raw_key.eq_ignore_ascii_case("i")) {
            return Some(ShortcutAction::FormatDocument);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_parsing_and_matching() {
        let key = ShortcutKey::parse("Ctrl+Shift+S").unwrap_or_else(|| panic!("failed parse"));
        assert!(key.ctrl);
        assert!(key.shift);
        assert!(!key.alt);
        assert!(!key.meta);
        assert_eq!(key.key, "S");
        assert!(key.matches("s", true, false, true, false));
        assert!(key.matches("S", true, false, true, false));
        assert!(!key.matches("s", true, false, false, false));
        assert!(!key.matches("o", true, false, true, false));
    }

    #[test]
    fn test_alt_shift_shortcut() {
        let key = ShortcutKey::parse("Shift+Alt+F").unwrap_or_else(|| panic!("failed parse"));
        assert!(!key.ctrl);
        assert!(key.shift);
        assert!(key.alt);
        assert_eq!(key.key, "F");
        assert!(key.matches("f", false, true, true, false));
        assert!(key.matches("F", false, true, true, false));
    }

    #[test]
    fn test_zoom_plus_equal_matching() {
        let key = ShortcutKey::parse("Ctrl+=").unwrap_or_else(|| panic!("failed parse"));
        assert!(key.ctrl);
        assert_eq!(key.key, "=");
        assert!(key.matches("=", true, false, false, false));
        assert!(key.matches("+", true, false, false, false));
    }

    #[test]
    fn test_shortcuts_config_match_event() {
        let config = ShortcutsConfig::default();
        assert_eq!(config.match_event("s", true, false, false, false), Some(ShortcutAction::Save));
        assert_eq!(config.match_event("S", true, false, true, false), Some(ShortcutAction::SaveAs));
        assert_eq!(config.match_event("o", true, false, false, false), Some(ShortcutAction::OpenFile));
        assert_eq!(config.match_event("O", true, false, true, false), Some(ShortcutAction::OpenFolder));
        assert_eq!(config.match_event("e", true, false, false, false), Some(ShortcutAction::CycleMode));
        assert_eq!(config.match_event("b", true, false, false, false), Some(ShortcutAction::ToggleSidebar));
        assert_eq!(config.match_event(",", true, false, false, false), Some(ShortcutAction::ToggleSettings));
        assert_eq!(config.match_event("f", false, true, true, false), Some(ShortcutAction::FormatDocument));
    }

    #[test]
    fn test_custom_shortcut_override() {
        let mut config = ShortcutsConfig::default();
        config.set_binding(ShortcutAction::Save, "Ctrl+Alt+S".to_string());
        assert_eq!(config.match_event("s", true, false, false, false), None);
        assert_eq!(config.match_event("s", true, true, false, false), Some(ShortcutAction::Save));

        config.reset_action(ShortcutAction::Save);
        assert_eq!(config.match_event("s", true, false, false, false), Some(ShortcutAction::Save));
    }
}
