use super::types::{Check, FactDeclaration, ParsingError};

use yaml_rust::yaml::{Array, Hash, Yaml};
use yaml_rust::YamlLoader;

pub fn string_to_yaml(input: String) -> Vec<Yaml> {
    YamlLoader::load_from_str(&input).expect("Unable to parse YAML content")
}

pub fn get_checks(yaml_document: &Yaml) -> Vec<Result<Check, Vec<ParsingError>>> {
    yaml_document
        .as_hash()
        .unwrap_or(&Hash::new())
        .iter()
        .map(|entry| {
            let check_hash = &*entry.1.as_hash().unwrap();
            let mut errors = vec![];

            let id = field("id", check_hash)
                .map_err(|_| {
                    errors.push(ParsingError {
                        check_id: "Unknown id".to_string(),
                        error: "declared check has no id".to_string(),
                    })
                })
                .unwrap_or_default();

            let name = field("name", check_hash)
                .map_err(|_| {
                    errors.push(ParsingError {
                        check_id: id.clone(),
                        error: "check has no name declared".to_string(),
                    })
                })
                .unwrap_or_default();

            let group = field("group", check_hash)
                .map_err(|_| {
                    errors.push(ParsingError {
                        check_id: id.clone(),
                        error: "check has no group declared".to_string(),
                    })
                })
                .unwrap_or_default();

            let description = field("description", check_hash)
                .map_err(|_| {
                    errors.push(ParsingError {
                        check_id: id.clone(),
                        error: "check has no description declared".to_string(),
                    })
                })
                .unwrap_or_default();

            let remediation = field("remediation", check_hash)
                .map_err(|_| {
                    errors.push(ParsingError {
                        check_id: id.clone(),
                        error: "check has no remediation declared".to_string(),
                    })
                })
                .unwrap_or_default();

            let fact_declarations = fact_declarations(check_hash)
                .map_err(|fact_declarations_parsing_errors| {
                    let combined_error = fact_declarations_parsing_errors.iter().fold(
                        String::new(),
                        |acc, ParsingError { error, .. }| match acc.is_empty() {
                            true => error.to_string(),
                            false => format!("{}, {}", acc, error),
                        },
                    );
                    errors.push(ParsingError {
                        check_id: id.clone(),
                        error: combined_error,
                    })
                })
                .unwrap_or_default();

            match errors.is_empty() {
                true => Ok(Check {
                    id: id,
                    name: name,
                    group: group,
                    description: description,
                    remediation: remediation,
                    expectations: vec![],
                    facts: fact_declarations,
                }),
                false => Err(errors),
            }
        })
        .collect()
}

fn field(field: &str, yaml_hash: &Hash) -> Result<String, ParsingError> {
    yaml_hash
        .get(&Yaml::from_str(field))
        .map_or(
            Err(ParsingError {
                check_id: "".to_string(),
                error: format!("{} not found", field),
            }),
            |value| {
                value.as_str().ok_or(ParsingError {
                    check_id: "".to_string(),
                    error: format!("{} not found", field),
                })
            },
        )
        .map_or(
            Err(ParsingError {
                check_id: "".to_string(),
                error: format!("{} not found", field),
            }),
            |value| Ok(value.to_string()),
        )
}

fn fact_declarations(yaml_hash: &Hash) -> Result<Vec<FactDeclaration>, Vec<ParsingError>> {
    yaml_hash
        .get(&Yaml::from_str("facts"))
        .unwrap()
        .as_vec()
        .unwrap_or(&Array::new())
        .clone()
        .iter()
        .enumerate()
        .map(|(index, hash)| {
            let yaml_fact = hash.as_hash().unwrap_or(&Hash::new()).clone();
            let mut errors = vec![];

            let fact_name = field("name", &yaml_fact)
                .map_err(|_| {
                    errors.push(ParsingError {
                        check_id: "".to_string(),
                        error: format!("fact {} - name not defined", index),
                    })
                })
                .unwrap_or_default();

            let gatherer = field("gatherer", &yaml_fact)
                .map_err(|_| {
                    errors.push(ParsingError {
                        check_id: "".to_string(),
                        error: format!("fact {} - gatherer not defined", index),
                    })
                })
                .unwrap_or_default();

            let argument = field("argument", &yaml_fact)
                .map_err(|_| {
                    errors.push(ParsingError {
                        check_id: "".to_string(),
                        error: format!("fact {} - argument not defined", index),
                    })
                })
                .unwrap_or_default();

            match errors.is_empty() {
                true => Ok(FactDeclaration {
                    fact_name: fact_name,
                    gatherer: gatherer,
                    arguments: vec![argument],
                }),
                false => Err(errors),
            }
        })
        .collect()
}

#[cfg(test)]

mod test {
    use super::super::parsing;

    #[test]
    fn successfully_extract_checks() {
        let input = "
            check_corosync_token_timeout:
              id: 156F64
              name: Corosync configuration file
              group: Corosync
              description: |
                Corosync `token` timeout is set to `{{ platform.corosync.expectedTokenTimeout }}`
              remediation: |
                ## Abstract
                The value of the Corosync `token` timeout is not set as recommended.
                ## Remediation
                ...
              facts:
                -
                  name: corosync_token_timeout
                  gatherer: corosync
                  argument: totem.token
                -
                  name: some_other_fact_useful_for_this_check
                  gatherer: another_reference_to_a_gatherer
                  argument: something_else
              expectations:
                TDB: TBD
        "
        .to_string();
        let yaml_documents = parsing::string_to_yaml(input);
        let checks = parsing::get_checks(&yaml_documents[0]);

        let check = checks[0].as_ref().unwrap();
        assert_eq!(check.id, "156F64");
        assert_eq!(check.name, "Corosync configuration file");
    }
}
