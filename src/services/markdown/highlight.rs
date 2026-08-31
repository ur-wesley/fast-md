use super::{frontmatter, mdx};
use crate::types::{DocumentFormat, ParsedDocument, TocItem};
use pulldown_cmark::{html, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::sync::OnceLock;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

pub(crate) fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

pub(crate) fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

pub(crate) fn slugify(text: &str) -> String {
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

pub(crate) fn html_escape(input: &str) -> String {
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

fn is_mdx_wrapper_close(html: &str) -> bool {
    matches!(html.trim(), "</div>" | "</span>")
}

fn close_open_section(events: &mut Vec<Event>, in_section: &mut bool) {
    if *in_section {
        events.push(Event::Html("</section>\n".into()));
        *in_section = false;
    }
}

/// Parse full Markdown/MDX text into rendered HTML with table of contents and metadata.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn parse_markdown_document(raw: &str) -> ParsedDocument {
    let (metadata, body) = frontmatter::extract_frontmatter(raw);
    let preprocessed = mdx::preprocess_mdx(body);

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
            Event::TaskListMarker(checked) => {
                let checked_attr = if checked { " checked=\"\"" } else { "" };
                events_to_render.push(Event::Html(
                    format!("<input type=\"checkbox\"{checked_attr}/> ").into(),
                ));
            }
            other => {
                if !in_heading && !in_code_block {
                    if let Event::Html(ref html) = other {
                        if is_mdx_wrapper_close(html) {
                            close_open_section(&mut events_to_render, &mut in_section);
                        }
                    }
                    events_to_render.push(other);
                }
            }
        }
    }

    if in_section {
        close_open_section(&mut events_to_render, &mut in_section);
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
        format: DocumentFormat::Markdown,
        validation_error: None,
    }
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

    #[test]
    fn test_parse_markdown_tasklists() {
        let raw = "- [ ] Unchecked Task\n- [x] Checked Task";
        let doc = parse_markdown_document(raw);
        assert!(doc.html_content.contains("<input type=\"checkbox\"/>"));
        assert!(doc.html_content.contains("<input type=\"checkbox\" checked=\"\"/>"));
        assert!(!doc.html_content.contains("disabled"));
    }

    #[test]
    fn test_parse_mdx_callout_inner_markdown() {
        let raw = "<Callout type=\"info\">\n  This is a custom MDX **Callout** component.\n</Callout>";
        let doc = parse_markdown_document(raw);
        assert!(
            doc.html_content.contains("<strong>Callout</strong>"),
            "callout inner bold should render: {}",
            doc.html_content
        );
        assert!(
            !doc.html_content.contains("**Callout**"),
            "callout inner markdown should not stay literal: {}",
            doc.html_content
        );
    }

    #[test]
    fn test_parse_mdx_card_inner_markdown() {
        let raw = "<Card>\n  ### Interactive Documentation\n  Organize docs with sidebars.\n</Card>";
        let doc = parse_markdown_document(raw);
        assert!(
            doc.html_content.contains("markdown-section-h3")
                && doc.html_content.contains("Interactive Documentation</h3>"),
            "card inner heading should render: {}",
            doc.html_content
        );
        assert!(
            !doc.html_content.contains("### Interactive Documentation"),
            "card inner markdown should not stay literal: {}",
            doc.html_content
        );
        let card_close = doc.html_content.find("</div>").expect("card div close");
        let section_close = doc
            .html_content
            .find("</section>")
            .expect("section close");
        assert!(
            section_close < card_close,
            "section must close before card wrapper: {}",
            doc.html_content
        );
    }

    #[test]
    fn test_welcome_mdx_showcase() {
        let raw = include_str!("../../assets/welcome.md");
        let doc = parse_markdown_document(raw);
        assert!(
            doc.html_content.contains("<strong>Callout</strong>"),
            "welcome callout bold should render: {}",
            doc.html_content
        );
        assert!(
            doc.html_content.contains("Interactive Documentation</h3>"),
            "welcome card heading should render: {}",
            doc.html_content
        );
        assert!(!doc.html_content.contains("**Callout**"));
        assert!(!doc.html_content.contains("### Interactive Documentation"));
    }
}
