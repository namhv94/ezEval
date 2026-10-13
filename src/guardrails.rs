use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuardrailViolation {
    pub rule_id: String,
    pub description: String,
    pub severity: ViolationSeverity,
    pub matched_snippet: String,
}

pub struct SafetyGuardrail {
    rules: Vec<(&'static str, Regex, ViolationSeverity, &'static str)>,
}

impl Default for SafetyGuardrail {
    fn default() -> Self {
        Self::new()
    }
}

impl SafetyGuardrail {
    pub fn new() -> Self {
        let rules = vec![
            (
                "secret_generic_bearer",
                Regex::new(r"Bearer\s+[a-zA-Z0-9_\-\.]{25,}").unwrap(),
                ViolationSeverity::High,
                "Generic Bearer Token detected",
            ),
            (
                "secret_aws_key",
                Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
                ViolationSeverity::Critical,
                "AWS Access Key ID detected",
            ),
            (
                "pii_email",
                Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap(),
                ViolationSeverity::Medium,
                "Personal Email Address detected",
            ),
            (
                "pii_phone_vn",
                Regex::new(r"(?:\+84|0)(?:3[2-9]|5[6|8|9]|7[0|6-9]|8[1-5]|9[0-9])[0-9]{7}")
                    .unwrap(),
                ViolationSeverity::Medium,
                "Vietnam Phone Number detected",
            ),
            (
                "pii_credit_card",
                Regex::new(r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13})\b")
                    .unwrap(),
                ViolationSeverity::Critical,
                "Credit Card format detected",
            ),
            (
                "hazard_destructive_rm",
                Regex::new(r"rm\s+-(?:rf|fr)\s+(?:/|\.\.|~|\*)").unwrap(),
                ViolationSeverity::Critical,
                "Destructive root filesystem removal command",
            ),
            (
                "hazard_curl_pipe_sh",
                Regex::new(r"(?:curl|wget)\s+.*\|\s*(?:sh|bash|zsh)").unwrap(),
                ViolationSeverity::High,
                "Arbitrary remote code execution via curl/pipe bash",
            ),
            (
                "hazard_fork_bomb",
                Regex::new(r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:").unwrap(),
                ViolationSeverity::Critical,
                "Fork bomb command pattern",
            ),
        ];

        Self { rules }
    }

    pub fn inspect(&self, text: &str) -> Vec<GuardrailViolation> {
        let mut violations = Vec::new();
        for (rule_id, regex, severity, desc) in &self.rules {
            for m in regex.find_iter(text) {
                let snippet = m.as_str();
                let masked = if snippet.len() > 6 {
                    format!("{}***{}", &snippet[..3], &snippet[snippet.len() - 2..])
                } else {
                    "***".to_string()
                };
                violations.push(GuardrailViolation {
                    rule_id: rule_id.to_string(),
                    description: desc.to_string(),
                    severity: *severity,
                    matched_snippet: masked,
                });
            }
        }
        violations
    }

    pub fn is_safe(&self, text: &str, max_severity: ViolationSeverity) -> bool {
        let violations = self.inspect(text);
        !violations.iter().any(|v| v.severity >= max_severity)
    }
}
