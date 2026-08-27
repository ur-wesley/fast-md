use clap::Parser;
use std::path::PathBuf;

/// Lightning-fast native desktop Markdown & MDX viewer built with Dioxus.
#[derive(Parser, Debug, Clone)]
#[command(name = "fast-md", author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to a markdown file or directory to open on startup.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Start in Zen reading mode (no sidebars/toolbars).
    #[arg(short = 'z', long = "zen")]
    pub zen: bool,

    /// Initial theme: dark, midnight, light, nord, solarized, mocha, macchiato, frappe, latte.
    #[arg(short = 't', long = "theme", value_name = "THEME")]
    pub theme: Option<String>,

    /// Register Fast-MD in Windows Explorer and Default Apps for .md files.
    #[arg(long = "register")]
    pub register: bool,

    /// Unregister Fast-MD file associations.
    #[arg(long = "unregister")]
    pub unregister: bool,
}

impl CliArgs {
    /// Parse command-line arguments safely from the environment.
    #[must_use]
    pub fn parse_safe() -> Self {
        Self::parse()
    }
}
