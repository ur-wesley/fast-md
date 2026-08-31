use crate::types::Language;

pub mod de;
pub mod en;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommonTranslations {
    pub reset: &'static str,
    pub custom: &'static str,
    pub default: &'static str,
    pub done: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleBarTranslations {
    pub search_placeholder: &'static str,
    pub prev_match: &'static str,
    pub next_match: &'static str,
    pub clear_search: &'static str,
    pub minimize: &'static str,
    pub maximize: &'static str,
    pub restore: &'static str,
    pub close: &'static str,
    pub workspace: &'static str,
    pub workspace_search_placeholder: &'static str,
    pub workspace_this_window: &'static str,
    pub workspace_recent_projects: &'static str,
    pub workspace_open_folder: &'static str,
    pub shortcut_ctrl_f: &'static str,
    pub update_available_badge: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolbarTranslations {
    pub toggle_sidebar: &'static str,
    pub sidebar: &'static str,
    pub open_file: &'static str,
    pub open: &'static str,
    pub open_folder: &'static str,
    pub folder: &'static str,
    pub new_tab: &'static str,
    pub new: &'static str,
    pub save_file: &'static str,
    pub save: &'static str,
    pub saved: &'static str,
    pub unsaved: &'static str,
    pub mode_view: &'static str,
    pub mode_split: &'static str,
    pub mode_wysiwyg: &'static str,
    pub mode_source: &'static str,
    pub column_layout: &'static str,
    pub column: &'static str,
    pub full_width_layout: &'static str,
    pub full_width: &'static str,
    pub zoom_out: &'static str,
    pub reset_zoom: &'static str,
    pub zoom_in: &'static str,
    pub sticky_headers_enabled: &'static str,
    pub sticky_headers_disabled: &'static str,
    pub sticky: &'static str,
    pub theme_customizer: &'static str,
    pub catppuccin_flavors: &'static str,
    pub classic_themes: &'static str,
    pub primary_accent_color: &'static str,
    pub pick_custom_color: &'static str,
    pub reset_accent_color: &'static str,
    pub export_html: &'static str,
    pub export: &'static str,
    pub focus_zen_mode: &'static str,
    pub zen: &'static str,
    pub preferences: &'static str,
    pub settings: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorTranslations {
    pub bold: &'static str,
    pub italic: &'static str,
    pub strikethrough: &'static str,
    pub h1: &'static str,
    pub h2: &'static str,
    pub h3: &'static str,
    pub blockquote: &'static str,
    pub inline_code: &'static str,
    pub code_block: &'static str,
    pub bullet_list: &'static str,
    pub numbered_list: &'static str,
    pub task_list: &'static str,
    pub link: &'static str,
    pub image: &'static str,
    pub table: &'static str,
    pub callout: &'static str,
    pub format_document: &'static str,
    pub format_config: &'static str,
    pub insert_json_object: &'static str,
    pub insert_json_array: &'static str,
    pub insert_json_kv: &'static str,
    pub insert_toml_section: &'static str,
    pub insert_toml_array_table: &'static str,
    pub insert_yaml_kv: &'static str,
    pub insert_yaml_list: &'static str,
    pub valid_syntax: &'static str,
    pub invalid_syntax: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
    pub wysiwyg_placeholder: &'static str,
    pub source_placeholder: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SidebarTranslations {
    pub outline: &'static str,
    pub files: &'static str,
    pub no_headings: &'static str,
    pub filter_files: &'static str,
    pub filter_tooltip: &'static str,
    pub filter_md_only: &'static str,
    pub filter_md_config: &'static str,
    pub filter_all_supported: &'static str,
    pub filter_all_files: &'static str,
    pub no_folder_opened: &'static str,
    pub open_folder_hint: &'static str,
    pub loading_folder: &'static str,
    pub scanning_files: &'static str,
    pub open_folder_button: &'static str,
    pub reading_progress: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusBarTranslations {
    pub memory_doc: &'static str,
    pub mdx_doc: &'static str,
    pub markdown_doc: &'static str,
    pub text_doc: &'static str,
    pub generic_doc: &'static str,
    pub words_suffix: &'static str,
    pub chars_suffix: &'static str,
    pub lines_suffix: &'static str,
    pub min_suffix: &'static str,
    pub sec_suffix: &'static str,
    pub status_saved: &'static str,
    pub status_unsaved: &'static str,
    pub cycle_mode: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabBarTranslations {
    pub new_file_or_tab: &'static str,
    pub close_tab: &'static str,
    pub unsaved_changes: &'static str,
    pub no_open_tabs: &'static str,
    pub no_open_tabs_desc: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontmatterTranslations {
    pub author: &'static str,
    pub date: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZenTranslations {
    pub exit_tooltip: &'static str,
    pub exit_button: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBarTranslations {
    pub placeholder: &'static str,
    pub prev_match: &'static str,
    pub next_match: &'static str,
    pub close_search: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindInFilesTranslations {
    pub title: &'static str,
    pub placeholder: &'static str,
    pub indexing: &'static str,
    pub searching: &'static str,
    pub type_to_search: &'static str,
    pub no_results: &'static str,
    pub no_results_desc: &'static str,
    pub close: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsTranslations {
    pub modal_title: &'static str,
    pub auto_save_notice: &'static str,
    pub close_tooltip: &'static str,

    // Search
    pub search_placeholder: &'static str,
    pub search_results_title: &'static str,
    pub no_results_title: &'static str,
    pub no_results_desc: &'static str,
    pub clear_search: &'static str,

    // Navigation Tabs
    pub tab_appearance: &'static str,
    pub tab_reader: &'static str,
    pub tab_workspace: &'static str,
    pub tab_shortcuts: &'static str,
    pub tab_config_file: &'static str,
    pub tab_updates: &'static str,

    // Updates section
    pub updates_title: &'static str,
    pub updates_desc: &'static str,
    pub current_version_label: &'static str,
    pub check_for_updates_button: &'static str,
    pub checking_for_updates: &'static str,
    pub up_to_date_message: &'static str,
    pub update_available_title: &'static str,
    pub update_download_button: &'static str,
    pub downloading_update: &'static str,
    pub installing_update: &'static str,
    pub update_ready_title: &'static str,
    pub restart_and_update_button: &'static str,
    pub update_error_title: &'static str,
    pub auto_check_updates_title: &'static str,
    pub auto_check_updates_desc: &'static str,
    pub view_release_notes: &'static str,

    // Appearance tab
    pub theme_presets_title: &'static str,
    pub theme_presets_desc: &'static str,
    pub catppuccin_flavors: &'static str,
    pub classic_themes: &'static str,
    pub primary_accent_title: &'static str,
    pub primary_accent_desc: &'static str,
    pub reset_theme_default: &'static str,
    pub pick_custom_color_title: &'static str,

    // Reader & Layout tab
    pub default_mode_title: &'static str,
    pub default_mode_desc: &'static str,
    pub format_on_save_title: &'static str,
    pub format_on_save_desc: &'static str,
    pub reading_layout_title: &'static str,
    pub reading_layout_desc: &'static str,
    pub reading_column: &'static str,
    pub full_width: &'static str,
    pub zoom_title: &'static str,
    pub zoom_desc: &'static str,
    pub font_size_title: &'static str,
    pub font_size_desc: &'static str,
    pub line_wrap_title: &'static str,
    pub line_wrap_desc: &'static str,
    pub auto_reload_title: &'static str,
    pub auto_reload_desc: &'static str,
    pub sticky_headers_title: &'static str,
    pub sticky_headers_desc: &'static str,

    // Workspace & Sidebar tab
    pub sidebar_tab_title: &'static str,
    pub sidebar_tab_desc: &'static str,
    pub sidebar_tab_toc: &'static str,
    pub sidebar_tab_files: &'static str,
    pub sidebar_position_title: &'static str,
    pub sidebar_position_desc: &'static str,
    pub sidebar_position_left: &'static str,
    pub sidebar_position_right: &'static str,
    pub sidebar_visibility_title: &'static str,
    pub sidebar_visibility_desc: &'static str,
    pub file_filter_mode_title: &'static str,
    pub file_filter_mode_desc: &'static str,
    pub recent_history_title: &'static str,
    pub recent_history_desc: &'static str,
    pub files_label: &'static str,
    pub folders_label: &'static str,
    pub explorer_integration_title: &'static str,
    pub explorer_integration_desc: &'static str,
    pub register_explorer: &'static str,
    pub registered_explorer: &'static str,
    pub open_default_apps: &'static str,

    // Language section
    pub language_section_title: &'static str,
    pub language_section_desc: &'static str,
    pub language_en: &'static str,
    pub language_de: &'static str,

    // Config file tab
    pub config_location_title: &'static str,
    pub config_location_desc: &'static str,
    pub copy_path: &'static str,
    pub copied_path: &'static str,
    pub open_in_editor: &'static str,
    pub show_in_folder: &'static str,
    pub reset_all_title: &'static str,
    pub reset_all_desc: &'static str,
    pub reset_defaults: &'static str,

    // Footer
    pub done_button: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextMenuTranslations {
    pub cut: &'static str,
    pub copy: &'static str,
    pub paste: &'static str,
    pub select_all: &'static str,
    pub open: &'static str,
    pub refresh: &'static str,
    pub close_others: &'static str,
    pub copy_selection: &'static str,
    pub open_link: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortcutsTranslations {
    pub tab_title: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub reset_all: &'static str,
    pub reset_shortcut: &'static str,
    pub press_keys: &'static str,
    pub click_to_record: &'static str,
    pub recording: &'static str,
    pub category_file: &'static str,
    pub category_layout: &'static str,
    pub category_editor: &'static str,
    pub category_view: &'static str,

    // Actions
    pub action_save: &'static str,
    pub action_save_desc: &'static str,
    pub action_save_as: &'static str,
    pub action_save_as_desc: &'static str,
    pub action_open_file: &'static str,
    pub action_open_file_desc: &'static str,
    pub action_open_folder: &'static str,
    pub action_open_folder_desc: &'static str,
    pub action_new_tab: &'static str,
    pub action_new_tab_desc: &'static str,
    pub action_close_tab: &'static str,
    pub action_close_tab_desc: &'static str,
    pub action_toggle_sidebar: &'static str,
    pub action_toggle_sidebar_desc: &'static str,
    pub action_toggle_zen: &'static str,
    pub action_toggle_zen_desc: &'static str,
    pub action_cycle_mode: &'static str,
    pub action_cycle_mode_desc: &'static str,
    pub action_find: &'static str,
    pub action_find_desc: &'static str,
    pub action_find_in_files: &'static str,
    pub action_find_in_files_desc: &'static str,
    pub action_format_document: &'static str,
    pub action_format_document_desc: &'static str,
    pub action_zoom_in: &'static str,
    pub action_zoom_in_desc: &'static str,
    pub action_zoom_out: &'static str,
    pub action_zoom_out_desc: &'static str,
    pub action_reset_zoom: &'static str,
    pub action_reset_zoom_desc: &'static str,
    pub action_toggle_settings: &'static str,
    pub action_toggle_settings_desc: &'static str,
}

impl ShortcutsTranslations {
    #[must_use]
    pub const fn action_name(self, action: crate::types::ShortcutAction) -> &'static str {
        match action {
            crate::types::ShortcutAction::Save => self.action_save,
            crate::types::ShortcutAction::SaveAs => self.action_save_as,
            crate::types::ShortcutAction::OpenFile => self.action_open_file,
            crate::types::ShortcutAction::OpenFolder => self.action_open_folder,
            crate::types::ShortcutAction::NewTab => self.action_new_tab,
            crate::types::ShortcutAction::CloseTab => self.action_close_tab,
            crate::types::ShortcutAction::ToggleSidebar => self.action_toggle_sidebar,
            crate::types::ShortcutAction::ToggleZen => self.action_toggle_zen,
            crate::types::ShortcutAction::CycleMode => self.action_cycle_mode,
            crate::types::ShortcutAction::Find => self.action_find,
            crate::types::ShortcutAction::FindInFiles => self.action_find_in_files,
            crate::types::ShortcutAction::FormatDocument => self.action_format_document,
            crate::types::ShortcutAction::ZoomIn => self.action_zoom_in,
            crate::types::ShortcutAction::ZoomOut => self.action_zoom_out,
            crate::types::ShortcutAction::ResetZoom => self.action_reset_zoom,
            crate::types::ShortcutAction::ToggleSettings => self.action_toggle_settings,
        }
    }

    #[must_use]
    pub const fn action_desc(self, action: crate::types::ShortcutAction) -> &'static str {
        match action {
            crate::types::ShortcutAction::Save => self.action_save_desc,
            crate::types::ShortcutAction::SaveAs => self.action_save_as_desc,
            crate::types::ShortcutAction::OpenFile => self.action_open_file_desc,
            crate::types::ShortcutAction::OpenFolder => self.action_open_folder_desc,
            crate::types::ShortcutAction::NewTab => self.action_new_tab_desc,
            crate::types::ShortcutAction::CloseTab => self.action_close_tab_desc,
            crate::types::ShortcutAction::ToggleSidebar => self.action_toggle_sidebar_desc,
            crate::types::ShortcutAction::ToggleZen => self.action_toggle_zen_desc,
            crate::types::ShortcutAction::CycleMode => self.action_cycle_mode_desc,
            crate::types::ShortcutAction::Find => self.action_find_desc,
            crate::types::ShortcutAction::FindInFiles => self.action_find_in_files_desc,
            crate::types::ShortcutAction::FormatDocument => self.action_format_document_desc,
            crate::types::ShortcutAction::ZoomIn => self.action_zoom_in_desc,
            crate::types::ShortcutAction::ZoomOut => self.action_zoom_out_desc,
            crate::types::ShortcutAction::ResetZoom => self.action_reset_zoom_desc,
            crate::types::ShortcutAction::ToggleSettings => self.action_toggle_settings_desc,
        }
    }

    #[must_use]
    pub const fn category_name(self, category: crate::types::ShortcutCategory) -> &'static str {
        match category {
            crate::types::ShortcutCategory::FileAndTabs => self.category_file,
            crate::types::ShortcutCategory::LayoutAndModes => self.category_layout,
            crate::types::ShortcutCategory::EditorAndSearch => self.category_editor,
            crate::types::ShortcutCategory::ViewAndPreferences => self.category_view,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Translations {
    pub common: CommonTranslations,
    pub title_bar: TitleBarTranslations,
    pub toolbar: ToolbarTranslations,
    pub editor: EditorTranslations,
    pub sidebar: SidebarTranslations,
    pub status_bar: StatusBarTranslations,
    pub tab_bar: TabBarTranslations,
    pub frontmatter: FrontmatterTranslations,
    pub zen: ZenTranslations,
    pub search_bar: SearchBarTranslations,
    pub find_in_files: FindInFilesTranslations,
    pub settings: SettingsTranslations,
    pub context_menu: ContextMenuTranslations,
    pub shortcuts: ShortcutsTranslations,
}

/// Retrieve the translation bundle for a given language.
#[must_use]
pub const fn t(lang: Language) -> &'static Translations {
    match lang {
        Language::En => &en::EN,
        Language::De => &de::DE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translations_parity() {
        let en = t(Language::En);
        let de = t(Language::De);

        // Verify common
        assert!(!en.common.done.is_empty());
        assert!(!de.common.done.is_empty());

        // Verify title bar
        assert!(!en.title_bar.search_placeholder.is_empty());
        assert!(!de.title_bar.search_placeholder.is_empty());

        // Verify toolbar
        assert!(!en.toolbar.sidebar.is_empty());
        assert!(!de.toolbar.sidebar.is_empty());

        // Verify sidebar
        assert!(!en.sidebar.outline.is_empty());
        assert!(!de.sidebar.outline.is_empty());
        assert!(!en.sidebar.loading_folder.is_empty());
        assert!(!de.sidebar.loading_folder.is_empty());
        assert!(!en.sidebar.scanning_files.is_empty());
        assert!(!de.sidebar.scanning_files.is_empty());
        assert!(!en.sidebar.open_folder_button.is_empty());
        assert!(!de.sidebar.open_folder_button.is_empty());
        assert!(!en.sidebar.reading_progress.is_empty());
        assert!(!de.sidebar.reading_progress.is_empty());

        // Verify tab bar
        assert!(!en.tab_bar.close_tab.is_empty());
        assert!(!de.tab_bar.close_tab.is_empty());
        assert!(!en.tab_bar.no_open_tabs.is_empty());
        assert!(!de.tab_bar.no_open_tabs.is_empty());
        assert!(!en.tab_bar.no_open_tabs_desc.is_empty());
        assert!(!de.tab_bar.no_open_tabs_desc.is_empty());

        // Verify status bar
        assert!(!en.status_bar.words_suffix.is_empty());
        assert!(!de.status_bar.words_suffix.is_empty());

        // Verify editor
        assert!(!en.editor.bold.is_empty());
        assert!(!de.editor.bold.is_empty());
        assert!(!en.editor.format_document.is_empty());
        assert!(!de.editor.format_document.is_empty());
        assert!(!en.editor.wysiwyg_placeholder.is_empty());
        assert!(!de.editor.wysiwyg_placeholder.is_empty());

        // Verify settings
        assert!(!en.settings.modal_title.is_empty());
        assert!(!de.settings.modal_title.is_empty());
        assert!(!en.settings.default_mode_title.is_empty());
        assert!(!de.settings.default_mode_title.is_empty());
        assert!(!en.settings.format_on_save_title.is_empty());
        assert!(!de.settings.format_on_save_title.is_empty());
        assert!(!en.settings.language_section_title.is_empty());
        assert!(!de.settings.language_section_title.is_empty());
        assert!(!en.settings.tab_updates.is_empty());
        assert!(!de.settings.tab_updates.is_empty());
        assert!(!en.settings.check_for_updates_button.is_empty());
        assert!(!de.settings.check_for_updates_button.is_empty());

        assert!(!en.context_menu.cut.is_empty());
        assert!(!de.context_menu.cut.is_empty());
        assert!(!en.context_menu.open_link.is_empty());
        assert!(!de.context_menu.open_link.is_empty());

        // Verify shortcuts
        assert!(!en.shortcuts.tab_title.is_empty());
        assert!(!de.shortcuts.tab_title.is_empty());
        assert!(!en.shortcuts.reset_all.is_empty());
        assert!(!de.shortcuts.reset_all.is_empty());
        for &action in crate::types::ShortcutAction::all() {
            assert!(!en.shortcuts.action_name(action).is_empty());
            assert!(!de.shortcuts.action_name(action).is_empty());
            assert!(!en.shortcuts.action_desc(action).is_empty());
            assert!(!de.shortcuts.action_desc(action).is_empty());
        }
    }
}
