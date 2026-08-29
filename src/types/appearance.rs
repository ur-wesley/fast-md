use serde::{Deserialize, Serialize};

/// Open document viewing / editing modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentMode {
    #[default]
    View,
    Split,
    Wysiwyg,
    Source,
}

impl DocumentMode {
    #[must_use]
    #[allow(dead_code)]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Split => "split",
            Self::Wysiwyg => "wysiwyg",
            Self::Source => "source",
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::View => "View",
            Self::Split => "Split Preview",
            Self::Wysiwyg => "WYSIWYG",
            Self::Source => "Source",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::View => Self::Split,
            Self::Split => Self::Wysiwyg,
            Self::Wysiwyg => Self::Source,
            Self::Source => Self::View,
        }
    }
}

/// Available visual themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppTheme {
    #[default]
    Dark,
    Midnight,
    Light,
    Nord,
    SolarizedDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
}

impl AppTheme {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "theme-dark",
            Self::Midnight => "theme-midnight",
            Self::Light => "theme-light",
            Self::Nord => "theme-nord",
            Self::SolarizedDark => "theme-solarized",
            Self::CatppuccinLatte => "theme-catppuccin-latte",
            Self::CatppuccinFrappe => "theme-catppuccin-frappe",
            Self::CatppuccinMacchiato => "theme-catppuccin-macchiato",
            Self::CatppuccinMocha => "theme-catppuccin-mocha",
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Dark => "GitHub Dark",
            Self::Midnight => "Obsidian Night",
            Self::Light => "GitHub Light",
            Self::Nord => "Nordic Frost",
            Self::SolarizedDark => "Solarized Dark",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::CatppuccinFrappe => "Catppuccin Frappé",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinMocha => "Catppuccin Mocha",
        }
    }

    #[must_use]
    pub const fn default_accent(self) -> &'static str {
        match self {
            Self::Dark => "#58a6ff",
            Self::Midnight => "#8b5cf6",
            Self::Light => "#0969da",
            Self::Nord => "#88c0d0",
            Self::SolarizedDark => "#268bd2",
            Self::CatppuccinLatte => "#8839ef",
            Self::CatppuccinFrappe => "#ca9ee6",
            Self::CatppuccinMacchiato => "#c6a0f6",
            Self::CatppuccinMocha => "#cba6f7",
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn default_bg(self) -> &'static str {
        match self {
            Self::Dark => "#161b22",
            Self::Midnight => "#12141c",
            Self::Light => "#f6f8fa",
            Self::Nord => "#3b4252",
            Self::SolarizedDark => "#073642",
            Self::CatppuccinLatte => "#eff1f5",
            Self::CatppuccinFrappe => "#303446",
            Self::CatppuccinMacchiato => "#24273a",
            Self::CatppuccinMocha => "#1e1e2e",
        }
    }

    #[must_use]
    pub const fn is_dark(self) -> bool {
        !matches!(self, Self::Light | Self::CatppuccinLatte)
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn acrylic_tint(self) -> (u8, u8, u8, u8) {
        match self {
            Self::Light => (255, 255, 255, 120),
            Self::CatppuccinLatte => (239, 241, 245, 120),
            Self::CatppuccinFrappe => (48, 52, 70, 130),
            Self::CatppuccinMacchiato => (36, 39, 58, 130),
            Self::CatppuccinMocha => (30, 30, 46, 130),
            Self::Midnight => (9, 10, 15, 130),
            Self::Nord => (46, 52, 64, 130),
            Self::SolarizedDark => (0, 43, 54, 130),
            Self::Dark => (13, 17, 23, 130),
        }
    }
}

/// User interface display language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    #[default]
    En,
    De,
}

impl Language {
    #[must_use]
    #[allow(dead_code)]
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::De => "de",
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::De => "Deutsch",
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn strings(self) -> &'static crate::i18n::Translations {
        crate::i18n::t(self)
    }
}
