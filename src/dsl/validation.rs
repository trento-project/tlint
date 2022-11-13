use super::types::ValidationError;
use colored::*;
use jsonschema::{Draft, JSONSchema};

const SCHEMA: &str = include_str!("../../schema.json");

pub fn error_header(head: &str) -> String {
    format!("  {}  ", head).on_red().black().to_string()
}

pub fn validate(
    json_check: &serde_json::Value,
    check_id: &str,
    schema: &JSONSchema,
) -> Result<(), Vec<ValidationError>> {
    let validation_result = match schema.validate(json_check) {
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

pub fn get_json_schema() -> JSONSchema {
    let value = serde_json::from_str(&SCHEMA).unwrap();

    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&value)
        .expect("A valid schema");

    compiled_schema
}
