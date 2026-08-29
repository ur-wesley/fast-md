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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

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
