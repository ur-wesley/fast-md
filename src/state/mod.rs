mod files;
mod layout;
mod tabs;
mod workspace;

use crate::services::fs::scan_file_tree;
use crate::services::fts;
use crate::services::markdown::parse_markdown_document;
use crate::services::settings::{load_settings, save_settings};
use crate::services::workspace::load_workspaces;
use crate::types::{
    AppSettings, AppTheme, DocumentMode, FileFilterMode, FileTreeEntry, Language, SidebarPosition,
    SidebarTab, TabItem, UpdateStatus, WorkspacesFile,
};
use dioxus::prelude::{spawn, Signal, WritableExt};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const WELCOME_DOC: &str = include_str!("../assets/welcome.md");

/// Central Application State Store managing all tabs, preferences, layout, and document data.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppStore {
    pub tabs: Vec<TabItem>,
    pub active_tab_id: usize,
    pub next_tab_id: usize,
    pub mode: DocumentMode,
    pub language: Language,
    pub theme: AppTheme,
    pub primary_color: Option<String>,
    pub is_zen: bool,
    pub is_full_width: bool,
    pub zoom_level: u32,
    pub show_sidebar: bool,
    pub sidebar_tab: SidebarTab,
    pub sidebar_position: SidebarPosition,
    pub sidebar_width: u32,
    pub file_filter_mode: FileFilterMode,
    pub sticky_headers: bool,
    pub show_search: bool,
    pub show_find_in_files: bool,
    pub show_settings_modal: bool,
    pub file_tree: Vec<FileTreeEntry>,
    pub is_loading_files: bool,
    pub opened_folder: Option<PathBuf>,
    pub pending_tree_scan: Option<PathBuf>,
    pub expanded_dirs: HashSet<PathBuf>,
    pub workspaces: WorkspacesFile,
    pub settings: AppSettings,
    pub update_status: UpdateStatus,
}

impl Default for AppStore {
    fn default() -> Self {
        let welcome_parsed = parse_markdown_document(WELCOME_DOC);
        let settings = load_settings();
        let workspaces = load_workspaces();
        Self {
            tabs: vec![TabItem {
                id: 1,
                path: None,
                title: "Welcome.md".to_string(),
                content: WELCOME_DOC.to_string(),
                parsed: welcome_parsed,
                is_dirty: false,
                html_revision: 0,
            }],
            active_tab_id: 1,
            next_tab_id: 2,
            mode: settings.default_mode,
            language: settings.language,
            theme: settings.theme,
            primary_color: settings.primary_color.clone(),
            is_zen: false,
            is_full_width: settings.is_full_width,
            zoom_level: settings.zoom_level,
            show_sidebar: settings.show_sidebar,
            sidebar_tab: settings.sidebar_tab,
            sidebar_position: settings.sidebar_position,
            sidebar_width: settings.sidebar_width,
            file_filter_mode: settings.file_filter_mode,
            sticky_headers: settings.sticky_headers,
            show_search: false,
            show_find_in_files: false,
            show_settings_modal: false,
            file_tree: Vec::new(),
            is_loading_files: false,
            opened_folder: None,
            pending_tree_scan: None,
            expanded_dirs: HashSet::new(),
            workspaces,
            settings,
            update_status: UpdateStatus::Idle,
        }
    }
}

impl AppStore {
    /// Initialize state with optional startup path, CLI theme/language override, and zen mode flags.
    #[must_use]
    pub fn new_with_options(
        initial_path: Option<&Path>,
        cli_theme: Option<AppTheme>,
        cli_lang: Option<Language>,
        zen: bool,
    ) -> Self {
        let settings = load_settings();
        let effective_theme = cli_theme.unwrap_or(settings.theme);
        let effective_language = cli_lang.unwrap_or(settings.language);

        let mut store = Self {
            mode: settings.default_mode,
            language: effective_language,
            theme: effective_theme,
            primary_color: settings.primary_color.clone(),
            is_zen: zen,
            is_full_width: settings.is_full_width,
            zoom_level: settings.zoom_level,
            show_sidebar: settings.show_sidebar,
            sidebar_tab: settings.sidebar_tab,
            sidebar_position: settings.sidebar_position,
            sidebar_width: settings.sidebar_width,
            file_filter_mode: settings.file_filter_mode,
            sticky_headers: settings.sticky_headers,
            settings,
            ..Default::default()
        };

        if let Some(path) = initial_path {
            store.boot_from_cli_path(path);
        } else if !store.boot_restore_last_workspace() {
            // keep default Welcome tab
        }

        store
    }

    /// Persist current settings to the settings.json file safely in a background thread.
    pub fn persist_settings(&self) {
        let settings = self.settings.clone();
        std::thread::spawn(move || {
            let _ = save_settings(&settings);
        });
    }

    /// Retrieve the currently active tab if one exists.
    #[must_use]
    pub fn active_tab(&self) -> Option<&TabItem> {
        self.tabs.iter().find(|t| t.id == self.active_tab_id)
    }
}

/// Dispatch an application shortcut action globally.
pub fn execute_shortcut_action(mut store: Signal<AppStore>, action: crate::types::ShortcutAction) {
    use crate::services::fs::{pick_file_async, pick_folder_async, pick_save_file_async};
    use crate::types::ShortcutAction;

    match action {
        ShortcutAction::Save => {
            spawn(async move {
                let s = store();
                if let Some(active) = s.active_tab() {
                    if active.path.is_some() {
                        let _ = store.write().save_active_tab();
                    } else {
                        let title = active.title.clone();
                        if let Some(path) = pick_save_file_async(&title).await {
                            let id = active.id;
                            let _ = store.write().save_tab_with_path(id, path);
                        }
                    }
                }
            });
        }
        ShortcutAction::SaveAs => {
            spawn(async move {
                let s = store();
                if let Some(active) = s.active_tab() {
                    let title = active.title.clone();
                    if let Some(path) = pick_save_file_async(&title).await {
                        let id = active.id;
                        let _ = store.write().save_tab_with_path(id, path);
                    }
                }
            });
        }
        ShortcutAction::OpenFile => {
            spawn(async move {
                if let Some(path) = pick_file_async().await {
                    store.write().open_file_from_path(path);
                    kick_pending_tree_scan(store);
                }
            });
        }
        ShortcutAction::OpenFolder => {
            spawn(async move {
                if let Some(dir) = pick_folder_async().await {
                    store.write().switch_workspace(dir);
                }
            });
        }
        ShortcutAction::NewTab => {
            store.write().new_empty_tab();
        }
        ShortcutAction::CloseTab => {
            let id = store().active_tab_id;
            store.write().close_tab(id);
        }
        ShortcutAction::ToggleSidebar => {
            store.write().toggle_sidebar();
        }
        ShortcutAction::ToggleZen => {
            store.write().toggle_zen();
        }
        ShortcutAction::CycleMode => {
            store.write().cycle_mode();
        }
        ShortcutAction::Find => {
            let _ = dioxus::prelude::document::eval(
                r"
                const input = document.getElementById('titlebar-search-input');
                if (input) { input.focus(); input.select(); }
                ",
            );
        }
        ShortcutAction::FindInFiles => {
            store.write().set_find_in_files(true);
        }
        ShortcutAction::FormatDocument => {
            store.write().format_active_tab();
        }
        ShortcutAction::ZoomIn => {
            store.write().zoom_in();
        }
        ShortcutAction::ZoomOut => {
            store.write().zoom_out();
        }
        ShortcutAction::ResetZoom => {
            store.write().reset_zoom();
        }
        ShortcutAction::ToggleSettings => {
            store.write().toggle_settings_modal();
        }
    }
}

/// Handle global Escape key behavior (close modals, exit zen, close search).
pub fn handle_escape_action(mut store: Signal<AppStore>) {
    let mut s = store.write();
    if s.show_settings_modal {
        s.set_settings_modal(false);
    } else if s.show_find_in_files {
        s.set_find_in_files(false);
    } else if s.is_zen {
        s.set_zen(false);
    } else if s.show_search {
        s.show_search = false;
    }
}

/// Complete an async directory scan and update the store.
pub async fn complete_directory_scan(mut store: Signal<AppStore>, dir: PathBuf) {
    let filter_mode = store().file_filter_mode;
    let scan_dir = dir.clone();
    let tree_res = tokio::task::spawn_blocking(move || scan_file_tree(&scan_dir, filter_mode)).await;

    if let Ok(Ok(tree)) = tree_res {
        store.write().finish_loading_directory(&dir, tree);
        let filter = store().file_filter_mode;
        kick_fts_rebuild_forced(dir, filter);
    } else {
        store.write().set_loading_files(false);
    }
}

/// Drop FTS index and rebuild for a new workspace root.
pub fn kick_fts_rebuild_forced(root: PathBuf, filter: FileFilterMode) {
    crate::services::fts::drop_index();
    std::thread::spawn(move || {
        let _ = crate::services::fts::rebuild_root(root, filter);
    });
}

/// Spawn a background rebuild of the session FTS index for the opened folder.
pub fn kick_fts_rebuild(store: Signal<AppStore>) {
    let root = store().opened_folder.clone();
    let filter = store().file_filter_mode;
    if let Some(dir) = root {
        if fts::is_index_for(&dir) {
            return;
        }
        kick_fts_rebuild_forced(dir, filter);
    } else {
        crate::services::fts::drop_index();
    }
}

/// Spawn a background scan if `open_file_from_path` queued one.
pub fn kick_pending_tree_scan(mut store: Signal<AppStore>) {
    let pending = store.write().take_pending_tree_scan();
    if let Some(dir) = pending {
        spawn(async move {
            complete_directory_scan(store, dir).await;
        });
    }
}

/// Locate the primary documentation file in a directory (e.g. README.md, index.md, or first markdown file).
#[must_use]
pub fn find_primary_doc_in_dir(dir: &Path) -> Option<PathBuf> {
    let candidates = [
        "README.md",
        "readme.md",
        "Readme.md",
        "README.markdown",
        "readme.markdown",
        "README.mdx",
        "readme.mdx",
        "index.md",
        "Index.md",
        "INDEX.md",
        "index.markdown",
        "index.mdx",
    ];
    for name in &candidates {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut md_files: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                if p.is_file() {
                    p.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
                        matches!(ext.to_ascii_lowercase().as_str(), "md" | "markdown" | "mdx" | "mdown")
                    })
                } else {
                    false
                }
            })
            .collect();
        md_files.sort();
        if let Some(first) = md_files.into_iter().next() {
            return Some(first);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::fs::scan_file_tree;
    use crate::types::DocumentFormat;
    use std::sync::Arc;

    #[test]
    fn test_primary_color_selection() {
        let mut store = AppStore::default();
        store.set_primary_color(None);
        // Select Catppuccin Mocha
        store.set_theme(AppTheme::CatppuccinMocha);
        assert_eq!(store.effective_primary_color(), "#cba6f7"); // Mocha default accent

        // Select custom primary color
        store.set_primary_color(Some("#ff007f".to_string()));
        assert_eq!(store.effective_primary_color(), "#ff007f");

        // Reset to theme default
        store.set_primary_color(None);
        assert_eq!(store.effective_primary_color(), "#cba6f7");
    }

    #[test]
    fn test_store_settings_sync() {
        let mut store = AppStore::default();
        store.reset_settings_to_default();
        assert_eq!(store.language, Language::En);

        store.set_language(Language::De);
        assert_eq!(store.language, Language::De);
        assert_eq!(store.settings.language, Language::De);

        store.set_theme(AppTheme::CatppuccinFrappe);
        assert_eq!(store.settings.theme, AppTheme::CatppuccinFrappe);

        store.toggle_full_width();
        assert!(store.settings.is_full_width);

        store.set_auto_reload(false);
        assert!(!store.settings.auto_reload);

        assert!(!store.sticky_headers);
        store.toggle_sticky_headers();
        assert!(store.sticky_headers);
        assert!(store.settings.sticky_headers);

        assert!(!store.settings.line_wrap);
        store.set_line_wrap(true);
        assert!(store.settings.line_wrap);

        store.reset_settings_to_default();
        assert_eq!(store.settings.language, Language::En);
        assert_eq!(store.language, Language::En);
        assert_eq!(store.settings.theme, AppTheme::Dark);
        assert_eq!(store.theme, AppTheme::Dark);
        assert!(!store.is_full_width);
        assert!(store.settings.auto_reload);
        assert!(!store.sticky_headers);
    }

    #[test]
    fn test_tab_editing_and_dirty_state() {
        let mut store = AppStore::default();
        assert_eq!(store.mode, DocumentMode::View);
        assert!(!store.tabs[0].is_dirty);

        store.cycle_mode();
        assert_eq!(store.mode, DocumentMode::Split);
        store.set_mode(DocumentMode::Wysiwyg);
        assert_eq!(store.mode, DocumentMode::Wysiwyg);

        store.update_active_tab_content("# Modified Title\n\nNew edited body text.".to_string());
        assert!(store.tabs[0].is_dirty);
        assert_eq!(store.tabs[0].content, "# Modified Title\n\nNew edited body text.");

        store.set_mode(DocumentMode::Source);
        assert_eq!(store.tabs[0].parsed.toc.len(), 1);
        assert_eq!(store.tabs[0].parsed.toc[0].title, "Modified Title");
    }

    #[test]
    fn test_formatting_and_format_on_save() {
        let mut store = AppStore::default();
        let unformatted = "# Unformatted\n| A | B |\n|---|---|\n| 1 | 2 |\n";
        store.update_active_tab_content(unformatted.to_string());

        assert!(store.settings.format_on_save);
        store.format_active_tab();

        let formatted = &store.tabs[0].content;
        assert!(formatted.contains("| A   | B   |"));
        assert!(formatted.contains("| --- | --- |"));
        assert!(formatted.contains("| 1   | 2   |"));

        store.set_format_on_save(false);
        assert!(!store.settings.format_on_save);
        store.toggle_format_on_save();
        assert!(store.settings.format_on_save);
    }

    #[test]
    fn test_config_tab_editing_and_formatting() {
        let mut store = AppStore::default();
        let json_path = PathBuf::from("test_config.json");
        store.tabs[0].path = Some(json_path);
        store.tabs[0].title = "test_config.json".to_string();

        let unformatted_json = r#"{"app":"fast-md","enabled":true}"#;
        store.update_active_tab_content(unformatted_json.to_string());
        assert_eq!(store.tabs[0].parsed.format, DocumentFormat::Json);
        assert!(store.tabs[0].is_dirty);
        assert!(store.tabs[0].parsed.validation_error.is_none());

        store.format_active_tab();
        let formatted = &store.tabs[0].content;
        assert!(formatted.contains("  \"app\": \"fast-md\""));
        assert!(formatted.contains("  \"enabled\": true"));
    }

    #[test]
    fn test_sidebar_position_setting() {
        let mut store = AppStore::default();
        store.set_sidebar_position(SidebarPosition::Left);
        assert_eq!(store.sidebar_position, SidebarPosition::Left);
        assert_eq!(store.settings.sidebar_position, SidebarPosition::Left);

        store.set_sidebar_position(SidebarPosition::Right);
        assert_eq!(store.sidebar_position, SidebarPosition::Right);
        assert_eq!(store.settings.sidebar_position, SidebarPosition::Right);

        store.set_sidebar_position(SidebarPosition::Left);
        assert_eq!(store.sidebar_position, SidebarPosition::Left);
        assert_eq!(store.settings.sidebar_position, SidebarPosition::Left);
    }

    #[test]
    fn test_file_filter_mode_cycling() {
        let mut store = AppStore::default();
        store.set_file_filter_mode(FileFilterMode::MarkdownAndConfig);
        assert_eq!(store.file_filter_mode, FileFilterMode::MarkdownAndConfig);

        store.cycle_file_filter_mode();
        assert_eq!(store.file_filter_mode, FileFilterMode::MarkdownOnly);

        store.cycle_file_filter_mode();
        assert_eq!(store.file_filter_mode, FileFilterMode::AllSupported);

        store.cycle_file_filter_mode();
        assert_eq!(store.file_filter_mode, FileFilterMode::AllFiles);

        store.cycle_file_filter_mode();
        assert_eq!(store.file_filter_mode, FileFilterMode::MarkdownAndConfig);
    }

    #[test]
    fn test_new_with_options_file() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let readme_path = manifest_dir.join("README.md");
        let store = AppStore::new_with_options(Some(&readme_path), None, None, false);

        assert_eq!(store.tabs.len(), 1);
        if let Some(path) = &store.tabs[0].path {
            assert_eq!(path, &readme_path);
        }
        assert_eq!(store.tabs[0].title, "README.md");
    }

    #[test]
    fn test_new_with_options_directory() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let store = AppStore::new_with_options(Some(&manifest_dir), None, None, false);

        // Should open the directory in file tree and find README.md
        assert_eq!(store.opened_folder.as_ref(), Some(&manifest_dir));
        assert!(!store.file_tree.is_empty());
        assert_eq!(store.tabs.len(), 1);
        assert_eq!(store.tabs[0].title, "README.md");
    }

    #[test]
    fn test_find_primary_doc_in_dir() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let primary = find_primary_doc_in_dir(&manifest_dir);
        assert!(primary.is_some());
        if let Some(p) = primary {
            if let Some(name) = p.file_name() {
                assert!(name.to_string_lossy().to_lowercase().starts_with("readme"));
            }
        }
    }

    #[test]
    fn test_set_sidebar_width_clamped() {
        let mut store = AppStore::default();
        store.set_sidebar_width(999);
        assert_eq!(store.sidebar_width, 560);
        assert_eq!(store.settings.sidebar_width, 560);
        store.set_sidebar_width(50);
        assert_eq!(store.sidebar_width, 180);
    }

    #[test]
    fn test_open_file_queues_async_tree_scan_when_tree_empty() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let readme = manifest_dir.join("README.md");
        let mut store = AppStore::default();
        assert!(store.file_tree.is_empty());

        let scan_parent = store.open_file_from_path(readme.clone());
        assert_eq!(scan_parent.as_ref(), Some(&manifest_dir));
        assert!(store.is_loading_files);
        assert_eq!(store.pending_tree_scan, Some(manifest_dir.clone()));
        assert!(store.tabs.iter().any(|t| t.path.as_ref() == Some(&readme)));

        let tree = scan_file_tree(&manifest_dir, store.file_filter_mode).unwrap();
        store.finish_loading_directory(&manifest_dir, tree);
        assert!(!store.is_loading_files);
        assert!(!store.file_tree.is_empty());
    }

    #[test]
    fn test_start_and_finish_loading_directory() {
        let mut store = AppStore::default();
        assert!(!store.is_loading_files);
        assert!(store.opened_folder.is_none());

        let target_dir = PathBuf::from("/test/docs");
        store.start_loading_directory(target_dir.clone());

        assert!(store.is_loading_files);
        assert_eq!(store.opened_folder.as_ref(), Some(&target_dir));
        assert_eq!(store.sidebar_tab, SidebarTab::Files);
        assert!(store.show_sidebar);

        let entries = vec![FileTreeEntry {
            name: "index.md".to_string(),
            path: target_dir.join("index.md"),
            is_dir: false,
            children: Arc::new(Vec::new()),
        }];

        // Finish with matching directory
        store.finish_loading_directory(&target_dir, entries.clone());
        assert!(!store.is_loading_files);
        assert_eq!(store.file_tree.len(), 1);
        assert_eq!(store.file_tree[0].name, "index.md");

        // Finish with stale/mismatched directory should be ignored
        store.start_loading_directory(target_dir.clone());
        let stale_dir = PathBuf::from("/different/dir");
        store.finish_loading_directory(&stale_dir, Vec::new());
        assert!(store.is_loading_files); // Still loading target_dir

        // Explicit set loading
        store.set_loading_files(false);
        assert!(!store.is_loading_files);
    }
}
