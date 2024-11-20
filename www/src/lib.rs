use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use rhai::Engine;
use tlint::validate;
use tlint::dsl::types::{Check, ValidationError};

#[derive(Serialize, Deserialize)]
struct ValidationResult {
    pub result: bool,
    pub messages: Vec<String>
}

#[wasm_bindgen]
pub fn lint(content: String) -> JsValue {
    let engine = Engine::new_raw();

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

            let validation_errors = validate(
                &json_value,
                &check_id,
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