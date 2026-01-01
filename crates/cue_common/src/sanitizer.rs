use regex::Regex;
use std::sync::OnceLock;

static PATTERNS: OnceLock<Vec<(Regex, String)>> = OnceLock::new();

pub struct LogSanitizer {
    patterns: Vec<(Regex, String)>,
}

impl LogSanitizer {
    pub fn new() -> Self {
        let patterns = PATTERNS.get_or_init(|| {
            vec![
                (
                    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
                    "***@***.***".to_string(),
                ),
                // Windows/Unix path redaction: matches /home/... or C:\Users\... up to next slash/backslash
                (
                    Regex::new(r"(/home/|C:\\Users\\)[^/\\s]+").unwrap(),
                    "$1***".to_string(),
                ),
                (
                    Regex::new(r"(sk|pk)-[a-zA-Z0-9]{20,}").unwrap(),
                    "$1-***".to_string(),
                ),
                // IPv4 Address
                (
                    Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap(),
                    "***.***.***.***".to_string(),
                ),
            ]
        });

        // Clone the Vec (Regex is wrapped in Arc, so checks clone is cheapRef)
        Self {
            patterns: patterns.clone(),
        }
    }

    pub fn sanitize(&self, message: &str) -> String {
        let mut result = message.to_string();
        for (pattern, replacement) in &self.patterns {
            result = pattern.replace_all(&result, replacement).to_string();
        }
        result
    }
}

impl Default for LogSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_redaction() {
        let sanitizer = LogSanitizer::new();
        let log = "User email is user@example.com";
        assert_eq!(sanitizer.sanitize(log), "User email is ***@***.***");
    }

    #[test]
    fn test_path_redaction_windows() {
        let sanitizer = LogSanitizer::new();
        let log = r"Error in C:\Users\Alice\project";
        // C:\Users\Alice -> C:\Users\***
        // Remaining: \project is untouched unless recursive, but the goal is to hide username
        let sanitized = sanitizer.sanitize(log);
        assert!(sanitized.contains(r"C:\Users\***"));
        assert!(sanitized.ends_with(r"\project"));
    }

    #[test]
    fn test_path_redaction_linux() {
        let sanitizer = LogSanitizer::new();
        let log = "/home/alice/project";
        let sanitized = sanitizer.sanitize(log);
        assert!(sanitized.contains("/home/***"));
        assert!(sanitized.ends_with("/project"));
    }

    #[test]
    fn test_api_key_redaction() {
        let sanitizer = LogSanitizer::new();
        let log = "Key: sk-12345678901234567890abcdef";
        assert_eq!(sanitizer.sanitize(log), "Key: sk-***");
    }

    #[test]
    fn test_ip_redaction() {
        let sanitizer = LogSanitizer::new();
        let log = "Connection from 192.168.1.1";
        assert_eq!(sanitizer.sanitize(log), "Connection from ***.***.***.***");
    }
}
