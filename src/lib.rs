pub mod app;
pub mod cli;
pub mod components;
pub mod i18n;
pub mod services;
pub mod state;
pub mod types;
pub mod ui;

use cli::CliArgs;
use dioxus::desktop::{Config, WindowBuilder};
use std::env;
use std::path::PathBuf;
use std::sync::OnceLock;

pub(crate) static CLI_ARGS: OnceLock<CliArgs> = OnceLock::new();

pub const APP_STYLES: &str = concat!(
    include_str!("assets/css/themes.css"),
    include_str!("assets/css/dx-theme-bridge.css"),
    include_str!("assets/css/chrome.css"),
    include_str!("assets/css/sidebar.css"),
    include_str!("assets/css/viewer.css"),
    include_str!("assets/css/overlays.css"),
    include_str!("assets/css/settings.css"),
    include_str!("assets/css/editor.css"),
    include_str!("assets/dx-components-theme.css"),
    include_str!("assets/css/dx-tooltips.css"),
);

const HELPER_JS: &str = concat!(
    include_str!("assets/js/clipboard.js"),
    include_str!("assets/js/search.js"),
    include_str!("assets/js/editor.js"),
    include_str!("assets/js/history.js"),
    include_str!("assets/js/wysiwyg.js"),
    include_str!("assets/js/toolbar_state.js"),
    include_str!("assets/js/scroll.js"),
    include_str!("assets/js/contextmenu.js"),
    include_str!("assets/js/shortcuts.js"),
);

#[allow(clippy::single_option_map)]
pub(crate) fn resolve_cli_path(raw_path: Option<&PathBuf>) -> Option<PathBuf> {
    raw_path.map(|p| {
        if p.is_absolute() && p.exists() {
            p.clone()
        } else if let Ok(current_dir) = env::current_dir() {
            let combined = current_dir.join(p);
            if combined.exists() {
                combined
            } else if p.exists() {
                p.clone()
            } else {
                combined
            }
        } else {
            p.clone()
        }
    })
}

pub fn run() {
    let args = CliArgs::parse_safe();

    if args.register {
        if services::association::register_file_associations() {
            println!("Successfully registered Fast-MD in Windows Explorer and Default Apps.");
        } else {
            eprintln!("Failed to register file associations.");
        }
        return;
    }

    if args.unregister {
        if services::association::unregister_file_associations() {
            println!("Successfully unregistered Fast-MD file associations.");
        } else {
            eprintln!("Failed to unregister file associations.");
        }
        return;
    }

    std::thread::spawn(|| {
        let _ = services::association::register_file_associations();
    });

    let _ = CLI_ARGS.set(args);

    let config = Config::new()
        .with_disable_context_menu(true)
        .with_window(
            WindowBuilder::new()
                .with_title("fast-md")
                .with_decorations(false)
                .with_transparent(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(1180.0, 800.0))
                .with_min_inner_size(dioxus::desktop::LogicalSize::new(640.0, 420.0)),
        )
        .with_background_color((0, 0, 0, 0))
        .with_custom_head(format!(
            "<script src=\"https://cdn.tailwindcss.com\"></script><style>{APP_STYLES}</style><script>{HELPER_JS}</script>"
        ));

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app::App);
}
