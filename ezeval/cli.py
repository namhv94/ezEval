"""Command Line Interface for ezEval.
Enables schema validation, boundary fuzzing, and safety scans from the terminal.
"""

from __future__ import annotations

import argparse
import json
import sys

from ezeval import __version__
from ezeval.fuzzer.boundary import BoundaryFuzzer
from ezeval.guardrails.safety import SafetyGuardrail
from ezeval.runner.evaluator import EvalRunner
from ezeval.schema.validator import SchemaValidator


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="ezeval",
        description="ezEval: Synthetic Evals & Safety Guardrails (EzLabs)",
    )
    parser.add_argument("--version", action="version", version=f"ezeval v{__version__}")

    subparsers = parser.add_subparsers(dest="command", help="Sub-commands")

    # Command: fuzz
    fuzz_parser = subparsers.add_parser("fuzz", help="Generate synthetic boundary fuzz mutations")
    fuzz_parser.add_argument("--category", choices=["all", "unicode", "numeric", "injection"], default="all")

    # Command: check
    check_parser = subparsers.add_parser("check", help="Inspect text or JSON payload for safety violations")
    check_parser.add_argument("text", help="Text or payload string to inspect")

    # Command: benchmark
    bm_parser = subparsers.add_parser("benchmark", help="Run synthetic evaluation benchmark suite")

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        sys.exit(0)

    if args.command == "fuzz":
        fuzzer = BoundaryFuzzer()
        all_mutations = fuzzer.generate_all_mutations()
        print(f"=== ezEval Boundary Fuzzer (v{__version__}) ===")
        print(f"Generated {len(all_mutations)} synthetic edge-case mutations:\n")
        for m in all_mutations:
            if args.category == "all" or m.category == args.category:
                repr_val = repr(m.value) if len(repr(m.value)) < 40 else repr(m.value)[:40] + "..."
                print(f"[{m.category.upper()}] {m.name} ➔ {repr_val}")
                print(f"  Description: {m.description}")
        print("\nReady for injection testing.")

    elif args.command == "check":
        guard = SafetyGuardrail()
        violations = guard.inspect(args.text)
        print(f"=== ezEval Safety Scan (v{__version__}) ===")
        print(f"Target: {args.text[:80]}...")
        if not violations:
            print("✅ Status: CLEAN. Zero safety guardrail violations detected.")
        else:
            print(f"🚨 Status: VIOLATIONS DETECTED ({len(violations)} hits):")
            for v in violations:
                print(f"  - [{v.severity.value.upper()}] {v.rule_id}: {v.description} (Snippet: {v.matched_snippet})")

    elif args.command == "benchmark":
        runner = EvalRunner()
        sample_schema = {
            "type": "object",
            "properties": {
                "name": {"type": "string", "minLength": 2},
                "age": {"type": "integer", "minimum": 0, "maximum": 150},
            },
            "required": ["name", "age"],
        }
        test_cases = [
            {"id": "valid_user", "payload": {"name": "Alice", "age": 28}},
            {"id": "missing_age", "payload": {"name": "Bob"}},
            {"id": "negative_age", "payload": {"name": "Charlie", "age": -5}},
            {"id": "leaked_secret", "payload": {"name": "Dave", "age": 30, "key": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.xyz"}},
            {"id": "rm_command", "payload": {"name": "Eve", "age": 22, "cmd": "rm -rf /"}},
        ]
        suite_res = runner.run_suite("agent_user_tool_benchmark", test_cases, schema=sample_schema)
        print(suite_res.to_markdown_report())


if __name__ == "__main__":
    main()
