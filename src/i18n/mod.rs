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
    pub no_folder_opened: &'static str,
    pub open_folder_hint: &'static str,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabBarTranslations {
    pub new_file_or_tab: &'static str,
    pub close_tab: &'static str,
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
pub struct SettingsTranslations {
    pub modal_title: &'static str,
    pub auto_save_notice: &'static str,
    pub close_tooltip: &'static str,

    // Navigation Tabs
    pub tab_appearance: &'static str,
    pub tab_reader: &'static str,
    pub tab_workspace: &'static str,
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
    pub auto_reload_title: &'static str,
    pub auto_reload_desc: &'static str,
    pub sticky_headers_title: &'static str,
    pub sticky_headers_desc: &'static str,

    // Workspace & Sidebar tab
    pub sidebar_tab_title: &'static str,
    pub sidebar_tab_desc: &'static str,
    pub sidebar_tab_toc: &'static str,
    pub sidebar_tab_files: &'static str,
    pub sidebar_visibility_title: &'static str,
    pub sidebar_visibility_desc: &'static str,
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
    pub settings: SettingsTranslations,
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
        assert!(!en.sidebar.reading_progress.is_empty());
        assert!(!de.sidebar.reading_progress.is_empty());

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
    }
}
