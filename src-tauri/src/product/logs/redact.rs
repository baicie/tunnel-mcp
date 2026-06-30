use regex::Regex;
use serde_json::Value;

#[allow(dead_code)]
pub fn redact_text(input: &str) -> String {
    let patterns = [
        r"sk-[A-Za-z0-9_\-]{8,}",
        r"(?i)(openai[_-]?api[_-]?key\s*[:=]\s*)[^\s,}]+",
        r"(?i)(token\s*[:=]\s*)[^\s,}]+",
    ];
    let mut output = input.to_string();
    for pattern in patterns {
        let re = Regex::new(pattern).unwrap();
        output = re.replace_all(&output, "$1[REDACTED]").to_string();
    }
    output
}

#[allow(dead_code)]
pub fn redact_value(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(redact_text(&text)),
        Value::Array(items) => Value::Array(items.into_iter().map(redact_value).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    let lowered = key.to_lowercase();
                    if lowered.contains("key")
                        || lowered.contains("token")
                        || lowered.contains("secret")
                    {
                        (key, Value::String("[REDACTED]".to_string()))
                    } else {
                        (key, redact_value(value))
                    }
                })
                .collect(),
        ),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::{redact_text, redact_value};
    use serde_json::json;

    #[test]
    fn redacts_openai_key_like_values() {
        let text = "openai_api_key=sk-1234567890abcdef";
        let redacted = redact_text(text);
        assert!(!redacted.contains("sk-1234567890abcdef"));
    }

    #[test]
    fn redacts_sensitive_json_keys() {
        let value =
            redact_value(json!({ "token": "abc", "nested": { "secret": "def" }, "safe": "ok" }));
        assert_eq!(value["token"], "[REDACTED]");
        assert_eq!(value["nested"]["secret"], "[REDACTED]");
        assert_eq!(value["safe"], "ok");
    }
}
