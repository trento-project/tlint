use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use rhai::Engine;

pub mod dsl;

use dsl::types::{Check, ValidationError};
use dsl::validation;

pub mod validators;

#[derive(Serialize, Deserialize)]
pub struct ValidationResult {
    pub result: bool,
    pub messages: Vec<String>
}

#[wasm_bindgen]
pub fn lint(content: String) -> JsValue {
    let engine = Engine::new_raw();
    let json_schema = validation::get_json_schema();

    let json_value: serde_json::Value = serde_yaml::from_str(&content)
        .expect("Unable to parse the YAML into a JSON payload");
    let deserialization_result = serde_yaml::from_str::<Check>(&content);

    let r = match deserialization_result {
        Err(ref error) => {
            ValidationResult {
                result: false,
                messages: vec![error.to_string()]
            }
        }
        Ok(check) => {
            let check_id = check.id;

            let validation_errors = validation::validate(
                &json_value,
                &check_id,
                &json_schema,
                &engine,
            );

            let messages = match validation_errors {
                Err(ref errors) => {
                    errors
                    .into_iter()
                    .map(|ValidationError { check_id: _, error, instance_path }| 
                        format!("{} - path: {}", error, instance_path)
                    )
                    .collect()
                }
                Ok(()) => {                    
                    vec![String::from("Ok!")]
                }
            };

            ValidationResult {
                result: validation_errors.is_ok(),
                messages: messages
            }
        }
    };

    serde_wasm_bindgen::to_value(&r).unwrap()
}