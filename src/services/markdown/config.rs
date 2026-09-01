use super::highlight::{
    get_syntax_set, get_theme_set, highlighted_config_html_for_string,
    highlighted_config_lines_for_string, html_escape, html_escape_char, slugify,
};
use crate::types::{DocumentFormat, ParsedDocument, TocItem};
use syntect::util::LinesWithEndings;

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

fn json_key_source_line(raw: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\":");
    for (line_idx, line) in raw.lines().enumerate() {
        if line.trim().contains(&needle) {
            return Some(line_idx);
        }
    }
    None
}

/// Extract structural outline/TOC for config formats (TOML sections, JSON top keys, YAML top keys).
#[must_use]
pub fn extract_config_toc(raw: &str, format: DocumentFormat) -> Vec<TocItem> {
    let mut items = Vec::new();

    match format {
        DocumentFormat::Toml => {
            for (line_idx, line) in raw.lines().enumerate() {
                let trimmed = line.trim();
                if (trimmed.starts_with('[') && trimmed.ends_with(']')) && !trimmed.starts_with("[[") {
                    let section_name = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
                    let level = if section_name.contains('.') { 2 } else { 1 };
                    let id = slugify(section_name);
                    items.push(TocItem {
                        id,
                        title: format!("[{section_name}]"),
                        level,
                        line: Some(line_idx),
                    });
                } else if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
                    let section_name = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
                    let id = slugify(section_name);
                    items.push(TocItem {
                        id,
                        title: format!("[[{section_name}]]"),
                        level: 2,
                        line: Some(line_idx),
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
                        line: json_key_source_line(raw, key),
                    });
                }
            } else {
                for (line_idx, line) in raw.lines().enumerate() {
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
                                    line: Some(line_idx),
                                });
                            }
                        }
                    }
                }
            }
        }
        DocumentFormat::Yaml => {
            for (line_idx, line) in raw.lines().enumerate() {
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
                                line: Some(line_idx),
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

fn plain_text_preview_lines(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    for line in LinesWithEndings::from(raw) {
        let mut escaped = String::new();
        for c in line.chars() {
            html_escape_char(c, &mut escaped);
        }
        lines.push(escaped);
    }
    lines
}

/// Parse plain text documents as escaped raw text (no markdown or config chrome).
#[must_use]
pub fn parse_plain_text(raw: &str) -> ParsedDocument {
    let preview_lines = plain_text_preview_lines(raw);
    let words = raw.split_whitespace().count();
    let reading_time = words.div_ceil(200).max(1);

    ParsedDocument {
        html_content: format!("<pre class=\"plain-text-doc\">{}</pre>", html_escape(raw)),
        preview_lines,
        toc: Vec::new(),
        metadata: None,
        word_count: words,
        reading_time_minutes: reading_time,
        format: DocumentFormat::PlainText,
        validation_error: None,
    }
}

fn config_preview_chrome(
    format: DocumentFormat,
    validation_error: &Option<String>,
    tab_id_attr: &str,
) -> (String, String) {
    let lang_label = format.label();
    let token = format.syntax_token();
    let validation_badge_html = validation_error.as_ref().map_or_else(String::new, |err| {
        format!("<div class=\"config-syntax-error-banner\"><span class=\"error-icon\">⚠️</span> <span class=\"error-text\">Syntax Error: {}</span></div>", html_escape(err))
    });
    let status_text = if validation_error.is_some() {
        "⚠️ Invalid Syntax"
    } else {
        "✓ Valid Config"
    };

    let prefix = format!(
        "<div class=\"config-doc-container format-{token}\">\
            {validation_badge_html}\
            <div class=\"code-block-container config-code-block\">\
                <div class=\"code-header\">\
                    <div class=\"flex items-center gap-2\">\
                        <span class=\"code-lang-label\">{lang_label}</span>\
                        <span class=\"config-status-tag\">{status_text}</span>\
                    </div>\
                    <button class=\"copy-code-button\" data-tab-id=\"{tab_id_attr}\" onclick=\"copyCodeSnippet(this)\">\
                        <svg class=\"copy-icon\" viewBox=\"0 0 24 24\" width=\"14\" height=\"14\"><rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"/><path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"/></svg>\
                        <span>Copy</span>\
                    </button>\
                </div>\
                <div class=\"code-content\"><pre class=\"highlight\">"
    );
    let suffix = "</pre></div></div></div>".to_string();
    (prefix, suffix)
}

/// Parse and render config documents (JSON, TOML, YAML, INI, etc.) into syntax-highlighted HTML views.
#[must_use]
pub fn parse_config_document(raw: &str, format: DocumentFormat) -> ParsedDocument {
    let syntax_set = get_syntax_set();
    let theme_set = get_theme_set();

    let syntax_token = format.syntax_token();
    let syntax = syntax_set
        .find_syntax_by_token(syntax_token)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let theme = theme_set.themes.get("base16-ocean.dark");

    let preview_lines = theme
        .map(|th| {
            highlighted_config_lines_for_string(raw, syntax_set, syntax, th)
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let highlighted_code = theme.map_or_else(
        || format!("<pre><code>{}</code></pre>", html_escape(raw)),
        |th| highlighted_config_html_for_string(raw, syntax_set, syntax, th)
            .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", html_escape(raw))),
    );

    let validation_error = validate_document(raw, format).err();
    let toc = extract_config_toc(raw, format);

    let joined_preview = preview_lines.join("");
    let highlighted_body = if joined_preview.is_empty() {
        highlighted_code
    } else {
        format!("<pre class=\"highlight\">{joined_preview}</pre>")
    };

    let (chrome_prefix, chrome_suffix) = config_preview_chrome(format, &validation_error, "0");
    let wrapped_html = format!("{chrome_prefix}{highlighted_body}{chrome_suffix}");

    let words = raw.split_whitespace().count();
    let reading_time = words.div_ceil(200).max(1);

    ParsedDocument {
        html_content: wrapped_html,
        preview_lines,
        toc,
        metadata: None,
        word_count: words,
        reading_time_minutes: reading_time,
        format,
        validation_error,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_config_preview_lines_match_line_count() {
        let raw = "{\n  \"a\": 1,\n  \"b\": 2\n}\n";
        let doc = parse_config_document(raw, DocumentFormat::Json);
        assert_eq!(doc.preview_lines.len(), 4);
        assert!(doc.uses_line_preview());
    }

    #[test]
    fn toml_syntax_is_not_plain_text() {
        let ss = get_syntax_set();
        let syntax = ss.find_syntax_by_token("toml").expect("toml syntax loaded");
        assert_ne!(syntax.name, ss.find_syntax_plain_text().name);
        let doc = parse_config_document("name = \"fast-md\"\n", DocumentFormat::Toml);
        assert!(!doc.preview_lines.is_empty());
        let joined = doc.preview_lines.join("");
        assert!(
            joined.contains("<span"),
            "expected highlighted spans, got {joined}"
        );
    }
}
