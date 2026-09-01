use super::AppStore;
use crate::types::{AppTheme, Language, SidebarPosition, SidebarTab, UpdateStatus};

impl AppStore {
    /// Zoom in by 10% (up to 250%).
    pub fn zoom_in(&mut self) {
        if self.zoom_level < 250 {
            self.zoom_level = self.zoom_level.saturating_add(10);
            self.settings.zoom_level = self.zoom_level;
            self.persist_settings();
        }
    }

    /// Zoom out by 10% (down to 50%).
    pub fn zoom_out(&mut self) {
        if self.zoom_level > 50 {
            self.zoom_level = self.zoom_level.saturating_sub(10);
            self.settings.zoom_level = self.zoom_level;
            self.persist_settings();
        }
    }

    /// Reset zoom to default 100%.
    pub fn reset_zoom(&mut self) {
        self.zoom_level = 100;
        self.settings.zoom_level = self.zoom_level;
        self.persist_settings();
    }

    /// Toggle Zen mode.
    pub const fn toggle_zen(&mut self) {
        self.is_zen = !self.is_zen;
    }

    /// Set Zen mode explicitly.
    pub const fn set_zen(&mut self, zen: bool) {
        self.is_zen = zen;
    }

    /// Toggle sidebar visibility.
    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
        self.settings.show_sidebar = self.show_sidebar;
        self.persist_settings();
    }

    /// Toggle reading column vs full width.
    pub fn toggle_full_width(&mut self) {
        self.is_full_width = !self.is_full_width;
        self.settings.is_full_width = self.is_full_width;
        self.persist_settings();
    }

    /// Toggle search overlay.
    #[allow(dead_code)]
    pub const fn toggle_search(&mut self) {
        self.show_search = !self.show_search;
    }

    /// Set workspace find-in-files overlay visibility.
    pub const fn set_find_in_files(&mut self, show: bool) {
        self.show_find_in_files = show;
    }

    pub const fn set_quick_open(&mut self, show: bool) {
        self.show_quick_open = show;
    }

    /// Set theme and persist.
    pub fn set_theme(&mut self, theme: AppTheme) {
        self.theme = theme;
        self.settings.theme = theme;
        self.persist_settings();
    }

    /// Set custom primary color (or None to reset to default) and persist.
    pub fn set_primary_color(&mut self, color: Option<String>) {
        self.primary_color.clone_from(&color);
        self.settings.primary_color = color;
        self.persist_settings();
    }

    /// Retrieve the effective primary accent color.
    #[must_use]
    pub fn effective_primary_color(&self) -> &str {
        self.primary_color
            .as_deref()
            .unwrap_or_else(|| self.theme.default_accent())
    }

    /// Set sidebar tab mode and persist.
    pub fn set_sidebar_tab(&mut self, tab: SidebarTab) {
        self.sidebar_tab = tab;
        self.settings.sidebar_tab = tab;
        self.persist_settings();
    }

    /// Set sidebar position (Left or Right) and persist.
    pub fn set_sidebar_position(&mut self, pos: SidebarPosition) {
        self.sidebar_position = pos;
        self.settings.sidebar_position = pos;
        self.persist_settings();
    }

    /// Set sidebar width and persist.
    pub fn set_sidebar_width(&mut self, width: u32) {
        let clamped = width.clamp(180, 560);
        self.sidebar_width = clamped;
        self.settings.sidebar_width = clamped;
        self.persist_settings();
    }

    /// Set auto-reload setting and persist.
    pub fn set_auto_reload(&mut self, auto_reload: bool) {
        self.settings.auto_reload = auto_reload;
        self.persist_settings();
    }

    /// Set sticky markdown headers setting and persist.
    pub fn set_sticky_headers(&mut self, sticky: bool) {
        self.sticky_headers = sticky;
        self.settings.sticky_headers = sticky;
        self.persist_settings();
    }

    /// Toggle sticky markdown headers setting and persist.
    #[allow(dead_code)]
    pub fn toggle_sticky_headers(&mut self) {
        self.sticky_headers = !self.sticky_headers;
        self.settings.sticky_headers = self.sticky_headers;
        self.persist_settings();
    }

    /// Set font size setting and persist.
    pub fn set_font_size(&mut self, size: u32) {
        self.settings.font_size = size;
        self.persist_settings();
    }

    /// Set source editor line wrap and persist.
    pub fn set_line_wrap(&mut self, line_wrap: bool) {
        self.settings.line_wrap = line_wrap;
        self.persist_settings();
    }

    /// Set language and persist.
    #[allow(dead_code)]
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
        self.settings.language = language;
        self.persist_settings();
    }

    /// Update the current reactive update status.
    pub fn set_update_status(&mut self, status: UpdateStatus) {
        self.update_status = status;
    }

    /// Toggle or set auto check for updates and persist.
    pub fn set_auto_check_updates(&mut self, enabled: bool) {
        self.settings.auto_check_updates = enabled;
        self.persist_settings();
    }

    /// Reset all settings to application defaults and persist.
    pub fn reset_settings_to_default(&mut self) {
        let defaults = crate::types::AppSettings::default();
        self.language = defaults.language;
        self.theme = defaults.theme;
        self.primary_color.clone_from(&defaults.primary_color);
        self.is_full_width = defaults.is_full_width;
        self.zoom_level = defaults.zoom_level;
        self.show_sidebar = defaults.show_sidebar;
        self.sidebar_tab = defaults.sidebar_tab;
        self.sidebar_position = defaults.sidebar_position;
        self.sidebar_width = defaults.sidebar_width;
        self.file_filter_mode = defaults.file_filter_mode;
        self.sticky_headers = defaults.sticky_headers;
        self.settings = defaults;
        self.is_loading_files = false;
        self.update_status = UpdateStatus::Idle;
        self.persist_settings();
        self.refresh_file_tree();
    }

    /// Set a custom keybinding for a shortcut action and persist.
    pub fn set_shortcut(&mut self, action: crate::types::ShortcutAction, binding: String) {
        self.settings.shortcuts.set_binding(action, binding);
        self.persist_settings();
    }

    /// Reset a single shortcut action to its default keybinding and persist.
    pub fn reset_shortcut(&mut self, action: crate::types::ShortcutAction) {
        self.settings.shortcuts.reset_action(action);
        self.persist_settings();
    }

    /// Reset all shortcut keybindings to defaults and persist.
    pub fn reset_all_shortcuts(&mut self) {
        self.settings.shortcuts.reset_all();
        self.persist_settings();
    }
}

