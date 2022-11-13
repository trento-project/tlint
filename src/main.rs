extern crate corosync_config_parser;

use clap::{Parser, Subcommand};
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

pub mod dsl;

use dsl::display;
use dsl::parsing;
use dsl::types::ParsingError;
use dsl::validation;
use dsl::validation::Validate;

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

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Lint { file } => {
            let input = get_input(file);

            let yaml_documents = parsing::string_to_yaml(input);

            let (checks, parsing_errors) = parsing::parse_checks(&yaml_documents[0]);

            let (_, validation_errors): (Vec<_>, Vec<_>) = checks
                .into_iter()
                .map(|check| check.validate())
                .partition(Result::is_ok);

            let exit_code = match parsing_errors.is_empty() && validation_errors.is_empty() {
                true => 0,
                false => 1,
            };

            let _ = parsing_errors
                .into_iter()
                .for_each(|ParsingError { check_id, error }| {
                    println!("{} - {}", validation::error_header(&check_id), error);
                });

            let _ = validation_errors
                .into_iter()
                .map(Result::unwrap_err)
                .for_each(|errors| {
                    errors.iter().for_each(|error| println!("{}", error));
                });

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
}
