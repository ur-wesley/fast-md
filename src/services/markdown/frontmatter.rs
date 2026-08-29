use crate::types::DocMetadata;
use std::collections::BTreeMap;

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

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
}
