use super::types::ValidationError;
use colored::*;
use jsonschema::{Draft, JSONSchema};
use rhai::Engine;
use serde_json::json;

const SCHEMA: &str = include_str!("../../wanda/guides/check_definition.schema.json");

pub fn error_header(head: &str) -> String {
    format!("  {}  ", head).on_red().black().to_string()
}

pub fn validate(
    json_check: &serde_json::Value,
    check_id: &str,
    schema: &JSONSchema,
    engine: &Engine,
) -> Result<(), Vec<ValidationError>> {
    let validation_result = match schema.validate(json_check) {
        Ok(_) => Ok(()),
        Err(errors) => {
            let validation_errors = errors
                .map(|error| ValidationError {
                    check_id: check_id.to_string(),
                    error: error.to_string(),
                    instance_path: error.instance_path.to_string(),
                })
                .collect();
            Err(validation_errors)
        }
    };

    let mut schema_validation_errors = match validation_result {
        Ok(_) => vec![],
        Err(errors) => errors,
    };

    let (_, expectation_expression_errors): (Vec<_>, Vec<_>) = json_check
        .get("expectations")
        .unwrap_or(&json!([]))
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|value| {
            let expect = value.get("expect");
            let expect_same = value.get("expect_same");

            let expectation_expression = if expect.is_some() {
                expect.unwrap().as_str().unwrap()
            } else {
                expect_same.unwrap().as_str().unwrap()
            };

            engine.compile(expectation_expression)
        })
        .partition(Result::is_ok);

    let mut expectation_errors: Vec<ValidationError> = expectation_expression_errors
        .into_iter()
        .map(Result::unwrap_err)
        .enumerate()
        .map(|(index, error)| ValidationError {
            check_id: check_id.to_string(),
            error: error.to_string(),
            instance_path: format!("/expectations/{:?}", index).to_string(),
        })
        .collect();

    let (_, values_expression_errors): (Vec<_>, Vec<_>) = json_check
        .get("values")
        .unwrap_or(&json!([]))
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .enumerate()
        .flat_map(|(value_index, value)| {
            let conditions_compilations_results: Vec<Result<_, _>> = value
                .get("conditions")
                .unwrap_or(&json!([]))
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .enumerate()
                .map(|(condition_index, condition)| {
                    let default_json_string = json!("");
                    let when_expression = condition
                        .get("when")
                        .unwrap_or(&default_json_string)
                        .as_str()
                        .unwrap();
                    engine
                        .compile(when_expression)
                        .map_err(|error| ValidationError {
                            check_id: check_id.to_string(),
                            error: error.to_string(),
                            instance_path: format!(
                                "/values/{:?}/conditions/{:?}",
                                value_index, condition_index
                            ),
                        })
                })
                .collect();

            conditions_compilations_results
        })
        .partition(Result::is_ok);

    let mut values_errors = values_expression_errors
        .into_iter()
        .map(Result::unwrap_err)
        .collect();

    let mut errors = vec![];
    errors.append(&mut schema_validation_errors);
    errors.append(&mut expectation_errors);
    errors.append(&mut values_errors);

    if errors.is_empty() {
        return Ok(());
    }

    Err(errors)
}

pub fn get_json_schema() -> JSONSchema {
    let value = serde_json::from_str(SCHEMA).unwrap();

    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&value)
        .expect("A valid schema");

    compiled_schema
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::types::Check;
    use rhai::Engine;
    use serde_json;

    #[test]
    fn validate_wrong_check() {
        let input = r#"
            id: 156F64
            name: Corosync configuration file
            group: Corosync
            description: |
              Corosync `token` timeout is set to expected value
            remediation: |
              ## Abstract
              The value of the Corosync `token` timeout is not set as recommended.
              ## Remediation
              ...
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
                argument: totem.token
            values:
              - name: expected_token_timeout
                default: 5000
                conditions:
                  - value: 30000
                    when: env.provider == "azure" || env.provider == "aws"
                  - value: 20000
                    whens: env.provider == "gcp"
            expectations:
              - name: timeout
                expect: facts.corosync_token_timeout == values.expected_token_timeout
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(&input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_errors = validate(&json_value, "156F64", &json_schema, &engine).unwrap_err();
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(
            validation_errors[0].error,
            "Additional properties are not allowed ('whens' was unexpected)"
        );
        assert_eq!(validation_errors[0].instance_path, "/values/0/conditions/1");
    }

    #[test]
    fn validate_ok_check() {
        let input = r#"
            id: 156F64
            name: Corosync configuration file
            group: Corosync
            description: |
              Corosync `token` timeout is set to expected value
            remediation: |
              ## Abstract
              The value of the Corosync `token` timeout is not set as recommended.
              ## Remediation
              ...
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
                argument: totem.token
            values:
              - name: expected_token_timeout
                default: 5000
                conditions:
                  - value: 30000
                    when: env.provider == "azure" || env.provider == "aws"
                  - value: 20000
                    when: env.provider == "gcp"
            expectations:
              - name: timeout
                expect: facts.corosync_token_timeout == values.expected_token_timeout
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(&input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        assert_eq!(validation_result.is_ok(), true);
    }

    #[test]
    fn validate_invalid_expect_expectation_check() {
        let input = r#"
            id: 156F64
            name: Corosync configuration file
            group: Corosync
            description: |
              Corosync `token` timeout is set to expected value
            remediation: |
              ## Abstract
              The value of the Corosync `token` timeout is not set as recommended.
              ## Remediation
              ...
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
                argument: totem.token
            values:
              - name: expected_token_timeout
                default: 5000
                conditions:
                  - value: 30000
                    when: env.provider == "azure" || env.provider == "aws"
                  - value: 20000
                    when: env.provider == "gcp"
            expectations:
              - name: timeout
                expect: kekw?
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(&input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_errors = validate(&json_value, "156F64", &json_schema, &engine).unwrap_err();
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(
            validation_errors[0].error,
            "Unknown operator: '?' (line 1, position 5)"
        );
        assert_eq!(validation_errors[0].instance_path, "/expectations/0");
    }

    #[test]
    fn validate_invalid_value() {
        let input = r#"
            id: 156F64
            name: Corosync configuration file
            group: Corosync
            description: |
              Corosync `token` timeout is set to expected value
            remediation: |
              ## Abstract
              The value of the Corosync `token` timeout is not set as recommended.
              ## Remediation
              ...
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
                argument: totem.token
            values:
              - name: expected_token_timeout
                default: 5000
                conditions:
                  - value: 30000
                    when: kekw?
                  - value: 20000
                    when: env.provider == "gcp"
            expectations:
              - name: timeout
                expect: facts.corosync_token_timeout == values.expected_token_timeout 
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(&input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_errors = validate(&json_value, "156F64", &json_schema, &engine).unwrap_err();
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(
            validation_errors[0].error,
            "Unknown operator: '?' (line 1, position 5)"
        );
        assert_eq!(validation_errors[0].instance_path, "/values/0/conditions/0");
    }

    #[test]
    fn validate_check_with_gatherer_no_arguments() {
        let input = r#"
            id: 156F64
            name: Corosync configuration file
            group: Corosync
            description: |
              Corosync `token` timeout is set to expected value
            remediation: |
              ## Abstract
              The value of the Corosync `token` timeout is not set as recommended.
              ## Remediation
              ...
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
            values:
              - name: expected_token_timeout
                default: 5000
                conditions:
                  - value: 30000
                    when: env.provider == "azure" || env.provider == "aws"
                  - value: 20000
                    when: env.provider == "gcp"
            expectations:
              - name: timeout
                expect: facts.corosync_token_timeout == values.expected_token_timeout
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(&input).expect("Unable to parse yaml");

        let deserialization_result = serde_yaml::from_str::<Check>(&input);

        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        assert_eq!(validation_result.is_ok(), true);
        assert_eq!(deserialization_result.is_ok(), true);
    }

    #[test]
    fn validate_check_expect_same() {
        let input = r#"
            id: 156F64
            name: Corosync configuration file
            group: Corosync
            description: |
              Corosync `token` timeout is set to expected value
            remediation: |
              ## Abstract
              The value of the Corosync `token` timeout is not set as recommended.
              ## Remediation
              ...
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
            values:
              - name: expected_token_timeout
                default: 5000
                conditions:
                  - value: 30000
                    when: env.provider == "azure" || env.provider == "aws"
                  - value: 20000
                    when: env.provider == "gcp"
            expectations:
              - name: timeout
                expect_same: facts.corosync_token_timeout == values.expected_token_timeout
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(&input).expect("Unable to parse yaml");

        let deserialization_result = serde_yaml::from_str::<Check>(&input);

        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        assert_eq!(validation_result.is_ok(), true);
        assert_eq!(deserialization_result.is_ok(), true);
    }
}
