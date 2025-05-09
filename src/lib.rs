use rhai::Engine;

pub mod dsl;

use dsl::types::ValidationDiagnostic;
use dsl::validation;

pub mod validators;

pub fn validate(
    json_check: &serde_json::Value,
    check_id: &str,
    engine: &Engine,
) -> Result<(), Vec<ValidationDiagnostic>> {
    let json_schema = validation::get_json_schema();

    validation::validate(&json_check, &check_id, &json_schema, &engine)
}
