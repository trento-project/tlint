use super::types;

use yaml_rust::yaml::{Array, Hash, Yaml};
use yaml_rust::YamlLoader;

pub fn string_to_yaml(input: String) -> Vec<Yaml> {
    YamlLoader::load_from_str(&input).expect("Unable to parse YAML content")
}

pub fn get_checks(yaml_document: &Yaml) -> Vec<types::Check> {
    yaml_document
        .as_hash()
        .unwrap_or(&Hash::new())
        .iter()
        .map(|entry| {
            let check_hash = &*entry.1.as_hash().unwrap();
            types::Check {
                id: field_or_empty_string("id", check_hash),
                name: field_or_empty_string("name", check_hash),
                group: field_or_empty_string("group", check_hash),
                description: field_or_empty_string("description", check_hash),
                remediation: field_or_empty_string("remediation", check_hash),
                expectations: vec![],
                facts: fact_declarations(check_hash),
            }
        })
        .collect()
}

fn field_or_empty_string(field: &str, yaml_hash: &Hash) -> String {
    yaml_hash
        .get(&Yaml::from_str(field))
        .unwrap_or(&Yaml::from_str(""))
        .as_str()
        .unwrap_or("")
        .to_string()
}

fn fact_declarations(yaml_hash: &Hash) -> Vec<types::FactDeclaration> {
    yaml_hash
        .get(&Yaml::from_str("facts"))
        .unwrap()
        .as_vec()
        .unwrap_or(&Array::new())
        .clone()
        .iter()
        .map(|hash| {
            let yaml_fact = hash.as_hash().unwrap_or(&Hash::new()).clone();
            types::FactDeclaration {
                fact_name: field_or_empty_string("name", &yaml_fact),
                gatherer: field_or_empty_string("gatherer", &yaml_fact),
                arguments: vec![field_or_empty_string("argument", &yaml_fact)],
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
                  arguments: totem.token
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

        assert_eq!(checks[0].id, "156F64");
        assert_eq!(checks[0].name, "Corosync configuration file");
    }
}
