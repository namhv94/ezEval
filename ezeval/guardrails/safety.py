"""Safety Guardrails and Policy Verification.
Scans LLM tool calls and outputs for PII leakage, credential leaks, and hazardous commands.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from enum import Enum
from typing import List, Optional


class ViolationSeverity(str, Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


@dataclass
class GuardrailViolation:
    rule_id: str
    description: str
    severity: ViolationSeverity
    matched_snippet: str


class SafetyGuardrail:
    """Detects credential leaks, hazardous shell commands, and PII in agent telemetry."""

    PATTERNS = {
        # Secrets & API Keys
        "secret_openai_key": (re.compile(r"sk-[a-zA-Z0-9]{20,T3BlbkFJ[a-zA-Z0-9]{20,}"), ViolationSeverity.CRITICAL, "OpenAI API Key detected"),
        "secret_generic_bearer": (re.compile(r"Bearer\s+[a-zA-Z0-9_\-\.]{25,}"), ViolationSeverity.HIGH, "Generic Bearer Token detected"),
        "secret_aws_key": (re.compile(r"AKIA[0-9A-Z]{16}"), ViolationSeverity.CRITICAL, "AWS Access Key ID detected"),
        "secret_github_pat": (re.compile(r"ghp_[0-9a-zA-Z]{36}"), ViolationSeverity.CRITICAL, "GitHub Personal Access Token detected"),

        # PII Detection
        "pii_email": (re.compile(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+"), ViolationSeverity.MEDIUM, "Personal Email Address detected"),
        "pii_phone_vn": (re.compile(r"(?:\+84|0)(?:3[2-9]|5[6|8|9]|7[0|6-9]|8[1-5]|9[0-9])[0-9]{7}"), ViolationSeverity.MEDIUM, "Vietnam Phone Number detected"),
        "pii_credit_card": (re.compile(r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13})\b"), ViolationSeverity.CRITICAL, "Credit Card number format detected"),

        # Hazardous OS Commands
        "hazard_destructive_rm": (re.compile(r"rm\s+-(?:rf|fr)\s+(?:/|\.\.|~|\*)"), ViolationSeverity.CRITICAL, "Destructive root filesystem removal command"),
        "hazard_curl_pipe_sh": (re.compile(r"(?:curl|wget)\s+.*\|\s*(?:sh|bash|zsh)"), ViolationSeverity.HIGH, "Arbitrary remote code execution via curl/pipe bash"),
        "hazard_fork_bomb": (re.compile(r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:"), ViolationSeverity.CRITICAL, "Fork bomb command pattern"),
    }

    def inspect(self, text: str) -> List[GuardrailViolation]:
        violations: List[GuardrailViolation] = []
        if not text:
            return violations

        for rule_id, (pattern, severity, desc) in self.PATTERNS.items():
            matches = pattern.findall(text)
            for m in matches:
                snippet = m if isinstance(m, str) else str(m)
                # Redact matched sensitive info
                masked = snippet[:4] + "*" * max(0, len(snippet) - 6) + snippet[-2:] if len(snippet) > 6 else "***"
                violations.append(
                    GuardrailViolation(
                        rule_id=rule_id,
                        description=desc,
                        severity=severity,
                        matched_snippet=masked,
                    )
                )
        return violations

    def is_safe(self, text: str, max_severity: ViolationSeverity = ViolationSeverity.HIGH) -> bool:
        violations = self.inspect(text)
        sev_rank = {
            ViolationSeverity.LOW: 1,
            ViolationSeverity.MEDIUM: 2,
            ViolationSeverity.HIGH: 3,
            ViolationSeverity.CRITICAL: 4,
        }
        threshold = sev_rank[max_severity]
        for v in violations:
            if sev_rank[v.severity] >= threshold:
                return False
        return True
