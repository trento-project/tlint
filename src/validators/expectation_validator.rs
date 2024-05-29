use crate::dsl::types::{ValidationError, Validator};
use rhai::{Engine, Expr, Stmt};
use serde_json::json;

pub struct ExpectationValidator<'a> {
    pub engine: &'a Engine,
}

impl<'a> Validator for ExpectationValidator<'a> {
    fn validate(&self, json_check: &serde_json::Value, check_id: &str) -> Vec<ValidationError> {
        validate_expectations(json_check, check_id, self.engine)
    }
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

fn validate_expect_enum_content(
    expression: &str,
    check_id: &str,
    index: usize,
) -> Vec<Result<(), ValidationError>> {
    let mut results = vec![];

    if !expression.contains("\"passing\"") {
        results.push(Err(ValidationError {
            check_id: check_id.to_string(),
            error: "passing return value not found".to_string(),
            instance_path: format!("/expectations/{:?}", index).to_string(),
        }));
    }

    if !expression.contains("\"warning\"") {
        results.push(Err(ValidationError {
      check_id: check_id.to_string(),
      error: "warning return value not found. Consider using `expect` expression if a warning return is not needed".to_string(),
      instance_path: format!("/expectations/{:?}", index).to_string(),
    }));
    }

    results
}

fn validate_expectations(
    json_check: &serde_json::Value,
    check_id: &str,
    engine: &Engine,
) -> Vec<ValidationError> {
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

            let is_expect = expect.is_some();
            let is_expect_same = expect_same.is_some();
            let is_expect_enum = expect_enum.is_some();

            let expectation_expression = if is_expect {
                expect.unwrap().as_str().unwrap()
            } else if is_expect_same {
                expect_same.unwrap().as_str().unwrap()
            } else if is_expect_enum {
                expect_enum.unwrap().as_str().unwrap()
            } else {
                ""
            };

            let mut results = vec![];

            match engine.compile(expectation_expression) {
                Ok(_) => results.push(Ok(())),
                Err(error) => results.push(Err(ValidationError {
                    check_id: check_id.to_string(),
                    error: error.to_string(),
                    instance_path: format!("/expectations/{:?}", index).to_string(),
                })),
            }

            let failure_message = value.get("failure_message");
            let warning_message = value.get("warning_message");

            if failure_message.is_some() {
                let failure_message_expression = failure_message.unwrap().as_str().unwrap();
                results.push(validate_string_expression(
                    failure_message_expression,
                    engine,
                    check_id,
                    index,
                    is_expect || is_expect_enum,
                ));
            }

            if warning_message.is_some() && !is_expect_enum {
                results.push(Err(ValidationError {
                    check_id: check_id.to_string(),
                    error: "warning_message is only available for expect_enum expectations"
                        .to_string(),
                    instance_path: format!("/expectations/{:?}", index).to_string(),
                }));
            } else if warning_message.is_some() {
                let warning_message_expression = warning_message.unwrap().as_str().unwrap();
                results.push(validate_string_expression(
                    warning_message_expression,
                    engine,
                    check_id,
                    index,
                    is_expect_enum,
                ));
            }

            if is_expect_enum {
                results.append(&mut validate_expect_enum_content(
                    expectation_expression,
                    check_id,
                    index,
                ));
            }

            results
        })
        .partition(Result::is_ok);

    return expectation_expression_errors
        .into_iter()
        .map(Result::unwrap_err)
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::types::Check;
    use rhai::Engine;
    use serde_json;

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
        let validation_result = validate_expectations(&json_value, "156F64", &engine);

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        assert!(validation_result.is_empty());
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
        let validation_errors = validate_expectations(&json_value, "156F64", &engine);
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(
            validation_errors[0].error,
            "Unknown operator: '?' (line 1, position 5)"
        );
        assert_eq!(validation_errors[0].instance_path, "/expectations/0");
    }
}
