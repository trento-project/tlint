#![deny(clippy::pedantic)]

use clap::{Parser, Subcommand, ValueEnum};
use rhai::Engine;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::process;

pub mod dsl;

use dsl::display;
use dsl::types::{Check, ValidationDiagnostic};
use dsl::validation::{self, EnabledValidator};

pub mod validators;

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum ArgValidator {
    All,
    Expectation,
    Link,
    Schema,
    Value,
}

impl Into<Option<EnabledValidator>> for ArgValidator {
    fn into(self) -> Option<EnabledValidator> {
        match self {
            ArgValidator::All => None,
            ArgValidator::Expectation => Some(EnabledValidator::Expectation),
            ArgValidator::Link => Some(EnabledValidator::Link),
            ArgValidator::Schema => Some(EnabledValidator::Schema),
            ArgValidator::Value => Some(EnabledValidator::Value),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Lint {
        file: Option<String>,

        #[clap(long, value_enum, default_value("all"))]
        rule: Vec<ArgValidator>,
    },
    Show {
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

fn is_directory(arg_path: Option<String>) -> bool {
    match arg_path {
        Some(path) => Path::new(&path).is_dir(),
        None => false,
    }
}

fn scan_directory(directory: &str) -> Result<Vec<String>, std::io::Error> {
    let files_list = fs::read_dir(directory)?
        .filter_map(|file| {
            file.ok().and_then(|e| match e.path().is_file() {
                true => e.path().to_str().map(|s| s.to_string()),
                false => None,
            })
        })
        .collect();
    Ok(files_list)
}

fn normalize_rules(rules: Vec<ArgValidator>) -> Vec<EnabledValidator> {
    if rules.contains(&ArgValidator::All) {
        vec![
            EnabledValidator::Expectation,
            EnabledValidator::Link,
            EnabledValidator::Schema,
            EnabledValidator::Value,
        ]
    } else {
        rules
            .iter()
            .filter_map(|val| Into::<Option<EnabledValidator>>::into(*val))
            .collect()
    }
}

fn main() -> Result<(), serde_yaml::Error> {
    let args = Args::parse();
    let engine = Engine::new();

    match args.command {
        Commands::Lint { file, rule } => match is_directory(file.clone()) {
            true => {
                if let Some(directory) = file {
                    let json_schema = validation::get_json_schema();
                    let files = scan_directory(&directory).expect("Unable to scan directory");
                    let mut parsing_errors = vec![];
                    let (_, validation_errors): (Vec<_>, Vec<_>) = files
                        .into_iter()
                        .filter(|check_path| {
                            let extension = Path::new(check_path).extension();
                            match extension {
                                Some(s) => s == "yml" || s == "yaml",
                                None => false,
                            }
                        })
                        .map(|check_path| {
                            let input = get_input(Some(check_path));
                            let json_value: serde_json::Value = serde_yaml::from_str(&input)
                                .expect("Unable to parse the YAML into a JSON payload");
                            let deserialization_result = serde_yaml::from_str::<Check>(&input);

                            match deserialization_result {
                                Err(ref error) => {
                                    parsing_errors.push(error.to_string());
                                    Ok(())
                                }
                                Ok(check) => {
                                    let check_id = check.id;
                                    let normalized_rules = normalize_rules(rule.clone());

                                    validation::validate(
                                        &json_value,
                                        &check_id,
                                        &json_schema,
                                        &engine,
                                        &normalized_rules,
                                    )
                                }
                            }
                        })
                        .partition(Result::is_ok);

                    let exit_code = match parsing_errors.is_empty() && validation_errors.is_empty()
                    {
                        true => 0,
                        false => 1,
                    };

                    for error in parsing_errors {
                        println!("{} - {}", validation::error_header("Parse error"), error);
                    }

                    validation_errors
                        .into_iter()
                        .flat_map(Result::unwrap_err)
                        .for_each(|diagnostic| match diagnostic {
                            ValidationDiagnostic::Warning {
                                check_id,
                                message,
                                instance_path,
                            } => {
                                println!("{} - {}", validation::warning_header(&check_id), message);
                                println!("  path: {instance_path}\n");
                            }
                            ValidationDiagnostic::Critical {
                                check_id,
                                message,
                                instance_path,
                            } => {
                                println!("{} - {}", validation::error_header(&check_id), message);
                                println!("  path: {instance_path}\n");
                            }
                        });

                    process::exit(exit_code);
                }
            }
            false => {
                let input = get_input(file);
                let json_value: serde_json::Value = serde_yaml::from_str(&input)?;
                let deserialization_result = serde_yaml::from_str::<Check>(&input);

                if let Err(ref error) = deserialization_result {
                    println!("{} - {}", validation::error_header("Parse error"), error);
                    process::exit(1)
                }

                let check = deserialization_result.unwrap();
                let check_id = check.id;
                let json_schema = validation::get_json_schema();
                let normalized_rules = normalize_rules(rule);
                let validation_result =
                    validation::validate(&json_value, &check_id, &json_schema, &engine, &normalized_rules);

                let exit_code = match validation_result {
                    Ok(_) => 0,
                    Err(validation_errors) => {
                        validation_errors
                            .iter()
                            .for_each(|diagnostic| match diagnostic {
                                ValidationDiagnostic::Warning {
                                    check_id,
                                    message,
                                    instance_path,
                                } => {
                                    println!(
                                        "{} - {}",
                                        validation::warning_header(&check_id),
                                        message
                                    );
                                    println!("  path: {instance_path}\n");
                                }
                                ValidationDiagnostic::Critical {
                                    check_id,
                                    message,
                                    instance_path,
                                } => {
                                    println!(
                                        "{} - {}",
                                        validation::error_header(&check_id),
                                        message
                                    );
                                    println!("  path: {instance_path}\n");
                                }
                            });
                        1
                    }
                };

                process::exit(exit_code);
            }
        },

        Commands::Show { file } => {
            let input = get_input(file);

            let check: Check = serde_yaml::from_str(&input)?;

            display::print_check(check);
        }
    }

    Ok(())
}
