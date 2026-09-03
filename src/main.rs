// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

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
    Exclude,
}

impl From<ArgValidator> for Option<EnabledValidator> {
    fn from(val: ArgValidator) -> Self {
        match val {
            ArgValidator::All => None,
            ArgValidator::Expectation => Some(EnabledValidator::Expectation),
            ArgValidator::Link => Some(EnabledValidator::Link),
            ArgValidator::Schema => Some(EnabledValidator::Schema),
            ArgValidator::Value => Some(EnabledValidator::Value),
            ArgValidator::Exclude => Some(EnabledValidator::Exclude),
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
            let mut file = File::open(&file_path).unwrap_or_else(|err| {
                let reason = if err.kind() == io::ErrorKind::NotFound {
                    "No such file or directory".to_string()
                } else {
                    err.to_string()
                };
                eprintln!("Unable to open file '{file_path}': {reason}");
                process::exit(1);
            });
            file.read_to_string(&mut payload).unwrap_or_else(|err| {
                eprintln!("Unable to read file '{file_path}': {err}");
                process::exit(1);
            });
        }
        None => {
            io::stdin().read_to_string(&mut payload).unwrap_or_else(|err| {
                eprintln!("Unable to read from stdin: {err}");
                process::exit(1);
            });
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
            file.ok().and_then(|e| if e.path().is_file() { e.path().to_str().map(std::string::ToString::to_string) } else { None })
        })
        .collect();
    Ok(files_list)
}

fn normalize_rules(rules: &[ArgValidator]) -> Vec<EnabledValidator> {
    if rules.contains(&ArgValidator::All) {
        vec![
            EnabledValidator::Expectation,
            EnabledValidator::Link,
            EnabledValidator::Schema,
            EnabledValidator::Value,
            EnabledValidator::Exclude,
        ]
    } else {
        rules
            .iter()
            .filter_map(|val| Into::<Option<EnabledValidator>>::into(*val))
            .collect()
    }
}

fn with_path(label: &str, check_path: Option<&str>) -> String {
    match check_path {
        Some(path) => format!("{label} ({path})"),
        None => label.to_string(),
    }
}

fn diagnostic_header(check_id: &str, check_path: Option<&str>) -> String {
    with_path(check_id, check_path)
}

fn extract_check_id(json_value: &serde_json::Value) -> Option<&str> {
    json_value.get("id").and_then(serde_json::Value::as_str)
}

fn parse_error_header(check_id: Option<&str>, check_path: Option<&str>) -> String {
    let label = match check_id {
        Some(id) => format!("Parse error {id}"),
        None => "Parse error".to_string(),
    };
    with_path(&label, check_path)
}

fn print_diagnostic(diagnostic: &ValidationDiagnostic, check_path: Option<&str>) {
    match diagnostic {
        ValidationDiagnostic::Warning {
            check_id,
            message,
            instance_path,
        } => {
            let header = diagnostic_header(check_id, check_path);
            println!("{} - {message}", validation::warning_header(&header));
            println!("{}\n", validation::instance_path_line(instance_path));
        }
        ValidationDiagnostic::Critical {
            check_id,
            message,
            instance_path,
        } => {
            let header = diagnostic_header(check_id, check_path);
            println!("{} - {message}", validation::error_header(&header));
            println!("{}\n", validation::instance_path_line(instance_path));
        }
    }
}

fn lint_directory(directory: &str, rule: &[ArgValidator], engine: &Engine) -> i32 {
    let files = match scan_directory(directory) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("Unable to scan directory '{directory}': {error}");
            return 1;
        }
    };
    let json_schema = validation::get_json_schema();
    let mut parsing_errors: Vec<(String, Option<String>, String)> = vec![];
    let mut diagnostics_by_file: Vec<(String, Vec<ValidationDiagnostic>)> = vec![];

    files
        .into_iter()
        .filter(|check_path| {
            let extension = Path::new(check_path).extension();
            match extension {
                Some(s) => s == "yml" || s == "yaml",
                None => false,
            }
        })
        .for_each(|check_path| {
            let input = get_input(Some(check_path.clone()));
            let json_value: serde_json::Value = match serde_yaml::from_str(&input) {
                Ok(value) => value,
                Err(error) => {
                    parsing_errors.push((check_path, None, error.to_string()));
                    return;
                }
            };
            let deserialization_result = serde_yaml::from_str::<Check>(&input);

            match deserialization_result {
                Err(ref error) => {
                    let check_id = extract_check_id(&json_value).map(String::from);
                    parsing_errors.push((check_path, check_id, error.to_string()));
                }
                Ok(check) => {
                    let check_id = check.id;
                    let normalized_rules = normalize_rules(rule);

                    if let Err(diagnostics) = validation::validate(
                        &json_value,
                        &check_id,
                        &json_schema,
                        engine,
                        &normalized_rules,
                    ) {
                        diagnostics_by_file.push((check_path, diagnostics));
                    }
                }
            }
        });

    let exit_code = i32::from(!(parsing_errors.is_empty() && diagnostics_by_file.is_empty()));

    for (check_path, check_id, error) in parsing_errors {
        println!(
            "{} - {error}",
            validation::error_header(&parse_error_header(check_id.as_deref(), Some(&check_path)))
        );
    }

    for (check_path, diagnostics) in diagnostics_by_file {
        for diagnostic in diagnostics {
            print_diagnostic(&diagnostic, Some(&check_path));
        }
    }

    exit_code
}

fn lint_file(file: Option<String>, rule: &[ArgValidator], engine: &Engine) -> i32 {
    let file_path = file.clone();
    let input = get_input(file);
    let json_value: serde_json::Value = match serde_yaml::from_str(&input) {
        Ok(value) => value,
        Err(error) => {
            let header = parse_error_header(None, file_path.as_deref());
            println!("{} - {error}", validation::error_header(&header));
            return 1;
        }
    };
    let deserialization_result = serde_yaml::from_str::<Check>(&input);

    if let Err(ref error) = deserialization_result {
        let check_id = extract_check_id(&json_value);
        let header = parse_error_header(check_id, file_path.as_deref());
        println!("{} - {error}", validation::error_header(&header));
        return 1;
    }

    let check = deserialization_result.unwrap();
    let check_id = check.id;
    let json_schema = validation::get_json_schema();
    let normalized_rules = normalize_rules(rule);
    let validation_result =
        validation::validate(&json_value, &check_id, &json_schema, engine, &normalized_rules);

    match validation_result {
        Ok(()) => 0,
        Err(validation_errors) => {
            for diagnostic in &validation_errors {
                print_diagnostic(diagnostic, file_path.as_deref());
            }
            1
        }
    }
}

fn main() -> Result<(), serde_yaml::Error> {
    let args = Args::parse();
    let engine = Engine::new();

    match args.command {
        Commands::Lint { file, rule } => {
            let exit_code = if is_directory(file.clone()) {
                file.map_or(0, |directory| lint_directory(&directory, &rule, &engine))
            } else {
                lint_file(file, &rule, &engine)
            };
            process::exit(exit_code);
        }

        Commands::Show { file } => {
            let input = get_input(file);

            let check: Check = serde_yaml::from_str(&input)?;

            display::print_check(check);
        }
    }

    Ok(())
}
