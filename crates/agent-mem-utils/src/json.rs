//! JSON processing utilities (inspired by mem0)

use agent_mem_traits::Result;
use regex::Regex;
use serde_json::Value;

/// Extract JSON from text, handling code blocks and other formats
pub fn extract_json(text: &str) -> Result<String> {
    let text = text.trim();

    // Try to match JSON in code blocks first
    let code_block_regex = Regex::new(r"```(?:json)?\s*(.*?)\s*```").unwrap();
    if let Some(captures) = code_block_regex.captures(text) {
        let json_content = captures.get(1).unwrap().as_str();
        return Ok(json_content.to_string());
    }

    // Try to find JSON object boundaries
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                let json_candidate = &text[start..=end];
                // Validate it's proper JSON
                if serde_json::from_str::<Value>(json_candidate).is_ok() {
                    return Ok(json_candidate.to_string());
                }
            }
        }
    }

    // Try to find JSON array boundaries
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            if end > start {
                let json_candidate = &text[start..=end];
                // Validate it's proper JSON
                if serde_json::from_str::<Value>(json_candidate).is_ok() {
                    return Ok(json_candidate.to_string());
                }
            }
        }
    }

    // If no JSON structure found, assume the entire text is JSON
    Ok(text.to_string())
}

/// Remove code block markers from content
pub fn remove_code_blocks(content: &str) -> String {
    let pattern = r"^```[a-zA-Z0-9]*\n([\s\S]*?)\n```$";
    let regex = Regex::new(pattern).unwrap();

    if let Some(captures) = regex.captures(content.trim()) {
        captures.get(1).unwrap().as_str().trim().to_string()
    } else {
        content.trim().to_string()
    }
}

/// Parse JSON string to a specific type
pub fn parse_json<T>(json_str: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let cleaned_json = extract_json(json_str)?;
    let parsed: T = serde_json::from_str(&cleaned_json)
        .map_err(agent_mem_traits::AgentMemError::SerializationError)?;
    Ok(parsed)
}

/// Pretty print JSON
pub fn pretty_print_json(value: &Value) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(agent_mem_traits::AgentMemError::SerializationError)
}

/// Validate JSON string
pub fn validate_json(json_str: &str) -> bool {
    serde_json::from_str::<Value>(json_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_code_block() {
        let text = r#"
Here's the JSON:
```json
{"name": "test", "value": 42}
```
That's it.
        "#;

        let result = extract_json(text).unwrap();
        assert_eq!(result, r#"{"name": "test", "value": 42}"#);
    }

    #[test]
    fn test_extract_json_object() {
        let text = r#"The result is {"name": "test", "value": 42} and that's final."#;
        let result = extract_json(text).unwrap();
        assert_eq!(result, r#"{"name": "test", "value": 42}"#);
    }

    #[test]
    fn test_extract_json_array() {
        let text = r#"The results are [{"name": "test"}, {"name": "test2"}] here."#;
        let result = extract_json(text).unwrap();
        assert_eq!(result, r#"[{"name": "test"}, {"name": "test2"}]"#);
    }

    #[test]
    fn test_remove_code_blocks() {
        let content = r#"```json
{"test": true}
```"#;
        let result = remove_code_blocks(content);
        assert_eq!(result, r#"{"test": true}"#);
    }

    #[test]
    fn test_validate_json() {
        assert!(validate_json(r#"{"valid": true}"#));
        assert!(!validate_json(r#"{"invalid": }"#));
    }
}
