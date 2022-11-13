extern crate corosync_config_parser;

use clap::{Parser, Subcommand};
use jsonschema::{Draft, JSONSchema};
use serde_json::json;
use serde_yaml;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

pub mod dsl;

use dsl::display;
use dsl::parsing;
use dsl::types::{Check, ValidationError};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Lint {
        #[clap(short, long, value_parser)]
        file: Option<String>,
    },
    Show {
        #[clap(short, long, value_parser)]
        file: Option<String>,
    },
}

fn get_input(file: Option<String>) -> String {
    let mut payload = String::new();
    match file {
        Some(file_path) => {
            let mut file = File::open(file_path).expect("Unable to open file");
            file.read_to_string(&mut payload).expect("");
        }
        None => {
            io::stdin()
                .read_to_string(&mut payload)
                .expect("Unable to read from stdin");
        }
    }
    payload
}

fn validate(json_check: &serde_json::Value, check_id: &str) -> Result<(), Vec<ValidationError>> {
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

fn main() -> Result<(), serde_yaml::Error> {
    let args = Args::parse();

    match args.command {
        Commands::Lint { file } => {
            let input = get_input(file);

            let json_value: serde_json::Value = serde_yaml::from_str(&input)?;

            let check: Check = serde_yaml::from_str(&input)?;

            let check_id = check.id;

            println!("{}", serde_json::to_string_pretty(&json_value).unwrap());

            let validation_result = validate(&json_value, &check_id);

            process::exit(0);
        }

        Commands::Show { file } => {
            let input = get_input(file);
            let yaml_documents = parsing::string_to_yaml(input);
            let (checks, _) = parsing::parse_checks(&yaml_documents[0]);

            checks.into_iter().for_each(|check| {
                display::print_check(check);
            })
        }
    }

    Ok(())
}
