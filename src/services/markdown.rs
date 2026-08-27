use crate::types::{DocMetadata, ParsedDocument, TocItem};
use pulldown_cmark::{html, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::sync::OnceLock;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

/// Convert a heading text into a URL-friendly anchor slug.
fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut prev_dash = false;

    for c in text.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if (c == ' ' || c == '-' || c == '_') && !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }

    if slug.ends_with('-') {
        let _ = slug.pop();
    }

    if slug.is_empty() {
        "section".to_owned()
    } else {
        slug
    }
}

/// Extract YAML or TOML frontmatter from the beginning of markdown content.
#[must_use]
pub fn extract_frontmatter(raw: &str) -> (Option<DocMetadata>, &str) {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return (None, raw);
    }

    let Some(rest) = trimmed.strip_prefix("---") else {
        return (None, raw);
    };

    let Some(first_newline_idx) = rest.find('\n') else {
        return (None, raw);
    };

    let content_after_first_line = &rest[first_newline_idx + 1..];
    let Some(closing_idx) = content_after_first_line.find("\n---") else {
        return (None, raw);
    };

    let frontmatter_str = &content_after_first_line[..closing_idx];
    let after_closing = &content_after_first_line[closing_idx + 4..];
    let markdown_body = after_closing.strip_prefix("\r\n").or_else(|| after_closing.strip_prefix('\n')).unwrap_or(after_closing);

    let parsed_yaml: Result<serde_yaml::Value, _> = serde_yaml::from_str(frontmatter_str);
    let Ok(yaml_val) = parsed_yaml else {
        return (None, markdown_body);
    };

    let Some(mapping) = yaml_val.as_mapping() else {
        return (None, markdown_body);
    };

    let mut metadata = DocMetadata::default();
    let mut extra = BTreeMap::new();

    for (k, v) in mapping {
        let Some(key_str) = k.as_str() else { continue };
        match key_str {
            "title" => metadata.title = v.as_str().map(ToString::to_string),
            "description" | "summary" => metadata.description = v.as_str().map(ToString::to_string),
            "author" => metadata.author = v.as_str().map(ToString::to_string),
            "date" => {
                metadata.date = v.as_str().map(ToString::to_string).or_else(|| {
                    v.as_i64().map(|i| i.to_string())
                });
            }
            "tags" | "keywords" => {
                if let Some(seq) = v.as_sequence() {
                    metadata.tags = seq.iter().filter_map(|item| item.as_str().map(ToString::to_string)).collect();
                } else if let Some(tag_str) = v.as_str() {
                    metadata.tags = tag_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                }
            }
            other => {
                if let Some(s) = v.as_str() {
                    let _ = extra.insert(other.to_string(), s.to_string());
                } else if let Some(b) = v.as_bool() {
                    let _ = extra.insert(other.to_string(), b.to_string());
                }
            }
        }
    }

    metadata.extra = extra;
    (Some(metadata), markdown_body)
}

/// Preprocess MDX specific syntax (JSX tags, Callouts, Steps, Badges) into HTML compatible syntax.
#[must_use]
pub fn preprocess_mdx(input: &str) -> String {
    let mut output = String::with_capacity(input.len() + 256);
    let lines = input.lines();

    for line in lines {
        let trimmed = line.trim();

        // Convert common MDX component tags into styled HTML
        if trimmed.starts_with("<Callout") || trimmed.starts_with("<Note") || trimmed.starts_with("<Warning") || trimmed.starts_with("<Info") {
            let callout_type = if trimmed.contains("type=\"warning\"") || trimmed.starts_with("<Warning") {
                "warning"
            } else if trimmed.contains("type=\"error\"") || trimmed.contains("type=\"danger\"") {
                "danger"
            } else if trimmed.contains("type=\"tip\"") || trimmed.contains("type=\"success\"") {
                "tip"
            } else {
                "info"
            };
            let _ = writeln!(output, "<div class=\"mdx-callout mdx-callout-{callout_type}\">");
        } else if trimmed == "</Callout>" || trimmed == "</Note>" || trimmed == "</Warning>" || trimmed == "</Info>" {
            output.push_str("</div>\n");
        } else if trimmed.starts_with("<Card") {
            output.push_str("<div class=\"mdx-card\">\n");
        } else if trimmed == "</Card>" {
            output.push_str("</div>\n");
        } else if trimmed.starts_with("<Badge") {
            output.push_str("<span class=\"mdx-badge\">");
        } else if trimmed == "</Badge>" {
            output.push_str("</span>\n");
        } else if trimmed.starts_with("<Steps>") {
            output.push_str("<div class=\"mdx-steps\">\n");
        } else if trimmed == "</Steps>" {
            output.push_str("</div>\n");
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}

/// Parse full Markdown/MDX text into rendered HTML with table of contents and metadata.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn parse_markdown_document(raw: &str) -> ParsedDocument {
    let (metadata, body) = extract_frontmatter(raw);
    let preprocessed = preprocess_mdx(body);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_GFM);

    let parser = Parser::new_ext(&preprocessed, options);

    let mut html_output = String::with_capacity(preprocessed.len() * 2);
    let mut toc = Vec::new();
    let mut heading_text_buffer = String::new();
    let mut in_heading = false;
    let mut current_heading_level = 1u8;

    let mut code_block_content = String::new();
    let mut in_code_block = false;
    let mut code_block_lang = String::new();

    let mut events_to_render = Vec::new();
    let mut in_section = false;

    let syntax_set = get_syntax_set();
    let theme_set = get_theme_set();
    let theme = theme_set
        .themes
        .get("base16-ocean.dark")
        .or_else(|| theme_set.themes.get("InspiredGitHub"))
        .or_else(|| theme_set.themes.values().next());

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_heading_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                heading_text_buffer.clear();
                if in_section {
                    events_to_render.push(Event::Html("</section>\n".into()));
                }
                in_section = true;
                events_to_render.push(Event::Html(
                    format!("<section class=\"markdown-section markdown-section-h{current_heading_level}\">\n").into(),
                ));
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                let title = heading_text_buffer.trim().to_string();
                let id = slugify(&title);

                toc.push(TocItem {
                    id: id.clone(),
                    title: title.clone(),
                    level: current_heading_level,
                });

                let heading_tag = format!("h{current_heading_level}");
                events_to_render.push(Event::Html(
                    format!("<{heading_tag} id=\"{id}\" class=\"doc-heading\"><a href=\"#{id}\" class=\"heading-anchor\">#</a> {title}</{heading_tag}>\n").into(),
                ));
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = true;
                code_block_content.clear();
                code_block_lang = lang.as_ref().to_string();
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                in_code_block = true;
                code_block_content.clear();
                code_block_lang.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let lang_clean = code_block_lang.split_whitespace().next().unwrap_or("");
                let syntax = if lang_clean.is_empty() {
                    syntax_set.find_syntax_plain_text()
                } else {
                    syntax_set
                        .find_syntax_by_token(lang_clean)
                        .unwrap_or_else(|| syntax_set.find_syntax_plain_text())
                };

                let highlighted_html = theme.map_or_else(
                    || format!("<pre><code>{}</code></pre>", html_escape(&code_block_content)),
                    |th| highlighted_html_for_string(&code_block_content, syntax_set, syntax, th)
                        .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", html_escape(&code_block_content))),
                );

                let lang_badge = if lang_clean.is_empty() { "text" } else { lang_clean };
                let escaped_code_attr = html_escape(&code_block_content);

                let wrapped_code = format!(
                    "<div class=\"code-block-container\">\
                        <div class=\"code-header\">\
                            <span class=\"code-lang-label\">{lang_badge}</span>\
                            <button class=\"copy-code-button\" data-code=\"{escaped_code_attr}\" onclick=\"copyCodeSnippet(this)\">\
                                <svg class=\"copy-icon\" viewBox=\"0 0 24 24\" width=\"14\" height=\"14\"><rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"/><path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"/></svg>\
                                <span>Copy</span>\
                            </button>\
                        </div>\
                        <div class=\"code-content\">{highlighted_html}</div>\
                    </div>"
                );

                events_to_render.push(Event::Html(wrapped_code.into()));
            }
            Event::Text(ref text) => {
                if in_heading {
                    heading_text_buffer.push_str(text);
                } else if in_code_block {
                    code_block_content.push_str(text);
                } else {
                    events_to_render.push(event);
                }
            }
            other => {
                if !in_heading && !in_code_block {
                    events_to_render.push(other);
                }
            }
        }
    }

    if in_section {
        events_to_render.push(Event::Html("</section>\n".into()));
    }

    html::push_html(&mut html_output, events_to_render.into_iter());

    let words = raw.split_whitespace().count();
    let reading_time = words.div_ceil(200).max(1);

    ParsedDocument {
        html_content: html_output,
        toc,
        metadata,
        word_count: words,
        reading_time_minutes: reading_time,
    }
}

/// Escape HTML special characters for safety in attributes/pre tags.
fn html_escape(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            other => other.to_string(),
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("Dioxus 0.6 & MDX"), "dioxus-06-mdx");
        assert_eq!(slugify("---"), "section");
    }

    #[test]
    fn test_extract_frontmatter() {
        let input = "---\ntitle: Test Doc\nauthor: Alice\ntags: [a, b, c]\n---\n# Main Heading\nContent here.";
        let (meta_opt, body) = extract_frontmatter(input);
        assert!(meta_opt.is_some());
        let meta = meta_opt.unwrap();
        assert_eq!(meta.title.as_deref(), Some("Test Doc"));
        assert_eq!(meta.author.as_deref(), Some("Alice"));
        assert_eq!(meta.tags, vec!["a", "b", "c"]);
        assert!(body.starts_with("# Main Heading"));
    }

    #[test]
    fn test_preprocess_mdx() {
        let input = "<Callout type=\"warning\">\nCaution text\n</Callout>";
        let output = preprocess_mdx(input);
        assert!(output.contains("<div class=\"mdx-callout mdx-callout-warning\">"));
        assert!(output.contains("</div>"));
    }

    #[test]
    fn test_parse_markdown_document() {
        let raw = "# Getting Started\n\nThis is a paragraph with **bold** text.\n\n## Subheading\n\n```rust\nfn main() {}\n```";
        let doc = parse_markdown_document(raw);
        assert_eq!(doc.toc.len(), 2);
        assert_eq!(doc.toc[0].title, "Getting Started");
        assert_eq!(doc.toc[0].level, 1);
        assert_eq!(doc.toc[1].title, "Subheading");
        assert_eq!(doc.toc[1].level, 2);
        assert!(doc.html_content.contains("<section class=\"markdown-section markdown-section-h1\">"));
        assert!(doc.html_content.contains("<section class=\"markdown-section markdown-section-h2\">"));
        assert!(doc.html_content.contains("code-block-container"));
        assert!(doc.word_count > 0);
    }
}

