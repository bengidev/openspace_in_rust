/// A utility for stripping sensitive patterns from strings before logging or auditing.
///
/// Patterns commonly include API keys, tokens, and passwords that may appear
/// in command output, URLs, or serialized configuration.
#[derive(Debug, Clone)]
pub struct Redaction {
    patterns: Vec<String>,
}

impl Default for Redaction {
    fn default() -> Self {
        Self::new()
    }
}

impl Redaction {
    /// Create a `Redaction` with the default built-in patterns.
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // Bearer tokens (catches Authorization: Bearer ... and standalone Bearer ...)
                "Bearer ".to_string(),
                // Common API key parameter names in URLs/query strings
                "api_key=".to_string(),
                "apikey=".to_string(),
                "api-key=".to_string(),
                "token=".to_string(),
                "access_token=".to_string(),
                "secret=".to_string(),
                "password=".to_string(),
                // Common header forms
                "x-api-key: ".to_string(),
            ],
        }
    }

    /// Create an empty `Redaction` with no patterns.
    pub fn empty() -> Self {
        Self { patterns: vec![] }
    }

    /// Add a custom pattern to the redaction list.
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    /// Strip all known sensitive patterns from the input string, replacing
    /// each occurrence with `[REDACTED]`.
    pub fn redact(&self, input: &str) -> String {
        let mut output = input.to_string();
        for pattern in &self.patterns {
            // Simple case-insensitive replacement of the pattern prefix
            output = redact_pattern(&output, pattern);
        }
        output
    }
}

/// Replace the value following a sensitive pattern prefix with `[REDACTED]`.
fn redact_pattern(input: &str, pattern: &str) -> String {
    let lower_input = input.to_lowercase();
    let lower_pattern = pattern.to_lowercase();

    let mut result = String::with_capacity(input.len());
    let mut last_end = 0;

    for (idx, _) in lower_input.match_indices(&lower_pattern) {
        let _pattern_start = idx;
        let pattern_end = idx + lower_pattern.len();

        // Append everything up to and including the pattern prefix
        result.push_str(&input[last_end..pattern_end]);

        // Find the value after the pattern (up to next whitespace, comma, &, or end)
        let rest = &input[pattern_end..];
        let value_len = rest
            .find(|c: char| c.is_whitespace() || c == ',' || c == '&' || c == ';')
            .unwrap_or(rest.len());

        if value_len > 0 {
            result.push_str("[REDACTED]");
            last_end = pattern_end + value_len;
        } else {
            last_end = pattern_end;
        }
    }

    result.push_str(&input[last_end..]);
    result
}

/// Convenience function to redact a string using the default patterns.
pub fn redact(input: &str) -> String {
    Redaction::new().redact(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_bearer_token() {
        let input = "Authorization: Bearer sk-1234567890abcdef";
        let output = redact(input);
        assert_eq!(output, "Authorization: Bearer [REDACTED]");
    }

    #[test]
    fn test_redact_api_key_in_url() {
        let input = "https://example.com/api?api_key=supersecret123&user=alice";
        let output = redact(input);
        assert_eq!(
            output,
            "https://example.com/api?api_key=[REDACTED]&user=alice"
        );
    }

    #[test]
    fn test_redact_multiple_patterns() {
        let input = "token=abc123 secret=mySecretValue";
        let output = redact(input);
        assert_eq!(output, "token=[REDACTED] secret=[REDACTED]");
    }

    #[test]
    fn test_redact_no_match() {
        let input = "hello world, nothing sensitive here";
        let output = redact(input);
        assert_eq!(output, "hello world, nothing sensitive here");
    }

    #[test]
    fn test_redact_custom_pattern() {
        let redaction = Redaction::empty().with_pattern("custom_secret=");
        let input = "custom_secret=very_private_data";
        let output = redaction.redact(input);
        assert_eq!(output, "custom_secret=[REDACTED]");
    }

    #[test]
    fn test_redact_case_insensitive() {
        let input = "API_KEY=topsecret";
        let output = redact(input);
        assert_eq!(output, "API_KEY=[REDACTED]");
    }
}
