//! ezEval: High-throughput boundary fuzzing, JSON Schema enforcement, and safety guardrails.
//! A Product of EzLabs (https://ezlabs.tech)

pub mod fuzzer;
pub mod guardrails;
pub mod runner;
pub mod schema;

pub use fuzzer::{BoundaryFuzzer, FuzzPayload};
pub use guardrails::{GuardrailViolation, SafetyGuardrail, ViolationSeverity};
pub use runner::{EvalRunner, EvalSuiteResult, TestCase, TestCaseResult};
pub use schema::{SchemaValidator, ValidationError, ValidationResult};
