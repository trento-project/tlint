// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use rhai::Engine;
use tlint::validate;
use tlint::dsl::types::{Check, ValidationDiagnostic};

#[derive(Serialize, Deserialize)]
struct ValidationResult {
    pub result: bool,
    pub messages: Vec<String>
}

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn lint(content: String) -> JsValue {
    let engine = Engine::new_raw();

    let json_value: serde_json::Value = match serde_yaml::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            let r = ValidationResult {
                result: false,
                messages: vec![error.to_string()],
            };
            return serde_wasm_bindgen::to_value(&r).unwrap();
        }
    };
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
                    .map(|diagnostic| {
                        match diagnostic {
                            ValidationDiagnostic::Warning { message, instance_path, ..} => format!("{} - path: {}", message, instance_path),
                            ValidationDiagnostic::Critical { message, instance_path, ..} => format!("{} - path: {}", message, instance_path),
                        }
                    })
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
