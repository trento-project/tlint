extern crate corosync_config_parser;

use clap::{Parser, Subcommand};
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

pub mod dsl;

use dsl::display;
use dsl::parsing;
use dsl::types::{Check, ValidationError};
use dsl::validation;

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

fn main() -> Result<(), serde_yaml::Error> {
    let args = Args::parse();

    match args.command {
        Commands::Lint { file } => {
            let input = get_input(file);

            let json_value: serde_json::Value = serde_yaml::from_str(&input)?;

            let check: Check = serde_yaml::from_str(&input)?;

            let check_id = check.id;

            let validation_result = validation::validate(&json_value, &check_id);

            let exit_code = match validation_result {
                Ok(_) => 0,
                Err(validation_errors) => {
                    validation_errors.iter().for_each(
                        |ValidationError {
                             check_id,
                             error,
                             instance_path,
                         }| {
                            println!("{} - {}", validation::error_header(&check_id), error);
                            println!("  path: {}\n", instance_path);
                        },
                    );
                    1
                }
            };

            process::exit(exit_code);
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
