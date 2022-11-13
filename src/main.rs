extern crate corosync_config_parser;

use clap::Parser;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

pub mod dsl;

use dsl::parsing;
use dsl::types::ParsingError;
use dsl::validation;
use dsl::validation::Validate;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    file: Option<String>,
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

    let input = get_input(args.file);

    let yaml_documents = parsing::string_to_yaml(input);

    let (checks, parsing_errors): (Vec<_>, Vec<_>) = parsing::get_checks(&yaml_documents[0])
        .into_iter()
        .partition(Result::is_ok);

    let (_, validation_errors): (Vec<_>, Vec<_>) = checks
        .into_iter()
        .map(Result::unwrap)
        .map(|check| check.validate())
        .partition(Result::is_ok);

    let exit_code = match parsing_errors.is_empty() && validation_errors.is_empty() {
        true => 0,
        false => 1,
    };

    let _ = parsing_errors
        .into_iter()
        .map(Result::unwrap_err)
        .for_each(|errors| {
            errors.iter().for_each(|ParsingError { check_id, error }| {
                println!("{} - {}", validation::error_header(&check_id), error)
            })
        });

    let _ = validation_errors
        .into_iter()
        .map(Result::unwrap_err)
        .for_each(|errors| {
            errors.iter().for_each(|error| println!("{}", error));
        });

    process::exit(exit_code);
}
