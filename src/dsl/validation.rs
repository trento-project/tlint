use super::types::ValidationError;
use colored::*;
use jsonschema::{Draft, JSONSchema};
use rhai::{Engine, Expr, Stmt};
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
        .enumerate()
        .flat_map(|(index, value)| {
            let expect = value.get("expect");
            let expect_same = value.get("expect_same");
            let expect_enum = value.get("expect_enum");
            let failure_message = value.get("failure_message");

            let is_expect = expect.is_some();
            let is_expect_same = expect_same.is_some();
            let is_expect_enum = expect_enum.is_some();

            let mut results = vec![];

            let expectation_expression = if is_expect {
              expect.unwrap().as_str().unwrap()
            } else if is_expect_same {
              expect_same.unwrap().as_str().unwrap()
            } else if is_expect_enum {
              expect_enum.unwrap().as_str().unwrap()
            } else {
              ""
            };

            match engine.compile(expectation_expression) {
                Ok(_) => results.push(Ok(())),
                Err(error) => results.push(Err(ValidationError {
                    check_id: check_id.to_string(),
                    error: error.to_string(),
                    instance_path: format!("/expectations/{:?}", index).to_string(),
                })),
            };

            if failure_message.is_some() {
                let failure_message_expression = failure_message.unwrap().as_str().unwrap();
                results.push(validate_string_expression(
                    failure_message_expression,
                    engine,
                    check_id,
                    index,
                    is_expect,
                ));
            };

            results
        })
        .partition(Result::is_ok);

    let mut expectation_errors: Vec<ValidationError> = expectation_expression_errors
        .into_iter()
        .map(Result::unwrap_err)
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

fn validate_string_expression(
    expression: &str,
    engine: &Engine,
    check_id: &str,
    index: usize,
    allow_interpolated_strings: bool,
) -> Result<(), ValidationError> {
    match engine.compile(format!("`{}`", expression)) {
        Ok(ast) => {
            let statements = ast.statements();
            if statements.len() > 1 {
                return Err(ValidationError {
                    check_id: check_id.to_string(),
                    error: "Too many statements".to_string(),
                    instance_path: format!("/expectations/{:?}", index).to_string(),
                });
            }

            match &statements[0] {
                Stmt::Expr(expression) => match **expression {
                    Expr::StringConstant(_, _) => Ok(()),
                    Expr::InterpolatedString(_, _) => {
                        if !allow_interpolated_strings {
                            Err(ValidationError {
                                check_id: check_id.to_string(),
                                error: "String interpolation is not allowed here".to_string(),
                                instance_path: format!("/expectations/{:?}", index).to_string(),
                            })
                        } else {
                            Ok(())
                        }
                    }
                    _ => Err(ValidationError {
                        check_id: check_id.to_string(),
                        error: "Field has to be a string".to_string(),
                        instance_path: format!("/expectations/{:?}", index).to_string(),
                    }),
                },
                _ => Err(ValidationError {
                    check_id: check_id.to_string(),
                    error: "Field has to be an expression".to_string(),
                    instance_path: format!("/expectations/{:?}", index).to_string(),
                }),
            }
        }
        Err(error) => Err(ValidationError {
            check_id: check_id.to_string(),
            error: error.to_string(),
            instance_path: format!("/expectations/{:?}", index).to_string(),
        }),
    }
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
            serde_yaml::from_str(input).expect("Unable to parse yaml");
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
            premium: true
            metadata:
              target_type: cluster
              provider:
                - aws
                - azure
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
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        assert!(validation_result.is_ok());
        assert!(deserialization_result.is_ok());
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
            serde_yaml::from_str(input).expect("Unable to parse yaml");
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
            serde_yaml::from_str(input).expect("Unable to parse yaml");
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
            serde_yaml::from_str(input).expect("Unable to parse yaml");

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        assert!(validation_result.is_ok());
        assert!(deserialization_result.is_ok());
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
            serde_yaml::from_str(input).expect("Unable to parse yaml");

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        assert!(validation_result.is_ok());
        assert!(deserialization_result.is_ok());
    }

    #[test]
    fn validate_check_expect_enum() {
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
              - name: expected_passing_value
                default: 5000
              - name: expected_warning_value
                default: 3000
            expectations:
              - name: timeout
                expect_enum: |
                  if facts.corosync_token_timeout == values.expected_passing_value {
                    "passing"
                  } else if facts.corosync_token_timeout == values.expected_warning_value {
                    "warning"
                  } else {
                    "critical"
                  }
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        assert!(validation_result.is_ok());
        assert!(deserialization_result.is_ok());
    }

    #[test]
    fn validate_check_failure_message_expect_ok() {
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
                failure_message: Expectation not met ${facts.corosync_token_timeout}
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        println!("{:?}", validation_result);

        assert!(validation_result.is_ok());
        assert!(deserialization_result.is_ok());
    }

    #[test]
    fn validate_check_failure_message_expect_same_ok() {
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
                failure_message: Expectation not met
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        println!("{:?}", validation_result);

        assert!(validation_result.is_ok());
        assert!(deserialization_result.is_ok());
    }

    #[test]
    fn validate_check_failure_message_expect_same_invalid() {
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
                failure_message: Expectation not met ${facts.corosync_token_timeout}
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156F64", &json_schema, &engine);

        println!("{:?}", validation_result);

        assert!(validation_result.is_err());
        assert!(deserialization_result.is_ok());
    }

    #[test]
    fn validate_invalid_metadata() {
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
        premium: true
        metadata:
          "": empty
          "  ": whitespace
          target_type: cluster
          provider:
            - aws
            - azure
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
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_errors = validate(&json_value, "156F64", &json_schema, &engine).unwrap_err();
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(
            validation_errors[0].error,
            "Additional properties are not allowed ('', '  ' were unexpected)"
        );
        assert_eq!(validation_errors[0].instance_path, "/metadata");
    }

    #[test]
    fn validate_expression_missing() {
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

            expectations:
              - name: timeout
                failure_message: critical!
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_errors = validate(&json_value, "156F64", &json_schema, &engine).unwrap_err();
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(
            validation_errors[0].error,
            "{\"failure_message\":\"critical!\",\"name\":\"timeout\"} is not valid under any of the given schemas"
        );
        assert_eq!(validation_errors[0].instance_path, "/expectations/0");
    }
}
