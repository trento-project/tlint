use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Check {
    pub id: String,
    pub name: String,
    pub group: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub when: Option<String>,
    pub description: String,
    pub remediation: String,
    pub premium: Option<bool>,
    pub facts: Vec<FactDeclaration>,
    pub expectations: Vec<Expectation>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FactDeclaration {
    pub name: String,
    pub gatherer: String,
    pub argument: Option<String>,
}

#[derive(Debug)]
pub struct Fact {
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Expectation {
    pub name: String,
    pub expect: Option<String>,
    pub expect_same: Option<String>,
    pub failure_message: Option<String>,
}

#[derive(Debug)]
pub enum Predicate {
    String,
    Bool,
    U64,
}

#[derive(Debug)]
pub struct ParsingError {
    pub check_id: String,
    pub error: String,
}

#[derive(Debug)]
pub struct ValidationError {
    pub check_id: String,
    pub error: String,
    pub instance_path: String,
}
