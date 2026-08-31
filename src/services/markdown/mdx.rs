/// Preprocess MDX specific syntax (JSX tags, Callouts, Steps, Badges) into HTML compatible syntax.
#[must_use]
pub fn preprocess_mdx(input: &str) -> (String, Vec<usize>) {
    let mut output = String::with_capacity(input.len() + 256);
    let mut line_map = Vec::new();

    let mut emit_line = |content: &str, source_line: usize| {
        output.push_str(content);
        output.push('\n');
        line_map.push(source_line);
    };

    for (source_line, line) in input.lines().enumerate() {
        let trimmed = line.trim();

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
            let callout_open = format!("<div class=\"mdx-callout mdx-callout-{callout_type}\">");
            emit_line(&callout_open, source_line);
            emit_line("", source_line);
        } else if trimmed == "</Callout>" || trimmed == "</Note>" || trimmed == "</Warning>" || trimmed == "</Info>" {
            emit_line("", source_line);
            emit_line("</div>", source_line);
        } else if trimmed.starts_with("<Card") {
            emit_line("<div class=\"mdx-card\">", source_line);
            emit_line("", source_line);
        } else if trimmed == "</Card>" {
            emit_line("", source_line);
            emit_line("</div>", source_line);
        } else if trimmed.starts_with("<Badge") {
            emit_line("<span class=\"mdx-badge\">", source_line);
            emit_line("", source_line);
        } else if trimmed == "</Badge>" {
            emit_line("", source_line);
            emit_line("</span>", source_line);
        } else if trimmed.starts_with("<Steps>") {
            emit_line("<div class=\"mdx-steps\">", source_line);
            emit_line("", source_line);
        } else if trimmed == "</Steps>" {
            emit_line("", source_line);
            emit_line("</div>", source_line);
        } else {
            emit_line(line, source_line);
        }
    }

    (output, line_map)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_mdx() {
        let input = "<Callout type=\"warning\">\nCaution text\n</Callout>";
        let (output, line_map) = preprocess_mdx(input);
        assert!(output.contains("<div class=\"mdx-callout mdx-callout-warning\">"));
        assert!(output.contains("</div>"));
        assert_eq!(line_map.len(), output.matches('\n').count());
        assert_eq!(line_map[0], 0);
    }

    #[test]
    fn test_preprocess_mdx_blank_lines_around_wrappers() {
        let input = "<Callout type=\"info\">\n  **bold**\n</Callout>";
        let (output, line_map) = preprocess_mdx(input);
        assert!(
            output.contains("<div class=\"mdx-callout mdx-callout-info\">\n\n"),
            "open wrapper should be followed by blank line: {output:?}"
        );
        assert!(
            output.contains("\n\n</div>"),
            "close wrapper should be preceded by blank line: {output:?}"
        );
        assert_eq!(line_map[0], 0);
        assert_eq!(line_map[1], 0);
        assert_eq!(line_map[2], 1);
        assert_eq!(line_map[3], 2);
        assert_eq!(line_map[4], 2);
    }
}
