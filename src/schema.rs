use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
    pub error_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn summary(&self) -> String {
        if self.is_valid {
            "Payload complies with JSON Schema.".to_string()
        } else {
            let errs: Vec<String> = self
                .errors
                .iter()
                .map(|e| format!("[{}] {}", e.path, e.message))
                .collect();
            format!(
                "Validation failed with {} errors: {}",
                self.errors.len(),
                errs.join("; ")
            )
        }
    }
}

pub struct SchemaValidator {
    schema: Value,
}

impl SchemaValidator {
    pub fn new(schema: Value) -> Self {
        Self { schema }
    }

    pub fn validate(&self, payload: &Value) -> ValidationResult {
        let mut errors = Vec::new();
        self.validate_node(payload, &self.schema, "$", &mut errors);
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    fn validate_node(
        &self,
        value: &Value,
        schema: &Value,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let target_type = schema.get("type").and_then(|t| t.as_str());

        match target_type {
            Some("object") => {
                let obj = match value.as_object() {
                    Some(o) => o,
                    None => {
                        errors.push(ValidationError {
                            path: path.to_string(),
                            message: format!("Expected object, got {}", value),
                            error_type: "type_mismatch".to_string(),
                        });
                        return;
                    }
                };

                // Check required fields
                if let Some(req_arr) = schema.get("required").and_then(|r| r.as_array()) {
                    for req in req_arr {
                        if let Some(req_str) = req.as_str() {
                            if !obj.contains_key(req_str) {
                                errors.push(ValidationError {
                                    path: format!("{}.{}", path, req_str),
                                    message: format!("Missing required property '{}'", req_str),
                                    error_type: "missing_required".to_string(),
                                });
                            }
                        }
                    }
                }

                // Check properties
                let properties = schema.get("properties").and_then(|p| p.as_object());
                for (k, val) in obj {
                    if let Some(props) = properties {
                        if let Some(prop_schema) = props.get(k) {
                            self.validate_node(
                                val,
                                prop_schema,
                                &format!("{}.{}", path, k),
                                errors,
                            );
                            continue;
                        }
                    }
                    if schema.get("additionalProperties").and_then(|a| a.as_bool()) == Some(false) {
                        errors.push(ValidationError {
                            path: format!("{}.{}", path, k),
                            message: format!("Unexpected property '{}' not permitted by schema", k),
                            error_type: "unexpected_property".to_string(),
                        });
                    }
                }
            }
            Some("string") => {
                let s = match value.as_str() {
                    Some(s) => s,
                    None => {
                        errors.push(ValidationError {
                            path: path.to_string(),
                            message: format!("Expected string, got {}", value),
                            error_type: "type_mismatch".to_string(),
                        });
                        return;
                    }
                };

                if let Some(min_len) = schema.get("minLength").and_then(|m| m.as_u64()) {
                    if s.chars().count() < min_len as usize {
                        errors.push(ValidationError {
                            path: path.to_string(),
                            message: format!(
                                "String length {} < minLength {}",
                                s.chars().count(),
                                min_len
                            ),
                            error_type: "min_length".to_string(),
                        });
                    }
                }

                if let Some(max_len) = schema.get("maxLength").and_then(|m| m.as_u64()) {
                    if s.chars().count() > max_len as usize {
                        errors.push(ValidationError {
                            path: path.to_string(),
                            message: format!(
                                "String length {} > maxLength {}",
                                s.chars().count(),
                                max_len
                            ),
                            error_type: "max_length".to_string(),
                        });
                    }
                }
            }
            Some("integer") => {
                let num = match value.as_i64() {
                    Some(n) => n,
                    None => {
                        errors.push(ValidationError {
                            path: path.to_string(),
                            message: format!("Expected integer, got {}", value),
                            error_type: "type_mismatch".to_string(),
                        });
                        return;
                    }
                };

                if let Some(min_val) = schema.get("minimum").and_then(|m| m.as_i64()) {
                    if num < min_val {
                        errors.push(ValidationError {
                            path: path.to_string(),
                            message: format!("Value {} < minimum {}", num, min_val),
                            error_type: "minimum_violation".to_string(),
                        });
                    }
                }

                if let Some(max_val) = schema.get("maximum").and_then(|m| m.as_i64()) {
                    if num > max_val {
                        errors.push(ValidationError {
                            path: path.to_string(),
                            message: format!("Value {} > maximum {}", num, max_val),
                            error_type: "maximum_violation".to_string(),
                        });
                    }
                }
            }
            Some("boolean") if !value.is_boolean() => {
                errors.push(ValidationError {
                    path: path.to_string(),
                    message: format!("Expected boolean, got {}", value),
                    error_type: "type_mismatch".to_string(),
                });
            }
            _ => {}
        }

        // Enum validation
        if let Some(enum_vals) = schema.get("enum").and_then(|e| e.as_array()) {
            if !enum_vals.contains(value) {
                errors.push(ValidationError {
                    path: path.to_string(),
                    message: format!("Value {} is not in enum {:?}", value, enum_vals),
                    error_type: "enum_violation".to_string(),
                });
            }
        }
    }
}
