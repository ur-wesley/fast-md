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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

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
}
