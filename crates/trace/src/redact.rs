use regex::Regex;
use std::sync::LazyLock;

static PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"(?i)(api[_-]?key|secret|token|password)\s*[:=]\s*\S+",
        r"sk-[a-zA-Z0-9]{20,}",
        r"ghp_[a-zA-Z0-9]{20,}",
        r"AKIA[0-9A-Z]{16}",
    ]
    .iter()
    .filter_map(|p| Regex::new(p).ok())
    .collect()
});

pub fn redact_secrets(input: &str) -> String {
    let mut out = input.to_string();
    for re in PATTERNS.iter() {
        out = re
            .replace_all(&out, |caps: &regex::Captures| {
                let m = caps.get(0).map(|x| x.as_str()).unwrap_or("");
                if let Some((prefix, _)) = m.split_once(':').or_else(|| m.split_once('=')) {
                    format!("{}:[REDACTED]", prefix.trim())
                } else {
                    "[REDACTED]".to_string()
                }
            })
            .into_owned();
    }
    out
}

pub fn contains_secrets(input: &str) -> bool {
    PATTERNS.iter().any(|re| re.is_match(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_key_line() {
        let s = "api_key: sk-abcdefghijklmnopqrstuvwxyz123456";
        assert!(contains_secrets(s));
        let r = redact_secrets(s);
        assert!(!r.contains("sk-abc"));
    }
}
