use crate::dsl::types::{ValidationDiagnostic, Validator};
use rhai::Engine;
use serde_json::json;

pub struct ValueValidator<'a> {
    pub engine: &'a Engine,
}

impl<'a> Validator for ValueValidator<'a> {
    fn validate(
        &self,
        json_check: &serde_json::Value,
        check_id: &str,
    ) -> Vec<ValidationDiagnostic> {
        validate_values(json_check, check_id, self.engine)
    }
}

fn validate_values(
    json_check: &serde_json::Value,
    check_id: &str,
    engine: &Engine,
) -> Vec<ValidationDiagnostic> {
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
                    engine.compile(when_expression).map_err(|error| {
                        ValidationDiagnostic::Critical {
                            check_id: check_id.to_string(),
                            message: error.to_string(),
                            instance_path: format!(
                                "/values/{:?}/conditions/{:?}",
                                value_index, condition_index
                            ),
                        }
                    })
                })
                .collect();

            conditions_compilations_results
        })
        .partition(Result::is_ok);

    return values_expression_errors
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
        let validation_result = validate_values(&json_value, "156F64", &engine);

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        assert!(validation_result.is_empty());
        assert!(deserialization_result.is_ok());
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
        let validation_errors = validate_values(&json_value, "156F64", &engine);
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(
            validation_errors[0].error,
            "Unknown operator: '?' (line 1, position 5)"
        );
        assert_eq!(validation_errors[0].instance_path, "/values/0/conditions/0");
    }
}
