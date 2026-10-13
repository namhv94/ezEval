import unittest

from ezeval.schema.validator import SchemaValidator
from ezeval.fuzzer.boundary import BoundaryFuzzer
from ezeval.guardrails.safety import SafetyGuardrail, ViolationSeverity
from ezeval.runner.evaluator import EvalRunner


class TestSchemaValidator(unittest.TestCase):
    def setUp(self):
        self.schema = {
            "type": "object",
            "properties": {
                "username": {"type": "string", "minLength": 3, "maxLength": 20},
                "score": {"type": "integer", "minimum": 0, "maximum": 100},
                "is_admin": {"type": "boolean"},
            },
            "required": ["username", "score"],
            "additionalProperties": False,
        }
        self.validator = SchemaValidator(self.schema)

    def test_valid_payload(self):
        res = self.validator.validate({"username": "alex", "score": 95})
        self.assertTrue(res.is_valid)
        self.assertEqual(len(res.errors), 0)

    def test_missing_required_property(self):
        res = self.validator.validate({"username": "alex"})
        self.assertFalse(res.is_valid)
        self.assertTrue(any(e.error_type == "missing_required" for e in res.errors))

    def test_type_and_range_violation(self):
        res = self.validator.validate({"username": "alex", "score": 150})
        self.assertFalse(res.is_valid)
        self.assertTrue(any(e.error_type == "maximum_violation" for e in res.errors))

    def test_unexpected_property_denied(self):
        res = self.validator.validate({"username": "alex", "score": 50, "extra": "forbidden"})
        self.assertFalse(res.is_valid)
        self.assertTrue(any(e.error_type == "unexpected_property" for e in res.errors))


class TestBoundaryFuzzer(unittest.TestCase):
    def test_mutation_generation(self):
        fuzzer = BoundaryFuzzer()
        mutations = fuzzer.generate_all_mutations()
        self.assertGreater(len(mutations), 10)
        categories = {m.category for m in mutations}
        self.assertIn("unicode", categories)
        self.assertIn("numeric", categories)
        self.assertIn("injection", categories)


class TestSafetyGuardrail(unittest.TestCase):
    def setUp(self):
        self.guardrail = SafetyGuardrail()

    def test_clean_input(self):
        violations = self.guardrail.inspect("Safe message for deployment")
        self.assertEqual(len(violations), 0)
        self.assertTrue(self.guardrail.is_safe("Safe message for deployment"))

    def test_destructive_command_detection(self):
        text = "Please run `rm -rf /` to clean the disk"
        violations = self.guardrail.inspect(text)
        self.assertGreater(len(violations), 0)
        self.assertTrue(any(v.rule_id == "hazard_destructive_rm" for v in violations))
        self.assertFalse(self.guardrail.is_safe(text))

    def test_pii_email_and_phone(self):
        text = "Contact support at test@example.com or 0912345678"
        violations = self.guardrail.inspect(text)
        self.assertGreaterEqual(len(violations), 2)


class TestEvalRunner(unittest.TestCase):
    def test_benchmark_runner(self):
        runner = EvalRunner()
        schema = {"type": "object", "properties": {"msg": {"type": "string"}}, "required": ["msg"]}
        cases = [
            {"id": "c1", "payload": {"msg": "hello"}},
            {"id": "c2", "payload": {"invalid": 123}},
        ]
        res = runner.run_suite("simple_suite", cases, schema=schema)
        self.assertEqual(res.total_cases, 2)
        self.assertEqual(res.passed_cases, 1)
        self.assertEqual(res.failed_cases, 1)
        report_md = res.to_markdown_report()
        self.assertIn("ezEval Benchmark Report", report_md)


if __name__ == "__main__":
    unittest.main()
