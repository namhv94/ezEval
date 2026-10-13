"""Automated Evaluation Runner and Benchmark Reporter.
Runs test suites in batch and generates detailed markdown/json compliance reports.
"""

from __future__ import annotations

import time
from dataclasses import dataclass, field
from typing import Any, Dict, List

from ezeval.guardrails.safety import SafetyGuardrail
from ezeval.schema.validator import SchemaValidator


@dataclass
class TestCaseResult:
    case_id: str
    passed: bool
    schema_valid: bool
    safety_violations_count: int
    execution_time_ms: float
    details: str


@dataclass
class EvalSuiteResult:
    suite_name: str
    total_cases: int
    passed_cases: int
    failed_cases: int
    total_duration_ms: float
    results: List[TestCaseResult] = field(default_factory=list)

    @property
    def pass_rate_percentage(self) -> float:
        if self.total_cases == 0:
            return 100.0
        return (self.passed_cases / self.total_cases) * 100.0

    def to_markdown_report(self) -> str:
        lines = [
            f"# 📊 ezEval Benchmark Report: {self.suite_name}",
            "",
            f"- **Total Test Cases**: {self.total_cases}",
            f"- **Passed**: {self.passed_cases} ({self.pass_rate_percentage:.1f}%)",
            f"- **Failed**: {self.failed_cases}",
            f"- **Execution Time**: {self.total_duration_ms:.2f}ms",
            "",
            "| Case ID | Status | Schema Valid | Violations | Latency (ms) | Notes |",
            "| :--- | :---: | :---: | :---: | :---: | :--- |",
        ]
        for r in self.results:
            icon = "✅ PASS" if r.passed else "❌ FAIL"
            lines.append(
                f"| `{r.case_id}` | {icon} | {'Yes' if r.schema_valid else 'No'} | {r.safety_violations_count} | {r.execution_time_ms:.2f}ms | {r.details} |"
            )
        return "\n".join(lines)


class EvalRunner:
    """Executes evaluation batches over schemas and guardrail policies."""

    def __init__(self):
        self.guardrail = SafetyGuardrail()

    def run_suite(
        self,
        suite_name: str,
        test_cases: List[Dict[str, Any]],
        schema: Optional[Dict[str, Any]] = None,
    ) -> EvalSuiteResult:
        validator = SchemaValidator(schema) if schema else None
        results: List[TestCaseResult] = []
        start_all = time.perf_counter()

        for case in test_cases:
            cid = case.get("id", f"case_{len(results)}")
            payload = case.get("payload", {})
            raw_text = str(payload)

            t0 = time.perf_counter()
            schema_ok = True
            details = "OK"

            if validator:
                s_res = validator.validate(payload)
                schema_ok = s_res.is_valid
                if not schema_ok:
                    details = s_res.summary()

            violations = self.guardrail.inspect(raw_text)
            passed = schema_ok and len(violations) == 0
            if passed and details == "OK":
                details = "Compliant"
            elif violations:
                details = f"Safety violations: {[v.rule_id for v in violations]}"

            elapsed = (time.perf_counter() - t0) * 1000
            results.append(
                TestCaseResult(
                    case_id=cid,
                    passed=passed,
                    schema_valid=schema_ok,
                    safety_violations_count=len(violations),
                    execution_time_ms=elapsed,
                    details=details,
                )
            )

        total_ms = (time.perf_counter() - start_all) * 1000
        passed_cnt = sum(1 for r in results if r.passed)

        return EvalSuiteResult(
            suite_name=suite_name,
            total_cases=len(results),
            passed_cases=passed_cnt,
            failed_cases=len(results) - passed_cnt,
            total_duration_ms=total_ms,
            results=results,
        )
