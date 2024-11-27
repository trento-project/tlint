use rhai::Engine;

pub mod dsl;

use dsl::types::ValidationError;
use dsl::validation;

pub mod validators;

pub fn validate(
    json_check: &serde_json::Value,
    check_id: &str,
    engine: &Engine,
) -> Result<(), Vec<ValidationError>> {
    let json_schema = validation::get_json_schema();

    validation::validate(
        &json_check,
        &check_id,
        &json_schema,
        &engine,
    )  
}