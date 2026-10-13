use ezeval::fuzzer::BoundaryFuzzer;
use ezeval::guardrails::{SafetyGuardrail, ViolationSeverity};
use ezeval::runner::{EvalRunner, TestCase};
use ezeval::schema::SchemaValidator;
use serde_json::json;

#[test]
fn test_schema_validator() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "minLength": 2 },
            "count": { "type": "integer", "minimum": 1, "maximum": 50 }
        },
        "required": ["name", "count"],
        "additionalProperties": false
    });

    let validator = SchemaValidator::new(schema);

    // Valid
    let ok_res = validator.validate(&json!({ "name": "ezlabs", "count": 10 }));
    assert!(ok_res.is_valid);
    assert!(ok_res.errors.is_empty());

    // Missing property
    let miss_res = validator.validate(&json!({ "name": "ezlabs" }));
    assert!(!miss_res.is_valid);
    assert_eq!(miss_res.errors[0].error_type, "missing_required");

    // Unexpected property
    let unexp_res =
        validator.validate(&json!({ "name": "ezlabs", "count": 10, "extra": "denied" }));
    assert!(!unexp_res.is_valid);
    assert_eq!(unexp_res.errors[0].error_type, "unexpected_property");
}

#[test]
fn test_boundary_fuzzer() {
    let mutations = BoundaryFuzzer::generate_all_mutations();
    assert!(mutations.len() >= 10);
    assert!(mutations.iter().any(|m| m.category == "unicode"));
    assert!(mutations.iter().any(|m| m.category == "numeric"));
    assert!(mutations.iter().any(|m| m.category == "structure"));
}

#[test]
fn test_safety_guardrails() {
    let guard = SafetyGuardrail::new();

    // Clean text
    assert!(guard.is_safe("Normal operations payload", ViolationSeverity::High));

    // Hazardous command
    let rm_cmd = "Execute: rm -rf /";
    let violations = guard.inspect(rm_cmd);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].rule_id, "hazard_destructive_rm");
    assert!(!guard.is_safe(rm_cmd, ViolationSeverity::High));

    // Vietnam phone number
    let phone_txt = "Liên hệ hotline 0912345678 để được hỗ trợ";
    let phone_v = guard.inspect(phone_txt);
    assert!(!phone_v.is_empty());
    assert_eq!(phone_v[0].rule_id, "pii_phone_vn");
}

#[test]
fn test_runner_batch() {
    let runner = EvalRunner::new();
    let cases = vec![
        TestCase {
            id: "case_ok".to_string(),
            payload: json!({ "msg": "valid" }),
        },
        TestCase {
            id: "case_hazard".to_string(),
            payload: json!({ "msg": "rm -rf /" }),
        },
    ];

    let res = runner.run_suite("test_batch", cases, None);
    assert_eq!(res.total_cases, 2);
    assert_eq!(res.passed_cases, 1);
    assert_eq!(res.failed_cases, 1);
    assert!(res.to_markdown_report().contains("ezEval Benchmark Report"));
}
