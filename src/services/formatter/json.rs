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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

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
}
