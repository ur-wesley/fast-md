use crate::types::DocumentFormat;

/// Format document according to its format (Markdown, JSON, TOML, YAML, etc.).
pub fn format_document(source: &str, format: DocumentFormat) -> Result<String, String> {
    match format {
        DocumentFormat::Markdown | DocumentFormat::Mdx => Ok(format_markdown(source)),
        DocumentFormat::Json => format_json(source),
        DocumentFormat::Toml => format_toml(source),
        DocumentFormat::Yaml => format_yaml(source),
        _ => Ok(source.to_string()),
    }
}

/// Format JSON document with 2-space pretty printing.
pub fn format_json(source: &str) -> Result<String, String> {
    if source.trim().is_empty() {
        return Ok(String::new());
    }
    let val: serde_json::Value = serde_json::from_str(source)
        .map_err(|e| format!("Invalid JSON: {e}"))?;
    let mut formatted = serde_json::to_string_pretty(&val)
        .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
    formatted.push('\n');
    Ok(formatted)
}

/// Minify a JSON string.
#[allow(dead_code)]
pub fn minify_json(source: &str) -> Result<String, String> {
    if source.trim().is_empty() {
        return Ok(String::new());
    }
    let val: serde_json::Value = serde_json::from_str(source)
        .map_err(|e| format!("Invalid JSON: {e}"))?;
    serde_json::to_string(&val).map_err(|e| format!("Failed to minify JSON: {e}"))
}

/// Format TOML document.
pub fn format_toml(source: &str) -> Result<String, String> {
    if source.trim().is_empty() {
        return Ok(String::new());
    }
    let val: toml::Value = toml::from_str(source)
        .map_err(|e| format!("Invalid TOML: {e}"))?;
    let mut formatted = toml::to_string_pretty(&val)
        .map_err(|e| format!("Failed to serialize TOML: {e}"))?;
    if !formatted.ends_with('\n') {
        formatted.push('\n');
    }
    Ok(formatted)
}

/// Format YAML document.
pub fn format_yaml(source: &str) -> Result<String, String> {
    if source.trim().is_empty() {
        return Ok(String::new());
    }
    let val: serde_yaml::Value = serde_yaml::from_str(source)
        .map_err(|e| format!("Invalid YAML: {e}"))?;
    let mut formatted = serde_yaml::to_string(&val)
        .map_err(|e| format!("Failed to serialize YAML: {e}"))?;
    if !formatted.ends_with('\n') {
        formatted.push('\n');
    }
    Ok(formatted)
}

/// Markdown source formatter supporting table alignment, whitespace cleanup,
/// code block preservation, and frontmatter handling.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColumnAlignment {
    Left,
    Center,
    Right,
    None,
}

impl ColumnAlignment {
    fn parse(cell: &str) -> Self {
        let trimmed = cell.trim();
        let starts_with_colon = trimmed.starts_with(':');
        let ends_with_colon = trimmed.ends_with(':');

        if starts_with_colon && ends_with_colon {
            Self::Center
        } else if ends_with_colon {
            Self::Right
        } else if starts_with_colon {
            Self::Left
        } else {
            Self::None
        }
    }

    fn render_delimiter(self, width: usize) -> String {
        let width = width.max(3);
        match self {
            Self::Left => format!(":{}", "-".repeat(width.saturating_sub(1))),
            Self::Center => format!(":{}:", "-".repeat(width.saturating_sub(2))),
            Self::Right => format!("{}:", "-".repeat(width.saturating_sub(1))),
            Self::None => "-".repeat(width),
        }
    }
}

/// Check if a line looks like a Markdown table row or delimiter.
fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Must contain at least one pipe and start or end with a pipe, or contain multiple pipes
    trimmed.contains('|') && (trimmed.starts_with('|') || trimmed.ends_with('|') || trimmed.matches('|').count() >= 2)
}

/// Check if a line is a valid table delimiter line (e.g. `| --- | :---: | ---: |`).
fn is_table_delimiter_row(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.contains('|') || !trimmed.contains('-') {
        return false;
    }
    let cells = split_table_row(trimmed);
    if cells.is_empty() {
        return false;
    }
    cells.iter().all(|c| {
        let t = c.trim();
        !t.is_empty() && t.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')
    })
}

/// Split a table row into cells, respecting escaped pipes `\|`.
fn split_table_row(line: &str) -> Vec<String> {
    let mut trimmed = line.trim();
    if let Some(stripped) = trimmed.strip_prefix('|') {
        trimmed = stripped;
    }
    if let Some(stripped) = trimmed.strip_suffix('|') {
        trimmed = stripped;
    }
    let without_outer = trimmed;

    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = without_outer.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'|') {
            current.push('|');
            chars.next();
        } else if ch == '|' {
            cells.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    cells.push(current.trim().to_string());
    cells
}

/// Format a block of table lines into an aligned ASCII grid.
fn format_table_block(lines: &[&str]) -> Vec<String> {
    if lines.len() < 2 {
        return lines.iter().map(|l| (*l).to_string()).collect();
    }

    let delimiter_idx = lines.iter().position(|l| is_table_delimiter_row(l));
    let delimiter_idx = match delimiter_idx {
        Some(idx) if idx > 0 => idx,
        _ => return lines.iter().map(|l| (*l).to_string()).collect(),
    };

    // Parse all rows into cells
    let parsed_rows: Vec<Vec<String>> = lines.iter().map(|l| split_table_row(l)).collect();
    let num_cols = parsed_rows.iter().map(Vec::len).max().unwrap_or(0);
    if num_cols == 0 {
        return lines.iter().map(|l| (*l).to_string()).collect();
    }

    // Parse alignments from delimiter row
    let alignments: Vec<ColumnAlignment> = (0..num_cols)
        .map(|col_idx| {
            parsed_rows
                .get(delimiter_idx)
                .and_then(|row| row.get(col_idx))
                .map_or(ColumnAlignment::None, |c| ColumnAlignment::parse(c))
        })
        .collect();

    // Compute column widths
    let mut col_widths = vec![3usize; num_cols];
    for (row_idx, row) in parsed_rows.iter().enumerate() {
        if row_idx == delimiter_idx {
            continue;
        }
        for (col_idx, cell) in row.iter().enumerate() {
            if col_idx < num_cols {
                col_widths[col_idx] = col_widths[col_idx].max(cell.len());
            }
        }
    }

    // Format rows
    let mut formatted_rows = Vec::with_capacity(parsed_rows.len());
    for (row_idx, row) in parsed_rows.iter().enumerate() {
        if row_idx == delimiter_idx {
            // Delimiter row
            let delimiter_cells: Vec<String> = (0..num_cols)
                .map(|col_idx| {
                    let align = alignments.get(col_idx).copied().unwrap_or(ColumnAlignment::None);
                    let width = col_widths.get(col_idx).copied().unwrap_or(3);
                    align.render_delimiter(width)
                })
                .collect();
            formatted_rows.push(format!("| {} |", delimiter_cells.join(" | ")));
        } else {
            // Header or Body row
            let padded_cells: Vec<String> = (0..num_cols)
                .map(|col_idx| {
                    let cell_content = row.get(col_idx).map_or("", String::as_str);
                    let width = col_widths.get(col_idx).copied().unwrap_or(3);
                    let align = alignments.get(col_idx).copied().unwrap_or(ColumnAlignment::None);

                    match align {
                        ColumnAlignment::Right => format!("{cell_content:>width$}"),
                        ColumnAlignment::Center => {
                            let total_pad = width.saturating_sub(cell_content.len());
                            let left_pad = total_pad / 2;
                            let right_pad = total_pad.saturating_sub(left_pad);
                            format!("{}{}{}", " ".repeat(left_pad), cell_content, " ".repeat(right_pad))
                        }
                        ColumnAlignment::Left | ColumnAlignment::None => {
                            format!("{cell_content:<width$}")
                        }
                    }
                })
                .collect();
            formatted_rows.push(format!("| {} |", padded_cells.join(" | ")));
        }
    }

    formatted_rows
}

/// Format entire Markdown document.
/// - Preserves YAML frontmatter untouched.
/// - Preserves code blocks (` ``` ` and `~~~`) untouched.
/// - Formats tables into aligned ASCII tables.
/// - Normalizes empty lines (max 2 consecutive empty lines).
/// - Trims trailing whitespace on all lines.
/// - Ensures single trailing newline.
#[must_use]
pub fn format_markdown(source: &str) -> String {
    if source.trim().is_empty() {
        return String::new();
    }

    let raw_lines: Vec<&str> = source.lines().collect();
    let mut formatted_lines: Vec<String> = Vec::new();

    let mut in_frontmatter = false;
    let mut in_code_block = false;
    let mut code_fence_char = '`';
    let mut code_fence_len = 0;

    let mut i = 0;
    while i < raw_lines.len() {
        let line = raw_lines[i];
        let trimmed = line.trim();

        // 1. Frontmatter detection at start of document
        if i == 0 && (trimmed == "---" || trimmed == "+++") {
            in_frontmatter = true;
            formatted_lines.push(line.to_string());
            i += 1;
            continue;
        }

        if in_frontmatter {
            formatted_lines.push(line.to_string());
            if trimmed == "---" || trimmed == "+++" {
                in_frontmatter = false;
            }
            i += 1;
            continue;
        }

        // 2. Code fence detection (``` or ~~~)
        let trimmed_start = line.trim_start();
        if !in_code_block && (trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~")) {
            let fence_char = trimmed_start.chars().next().unwrap_or('`');
            let fence_count = trimmed_start.chars().take_while(|&c| c == fence_char).count();
            in_code_block = true;
            code_fence_char = fence_char;
            code_fence_len = fence_count;
            formatted_lines.push(line.trim_end().to_string());
            i += 1;
            continue;
        }

        if in_code_block {
            let fence_prefix = code_fence_char.to_string().repeat(code_fence_len);
            if trimmed_start.starts_with(&fence_prefix) && trimmed_start.chars().all(|c| c == code_fence_char || c.is_whitespace()) {
                in_code_block = false;
            }
            // In code blocks, preserve line content exactly as is (only trim trailing newline)
            formatted_lines.push(line.to_string());
            i += 1;
            continue;
        }

        // 3. Table block detection
        if is_table_row(line) {
            let mut table_lines = Vec::new();
            while i < raw_lines.len() && is_table_row(raw_lines[i]) {
                table_lines.push(raw_lines[i]);
                i += 1;
            }

            let formatted_table = format_table_block(&table_lines);
            formatted_lines.extend(formatted_table);
            continue;
        }

        // 4. Regular line formatting
        let trimmed_line = line.trim_end();

        // Format headings: ensure single space after '#' (e.g. '##Heading' -> '## Heading')
        if trimmed_line.starts_with('#') {
            let hash_count = trimmed_line.chars().take_while(|&c| c == '#').count();
            if hash_count <= 6 {
                let rest = &trimmed_line[hash_count..];
                if !rest.starts_with(' ') && !rest.is_empty() {
                    formatted_lines.push(format!("{} {}", &trimmed_line[..hash_count], rest.trim()));
                    i += 1;
                    continue;
                }
            }
        }

        formatted_lines.push(trimmed_line.to_string());
        i += 1;
    }

    // 5. Clean up excessive consecutive blank lines (limit to max 2 blank lines)
    let mut normalized_lines: Vec<String> = Vec::new();
    let mut blank_count = 0;

    for line in formatted_lines {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                normalized_lines.push(String::new());
            }
        } else {
            blank_count = 0;
            normalized_lines.push(line);
        }
    }

    // 6. Ensure single trailing newline
    while normalized_lines.last().is_some_and(String::is_empty) {
        normalized_lines.pop();
    }

    let mut result = normalized_lines.join("\n");
    result.push('\n');
    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_format_markdown_tables() {
        let unformatted = r"
# Test Document

| Name|Age |City|
|:---|:---:|---:|
|Alice |24|New York |
|Bob| 30|San Francisco|
|Charlie| 22 |London|

Paragraph text.
";

        let formatted = format_markdown(unformatted);
        let expected_table_header = "| Name    | Age |          City |";
        let expected_delimiter = "| :------ | :-: | ------------: |";
        let expected_row = "| Alice   | 24  |      New York |";

        assert!(formatted.contains(expected_table_header), "Formatted:\n{formatted}");
        assert!(formatted.contains(expected_delimiter), "Formatted:\n{formatted}");
        assert!(formatted.contains(expected_row), "Formatted:\n{formatted}");
    }

    #[test]
    fn test_format_preserves_code_blocks() {
        let unformatted = "```rust\n  let x = 10;\n    let y = 20;\n```\n";
        let formatted = format_markdown(unformatted);
        assert_eq!(formatted, unformatted);
    }

    #[test]
    fn test_format_preserves_frontmatter() {
        let input = "---\ntitle: Fast-MD\nauthor: Antigravity\n---\n\n# Heading\n";
        let formatted = format_markdown(input);
        assert_eq!(formatted, input);
    }

    #[test]
    fn test_heading_normalization() {
        let input = "##Heading 2\n###Heading 3\n";
        let formatted = format_markdown(input);
        assert_eq!(formatted, "## Heading 2\n### Heading 3\n");
    }

    #[test]
    fn test_blank_lines_collapse() {
        let input = "Paragraph 1\n\n\n\n\nParagraph 2\n";
        let formatted = format_markdown(input);
        assert_eq!(formatted, "Paragraph 1\n\n\nParagraph 2\n");
    }

    #[test]
    fn test_format_json() {
        let raw = r#"{"name":"fast-md","version":"0.1.2","dependencies":{"dioxus":"0.6"}}"#;
        if let Ok(formatted) = format_json(raw) {
            assert!(formatted.contains("  \"name\": \"fast-md\""));
            assert!(formatted.contains("  \"dependencies\": {"));
            assert!(formatted.ends_with('\n'));

            if let Ok(minified) = minify_json(&formatted) {
                assert_eq!(minified, r#"{"dependencies":{"dioxus":"0.6"},"name":"fast-md","version":"0.1.2"}"#);
            } else {
                panic!("failed to minify json");
            }
        } else {
            panic!("failed to format json");
        }
    }

    #[test]
    fn test_format_toml() {
        let raw = "[package]\nname=\"fast-md\"\nversion=\"0.1.2\"\n";
        if let Ok(formatted) = format_toml(raw) {
            assert!(formatted.contains("[package]"));
            assert!(formatted.contains("name = \"fast-md\""));
            assert!(formatted.contains("version = \"0.1.2\""));
        } else {
            panic!("failed to format toml");
        }
    }

    #[test]
    fn test_format_yaml() {
        let raw = "name: fast-md\nversion: '0.1.2'\nfeatures:\n- desktop\n- syntect\n";
        if let Ok(formatted) = format_yaml(raw) {
            assert!(formatted.contains("name: fast-md"));
            assert!(formatted.contains("- desktop"));
        } else {
            panic!("failed to format yaml");
        }
    }
}
