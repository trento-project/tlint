extern crate corosync_config_parser;

use clap::Parser;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

pub mod dsl;

use dsl::parsing;
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

    let checks = parsing::get_checks(&yaml_documents[0]);

    let (_, validation_errors): (Vec<_>, Vec<_>) = checks
        .iter()
        .map(|check| check.validate())
        .partition(Result::is_ok);

    let exit_code = match validation_errors.is_empty() {
        true => 0,
        false => 1,
    };

    let _ = validation_errors
        .into_iter()
        .map(Result::unwrap_err)
        .for_each(|errors| {
            errors.iter().for_each(|error| println!("{}", error));
        });

    process::exit(exit_code);
}
