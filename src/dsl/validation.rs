use colored::*;

pub trait Validate {
    fn validate(&self) -> Result<(), Vec<String>>;
}

pub fn error_header(head: &str) -> String {
    format!("  {}  ", head).on_red().black().to_string()
}

pub fn string_not_empty(operand: &str, prefix: &str) -> Result<(), String> {
    match operand.is_empty() {
        true => Err(format!("{} - String must not be empty", prefix)),
        false => Ok(()),
    }
}

pub fn list_not_empty<T>(operand: &[T], prefix: &str) -> Result<(), String> {
    match operand.is_empty() {
        true => Err(format!("{} - List must not be empty", prefix)),
        false => Ok(()),
    }
}
