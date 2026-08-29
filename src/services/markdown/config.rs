use super::highlight::{get_syntax_set, get_theme_set, html_escape, slugify};
use crate::types::{DocumentFormat, ParsedDocument, TocItem};
use syntect::html::highlighted_html_for_string;

/// Validate configuration syntax for JSON, TOML, YAML.
pub fn validate_document(content: &str, format: DocumentFormat) -> Result<(), String> {
    if content.trim().is_empty() {
        return Ok(());
    }
    match format {
        DocumentFormat::Json => {
            serde_json::from_str::<serde_json::Value>(content)
                .map(|_| ())
                .map_err(|e| format!("{e}"))
        }
        DocumentFormat::Toml => {
            toml::from_str::<toml::Value>(content)
                .map(|_| ())
                .map_err(|e| format!("{e}"))
        }
        DocumentFormat::Yaml => {
            serde_yaml::from_str::<serde_yaml::Value>(content)
                .map(|_| ())
                .map_err(|e| format!("{e}"))
        }
        _ => Ok(()),
    }
}

/// Extract structural outline/TOC for config formats (TOML sections, JSON top keys, YAML top keys).
#[must_use]
pub fn extract_config_toc(raw: &str, format: DocumentFormat) -> Vec<TocItem> {
    let mut items = Vec::new();

    match format {
        DocumentFormat::Toml => {
            for line in raw.lines() {
                let trimmed = line.trim();
                if (trimmed.starts_with('[') && trimmed.ends_with(']')) && !trimmed.starts_with("[[") {
                    let section_name = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
                    let level = if section_name.contains('.') { 2 } else { 1 };
                    let id = slugify(section_name);
                    items.push(TocItem {
                        id,
                        title: format!("[{section_name}]"),
                        level,
                    });
                } else if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
                    let section_name = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
                    let id = slugify(section_name);
                    items.push(TocItem {
                        id,
                        title: format!("[[{section_name}]]"),
                        level: 2,
                    });
                }
            }
        }
        DocumentFormat::Json => {
            if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(raw) {
                for key in map.keys() {
                    let id = slugify(key);
                    items.push(TocItem {
                        id,
                        title: format!("\"{key}\""),
                        level: 1,
                    });
                }
            } else {
                for line in raw.lines() {
                    let trimmed = line.trim();
                    if let Some(stripped) = trimmed.strip_prefix('"') {
                        if let Some(quote_end) = stripped.find('"') {
                            let key = &stripped[..quote_end];
                            let rest = stripped[quote_end + 1..].trim_start();
                            if rest.starts_with(':') {
                                let id = slugify(key);
                                items.push(TocItem {
                                    id,
                                    title: format!("\"{key}\""),
                                    level: 1,
                                });
                            }
                        }
                    }
                }
            }
        }
        DocumentFormat::Yaml => {
            for line in raw.lines() {
                let trimmed = line.trim_end();
                if !trimmed.starts_with(' ') && !trimmed.starts_with('\t') && trimmed.contains(':') && !trimmed.starts_with('#') {
                    if let Some((k, _)) = trimmed.split_once(':') {
                        let clean_k = k.trim();
                        if !clean_k.is_empty() {
                            let id = slugify(clean_k);
                            items.push(TocItem {
                                id,
                                title: clean_k.to_string(),
                                level: 1,
                            });
                        }
                    }
                }
            }
        }
        _ => {}
    }

    items
}

/// Parse and render config documents (JSON, TOML, YAML, INI, etc.) into syntax-highlighted HTML views.
#[must_use]
pub fn parse_config_document(raw: &str, format: DocumentFormat) -> ParsedDocument {
    let syntax_set = get_syntax_set();
    let theme_set = get_theme_set();

    let syntax_token = format.syntax_token();
    let syntax = syntax_set.find_syntax_by_token(syntax_token)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let highlighted_code = theme_set.themes.get("base16-ocean.dark").map_or_else(
        || format!("<pre><code>{}</code></pre>", html_escape(raw)),
        |th| highlighted_html_for_string(raw, syntax_set, syntax, th)
            .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", html_escape(raw))),
    );

    let escaped_code = html_escape(raw);
    let lang_label = format.label();
    let token = format.syntax_token();

    let validation_error = validate_document(raw, format).err();
    let validation_badge_html = validation_error.as_ref().map_or_else(String::new, |err| {
        format!("<div class=\"config-syntax-error-banner\"><span class=\"error-icon\">⚠️</span> <span class=\"error-text\">Syntax Error: {}</span></div>", html_escape(err))
    });

    let toc = extract_config_toc(raw, format);

    let wrapped_html = format!(
        "<div class=\"config-doc-container format-{token}\">\
            {validation_badge_html}\
            <div class=\"code-block-container config-code-block\">\
                <div class=\"code-header\">\
                    <div class=\"flex items-center gap-2\">\
                        <span class=\"code-lang-label\">{lang_label}</span>\
                        <span class=\"config-status-tag\">{status_text}</span>\
                    </div>\
                    <button class=\"copy-code-button\" data-code=\"{escaped_code}\" onclick=\"copyCodeSnippet(this)\">\
                        <svg class=\"copy-icon\" viewBox=\"0 0 24 24\" width=\"14\" height=\"14\"><rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"/><path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"/></svg>\
                        <span>Copy</span>\
                    </button>\
                </div>\
                <div class=\"code-content\">{highlighted_code}</div>\
            </div>\
        </div>",
        status_text = if validation_error.is_some() { "⚠️ Invalid Syntax" } else { "✓ Valid Config" }
    );

    let words = raw.split_whitespace().count();
    let reading_time = words.div_ceil(200).max(1);

    ParsedDocument {
        html_content: wrapped_html,
        toc,
        metadata: None,
        word_count: words,
        reading_time_minutes: reading_time,
        format,
        validation_error,
    }
}
