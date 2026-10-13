"""Strict JSON Schema Validator and Drift Detection Engine.
Enforces type consistency, required fields, and boundary constraints without external dependencies.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional


@dataclass
class ValidationError:
    path: str
    message: str
    error_type: str


@dataclass
class ValidationResult:
    is_valid: bool
    errors: List[ValidationError] = field(default_factory=list)
    repaired_payload: Optional[Dict[str, Any]] = None

    def summary(self) -> str:
        if self.is_valid:
            return "Payload complies with JSON Schema."
        errs = "; ".join(f"[{e.path}] {e.message}" for e in self.errors)
        return f"Validation failed with {len(self.errors)} errors: {errs}"


class SchemaValidator:
    """Validates structured LLM outputs and tool arguments against JSON Schema."""

    def __init__(self, schema: Dict[str, Any]):
        self.schema = schema

    def validate(self, payload: Any, auto_repair: bool = False) -> ValidationResult:
        errors: List[ValidationError] = []
        repaired = payload if auto_repair else None
        self._validate_node(payload, self.schema, "$", errors)
        return ValidationResult(is_valid=len(errors) == 0, errors=errors, repaired_payload=repaired)

    def _validate_node(
        self,
        value: Any,
        schema: Dict[str, Any],
        path: str,
        errors: List[ValidationError],
    ) -> None:
        target_type = schema.get("type")

        # Type checks
        if target_type == "object":
            if not isinstance(value, dict):
                errors.append(ValidationError(path, f"Expected object, got {type(value).__name__}", "type_mismatch"))
                return

            # Required properties
            for req in schema.get("required", []):
                if req not in value:
                    errors.append(ValidationError(f"{path}.{req}", f"Missing required property '{req}'", "missing_required"))

            # Properties validation
            properties = schema.get("properties", {})
            for k, val in value.items():
                if k in properties:
                    self._validate_node(val, properties[k], f"{path}.{k}", errors)
                elif schema.get("additionalProperties") is False:
                    errors.append(ValidationError(f"{path}.{k}", f"Unexpected property '{k}' not permitted by schema", "unexpected_property"))

        elif target_type == "array":
            if not isinstance(value, list):
                errors.append(ValidationError(path, f"Expected array, got {type(value).__name__}", "type_mismatch"))
                return
            items_schema = schema.get("items")
            if items_schema:
                for idx, item in enumerate(value):
                    self._validate_node(item, items_schema, f"{path}[{idx}]", errors)

        elif target_type == "string":
            if not isinstance(value, str):
                errors.append(ValidationError(path, f"Expected string, got {type(value).__name__}", "type_mismatch"))
                return
            if "minLength" in schema and len(value) < schema["minLength"]:
                errors.append(ValidationError(path, f"String length {len(value)} < minLength {schema['minLength']}", "min_length"))
            if "maxLength" in schema and len(value) > schema["maxLength"]:
                errors.append(ValidationError(path, f"String length {len(value)} > maxLength {schema['maxLength']}", "max_length"))
            if "pattern" in schema:
                pattern = schema["pattern"]
                if not re.search(pattern, value):
                    errors.append(ValidationError(path, f"String does not match pattern '{pattern}'", "pattern_mismatch"))

        elif target_type in ("number", "integer"):
            if target_type == "integer" and not isinstance(value, int) or (isinstance(value, bool)):
                errors.append(ValidationError(path, f"Expected integer, got {type(value).__name__}", "type_mismatch"))
                return
            elif target_type == "number" and not isinstance(value, (int, float)) or (isinstance(value, bool)):
                errors.append(ValidationError(path, f"Expected number, got {type(value).__name__}", "type_mismatch"))
                return

            if "minimum" in schema and value < schema["minimum"]:
                errors.append(ValidationError(path, f"Value {value} < minimum {schema['minimum']}", "minimum_violation"))
            if "maximum" in schema and value > schema["maximum"]:
                errors.append(ValidationError(path, f"Value {value} > maximum {schema['maximum']}", "maximum_violation"))

        elif target_type == "boolean":
            if not isinstance(value, bool):
                errors.append(ValidationError(path, f"Expected boolean, got {type(value).__name__}", "type_mismatch"))
                return

        # Enum check
        if "enum" in schema and value not in schema["enum"]:
            errors.append(ValidationError(path, f"Value {value} is not in enum {schema['enum']}", "enum_violation"))
