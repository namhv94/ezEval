use crate::guardrails::SafetyGuardrail;
use crate::schema::SchemaValidator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestCase {
    pub id: String,
    pub payload: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestCaseResult {
    pub case_id: String,
    pub passed: bool,
    pub schema_valid: bool,
    pub safety_violations_count: usize,
    pub latency_ms: f64,
    pub details: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EvalSuiteResult {
    pub suite_name: String,
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub duration_ms: f64,
    pub results: Vec<TestCaseResult>,
}

impl EvalSuiteResult {
    pub fn pass_rate_percentage(&self) -> f64 {
        if self.total_cases == 0 {
            100.0
        } else {
            (self.passed_cases as f64 / self.total_cases as f64) * 100.0
        }
    }

    pub fn to_markdown_report(&self) -> String {
        let mut lines = vec![
            format!("# 📊 ezEval Benchmark Report: {}", self.suite_name),
            "".to_string(),
            format!("- **Total Test Cases**: {}", self.total_cases),
            format!(
                "- **Passed**: {} ({:.1}%)",
                self.passed_cases,
                self.pass_rate_percentage()
            ),
            format!("- **Failed**: {}", self.failed_cases),
            format!("- **Execution Time**: {:.2}ms", self.duration_ms),
            "".to_string(),
            "| Case ID | Status | Schema Valid | Violations | Latency (ms) | Notes |".to_string(),
            "| :--- | :---: | :---: | :---: | :---: | :--- |".to_string(),
        ];

        for r in &self.results {
            let icon = if r.passed { "✅ PASS" } else { "❌ FAIL" };
            lines.push(format!(
                "| `{}` | {} | {} | {} | {:.2}ms | {} |",
                r.case_id,
                icon,
                if r.schema_valid { "Yes" } else { "No" },
                r.safety_violations_count,
                r.latency_ms,
                r.details
            ));
        }

        lines.join("\n")
    }
}

pub struct EvalRunner {
    guardrail: SafetyGuardrail,
}

impl Default for EvalRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalRunner {
    pub fn new() -> Self {
        Self {
            guardrail: SafetyGuardrail::new(),
        }
    }

    pub fn run_suite(
        &self,
        suite_name: &str,
        test_cases: Vec<TestCase>,
        schema: Option<Value>,
    ) -> EvalSuiteResult {
        let validator = schema.map(SchemaValidator::new);
        let mut results = Vec::new();
        let start_all = Instant::now();

        for case in test_cases {
            let t0 = Instant::now();
            let mut schema_ok = true;
            let mut details = "OK".to_string();

            if let Some(v) = &validator {
                let s_res = v.validate(&case.payload);
                schema_ok = s_res.is_valid;
                if !schema_ok {
                    details = s_res.summary();
                }
            }

            let raw_text = case.payload.to_string();
            let violations = self.guardrail.inspect(&raw_text);
            let passed = schema_ok && violations.is_empty();

            if passed && details == "OK" {
                details = "Compliant".to_string();
            } else if !violations.is_empty() {
                let rules: Vec<String> = violations.iter().map(|v| v.rule_id.clone()).collect();
                details = format!("Safety violations: {:?}", rules);
            }

            let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
            results.push(TestCaseResult {
                case_id: case.id,
                passed,
                schema_valid: schema_ok,
                safety_violations_count: violations.len(),
                latency_ms: elapsed_ms,
                details,
            });
        }

        let total_ms = start_all.elapsed().as_secs_f64() * 1000.0;
        let passed_cnt = results.iter().filter(|r| r.passed).count();

        EvalSuiteResult {
            suite_name: suite_name.to_string(),
            total_cases: results.len(),
            passed_cases: passed_cnt,
            failed_cases: results.len() - passed_cnt,
            duration_ms: total_ms,
            results,
        }
    }
}
