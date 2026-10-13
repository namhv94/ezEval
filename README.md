<div align="center">

# 🛡️ ezEval

**High-Throughput Boundary Fuzzing, Strict JSON Schema Enforcement & Safety Guardrails**  
*Hardening AI agents and LLM tool calling against schema drift, boundary overflows, and safety violations.*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Python Version](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/)
[![Safety Alignment](https://img.shields.io/badge/Safety-Anthropic%20Aligned-emerald.svg)](https://www.anthropic.com/)
[![Product of EzLabs](https://img.shields.io/badge/Product%20of-EzLabs-0ea5e9.svg)](https://ezlabs.tech)

[**Website**](https://ezlabs.tech) • [**ezRouter**](https://github.com/namhv94/ezRouter) • [**ezAgents**](https://github.com/namhv94/ezAgents) • [**Issues**](https://github.com/namhv94/ezEval/issues)

</div>

---

## 💡 The Problem

Autonomous AI systems fail catastrophically at boundary edges:
* **Silent Schema Drift**: Models emit invalid types (strings instead of integers, hallucinated fields) that crash downstream database queries or API webhooks.
* **Adversarial & Edge Payloads**: Unhandled null-bytes, extreme float magnitudes, or nested JSON bombs exhaust memory and trigger server crashes.
* **Silent Leakage**: Agents inadvertently regurgitate PII (emails, phone numbers, credit cards) or sensitive API keys into logs and responses.
* **Unsanitized Command Execution**: Agent tool arguments frequently slip into destructive shell commands (`rm -rf /`, `curl | bash`).

## ⚡ The Solution: ezEval

**ezEval** is a high-speed, zero-dependency evaluation toolkit developed by **[EzLabs](https://ezlabs.tech)** to fuzz, validate, and guard AI agents:
* **Boundary Fuzzer**: Synthesizes adversarial mutations (Unicode RTL, zero-width spaces, integer bounds, recursive JSON depth).
* **Strict JSON Schema Validator**: Detects missing fields, invalid ranges, and hallucinated keys in sub-millisecond time.
* **Safety & PII Guardrails**: Pre-execution and post-generation scanning for tokens, API keys, credentials, and hazardous commands.

---

## 🏛️ System Architecture

```text
 ┌────────────────────────────────────────────────────────┐
 │            LLM Structured Output / Tool Call           │
 └───────────────────────────┬────────────────────────────┘
                             │
                             ▼
 ┌────────────────────────────────────────────────────────┐
 │            ezEval Fast Interception Engine             │
 └───────────────────────────┬────────────────────────────┘
                             │
              ┌──────────────┴──────────────┐
              ▼                             ▼
   ┌───────────────────────┐   ┌──────────────────────────┐
   │  Strict Schema Engine │   │   Safety Guardrail Wall  │
   │  - Type & Range Check │   │   - PII Scrubber (VN/Int)│
   │  - Required Keys      │   │   - API Key & Token Leak │
   │  - Drift & Hallucinate│   │   - Dangerous Shell Cmds │
   └──────────┬────────────┘   └────────────┬─────────────┘
              │                             │
              └──────────────┬──────────────┘
                             ▼
 ┌────────────────────────────────────────────────────────┐
 │               Automated Evaluation Report              │
 │          Compliant ➔ Passed to Environment             │
 │          Violation ➔ Bounded Intercept & Quarantine    │
 └────────────────────────────────────────────────────────┘
```

---

## ✨ Key Features

- **Strict Schema Enforcement**: Validates agent arguments against JSON Schema with zero dependencies. Catches subtle type coercion bugs and schema drift.
- **Boundary & Adversarial Fuzzing**: Generates extreme numeric edges, null-byte payloads, RTL overrides, and nested recursion bombs.
- **Realtime Safety Guardrails**: Realtime detection of API keys (OpenAI, AWS, GitHub), Bearer tokens, PII (emails, Vietnamese & international phones), and hazardous commands (`rm -rf`, pipe-to-shell, fork bombs).
- **Sub-Millisecond Performance**: Pure Python engine evaluating hundreds of test cases per millisecond without heavy ML overhead.
- **Markdown & CI/CD Export**: Generates clean evaluation tables suitable for GitHub Actions, PR comments, and compliance audits.

---

## 🚀 Quick Start (60 Seconds)

### 1. Installation

```bash
git clone https://github.com/namhv94/ezEval.git
cd ezEval
pip install -e .
```

### 2. Scan Text / Arguments for Safety Violations

```bash
ezeval check "Deploy with token and contact 0912345678"
```

*Output:*
```text
=== ezEval Safety Scan (v0.1.0) ===
Target: Deploy with token and contact 0912345678...
🚨 Status: VIOLATIONS DETECTED:
  - [MEDIUM] pii_phone_vn: Vietnam Phone Number detected (Snippet: 0912****78)
```

### 3. Generate Boundary Fuzz Payloads

```bash
ezeval fuzz --category unicode
```

### 4. Run an Automated Benchmark Suite

```bash
ezeval benchmark
```

---

## 💻 Python API Usage

### Schema Validation & Drift Detection

```python
from ezeval import SchemaValidator

schema = {
    "type": "object",
    "properties": {
        "user_id": {"type": "integer", "minimum": 1},
        "role": {"type": "string", "enum": ["admin", "editor", "viewer"]},
    },
    "required": ["user_id", "role"],
    "additionalProperties": False,
}

validator = SchemaValidator(schema)
res = validator.validate({"user_id": 42, "role": "admin"})

print(f"Valid: {res.is_valid}")  # True
```

### Safety Guardrail Inspection

```python
from ezeval import SafetyGuardrail, ViolationSeverity

guard = SafetyGuardrail()
violations = guard.inspect("Please run rm -rf / to clear logs")

for v in violations:
    print(f"[{v.severity.value}] {v.rule_id}: {v.description}")
```

### Automated Benchmark Runner

```python
from ezeval import EvalRunner

runner = EvalRunner()
test_cases = [
    {"id": "test_normal", "payload": {"user_id": 10, "role": "editor"}},
    {"id": "test_leak", "payload": {"user_id": 11, "role": "admin", "token": "Bearer secret_abc_xyz_1234567890"}},
]

results = runner.run_suite("agent_auth_suite", test_cases, schema=schema)
print(results.to_markdown_report())
```

---

## 🧪 Running Tests

```bash
python3 -m unittest discover tests
```

Output:
```text
.........
----------------------------------------------------------------------
Ran 9 tests in 0.001s

OK
```

---

## 🌐 EzLabs Ecosystem

| Project | Description | Status |
| :--- | :--- | :--- |
| **[ezRouter](https://github.com/namhv94/ezRouter)** | Ultra-fast LLM routing proxy in Rust with Token Saver & multi-provider failover | `v0.1.0` Open Source |
| **[ezAgents](https://github.com/namhv94/ezAgents)** | MCP-native Agent Orchestration Engine with deterministic verifiers | `v0.1.0` Active |
| **[ezEval](https://github.com/namhv94/ezEval)** | Boundary fuzzing, JSON Schema validation & safety guardrails | `v0.1.0` Active |

---

## 📄 License

Distributed under the MIT License. See [LICENSE](LICENSE) for more information.

Developed with ❤️ by **[EzLabs](https://ezlabs.tech)**.
