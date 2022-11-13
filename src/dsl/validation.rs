use colored::*;
use jsonschema::{Draft, JSONSchema};
use serde_json::json;

use crate::dsl::types::ValidationError;

pub trait Validate {
    fn validate(&self) -> Result<(), Vec<String>>;
}

pub fn error_header(head: &str) -> String {
    format!("  {}  ", head).on_red().black().to_string()
}

pub fn string_not_empty(operand: &str, prefix: &str) -> Result<(), String> {
    match operand.is_empty() {
        true => Err(format!("{} - String must not be empty", prefix)),
        false => Ok(()),
    }
}

pub fn list_not_empty<T>(operand: &[T], prefix: &str) -> Result<(), String> {
    match operand.is_empty() {
        true => Err(format!("{} - List must not be empty", prefix)),
        false => Ok(()),
    }
}

pub fn validate(
    json_check: &serde_json::Value,
    check_id: &str,
) -> Result<(), Vec<ValidationError>> {
    let schema = json!(
        {
            "$schema": "http://json-schema.org/draft-06/schema#",
            "$ref": "#/definitions/Welcome4",
            "definitions": {
                "Welcome4": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "description": {
                            "type": "string"
                        },
                        "expectations": {
                            "type": "array",
                            "items": {
                                "$ref": "#/definitions/Expectation"
                            }
                        },
                        "facts": {
                            "type": "array",
                            "items": {
                                "$ref": "#/definitions/Fact"
                            }
                        },
                        "group": {
                            "type": "string"
                        },
                        "id": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "remediation": {
                            "type": "string"
                        },
                        "values": {
                            "type": "array",
                            "items": {
                                "$ref": "#/definitions/Value"
                            }
                        }
                    },
                    "required": [
                        "description",
                        "expectations",
                        "facts",
                        "group",
                        "id",
                        "name",
                        "remediation",
                        "values"
                    ],
                    "title": "Welcome4"
                },
                "Expectation": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "expect": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "required": [
                        "expect",
                        "name"
                    ],
                    "title": "Expectation"
                },
                "Fact": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "argument": {
                            "type": "string"
                        },
                        "gatherer": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "required": [
                        "argument",
                        "gatherer",
                        "name"
                    ],
                    "title": "Fact"
                },
                "Value": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "conditions": {
                            "type": "array",
                            "items": {
                                "$ref": "#/definitions/Condition"
                            }
                        },
                        "default": {
                            "type": "integer"
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "required": [
                        "conditions",
                        "default",
                        "name"
                    ],
                    "title": "Value"
                },
                "Condition": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "value": {
                            "type": "integer"
                        },
                        "when": {
                            "type": "string"
                        }
                    },
                    "required": [
                        "value",
                        "when"
                    ],
                    "title": "Condition"
                }
            }
        }
    );

    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("A valid schema");

    let validation_result = match compiled_schema.validate(json_check) {
        Ok(_) => Ok(()),
        Err(errors) => {
            let validation_errors = errors
                .map(|error| ValidationError {
                    check_id: check_id.to_string(),
                    error: error.to_string(),
                    instance_path: error.instance_path.to_string(),
                })
                .collect();
            Err(validation_errors)
        }
    };

    validation_result
}
