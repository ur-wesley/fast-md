use std::fmt::Write as FmtWrite;

/// Preprocess MDX specific syntax (JSX tags, Callouts, Steps, Badges) into HTML compatible syntax.
#[must_use]
pub fn preprocess_mdx(input: &str) -> String {
    let mut output = String::with_capacity(input.len() + 256);
    let lines = input.lines();

    for line in lines {
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
            let _ = writeln!(output, "<div class=\"mdx-callout mdx-callout-{callout_type}\">");
            output.push('\n');
        } else if trimmed == "</Callout>" || trimmed == "</Note>" || trimmed == "</Warning>" || trimmed == "</Info>" {
            output.push('\n');
            output.push_str("</div>\n");
        } else if trimmed.starts_with("<Card") {
            output.push_str("<div class=\"mdx-card\">\n\n");
        } else if trimmed == "</Card>" {
            output.push('\n');
            output.push_str("</div>\n");
        } else if trimmed.starts_with("<Badge") {
            output.push_str("<span class=\"mdx-badge\">\n\n");
        } else if trimmed == "</Badge>" {
            output.push('\n');
            output.push_str("</span>\n");
        } else if trimmed.starts_with("<Steps>") {
            output.push_str("<div class=\"mdx-steps\">\n\n");
        } else if trimmed == "</Steps>" {
            output.push('\n');
            output.push_str("</div>\n");
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_mdx() {
        let input = "<Callout type=\"warning\">\nCaution text\n</Callout>";
        let output = preprocess_mdx(input);
        assert!(output.contains("<div class=\"mdx-callout mdx-callout-warning\">"));
        assert!(output.contains("</div>"));
    }

    #[test]
    fn test_preprocess_mdx_blank_lines_around_wrappers() {
        let input = "<Callout type=\"info\">\n  **bold**\n</Callout>";
        let output = preprocess_mdx(input);
        assert!(
            output.contains("<div class=\"mdx-callout mdx-callout-info\">\n\n"),
            "open wrapper should be followed by blank line: {output:?}"
        );
        assert!(
            output.contains("\n\n</div>"),
            "close wrapper should be preceded by blank line: {output:?}"
        );
    }
}
