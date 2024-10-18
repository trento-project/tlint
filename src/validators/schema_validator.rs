use jsonschema::JSONSchema;

use crate::dsl::types::{ValidationDiagnostic, Validator};

pub struct SchemaValidator<'a> {
    pub schema: &'a JSONSchema,
}

impl<'a> SchemaValidator<'a> {
    pub fn validate(
        &self,
        json_check: &serde_json::Value,
    ) -> Result<(), Vec<ValidationDiagnostic>> {
        match self.schema.validate(json_check) {
            Ok(v) => Ok(v),
            Err(errors) => Err(errors
                .map(|e| {
                    // TODO: Match on e.kind and decide what is a warning and what is an error
                    ValidationDiagnostic::Critical {
                        message: e.to_string(),
                        instance_path: e.instance_path.to_string(),
                    }
                })
                .collect()),
        }
    }
}

impl<'a> Validator for SchemaValidator<'a> {
    fn validate(&self, json_check: &serde_json::Value) -> Result<(), Vec<ValidationDiagnostic>> {
        self.validate(json_check)
    }
}

#[cfg(test)]
mod tests {
    use crate::dsl::{types::Check, validation::get_json_schema};

    use super::*;

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

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("the test string should be valid yaml");
        let json_schema = get_json_schema();
        let validator = SchemaValidator {
            schema: &json_schema,
        };
        let diagnostics = validator.validate(&json_value);

        assert!(diagnostics.as_ref().is_err_and(|d| d.len() == 2));

        if let Err(diagnostics) = diagnostics {
            match &diagnostics[0] {
                ValidationDiagnostic::Critical {
                    message,
                    instance_path,
                } => {
                    assert_eq!(
                        message,
                        "Additional properties are not allowed ('whens' was unexpected)"
                    );
                    assert_eq!(instance_path, "/values/0/conditions/1");
                }
                v => panic!("Unexpected variant {:?}", v),
            };

            match &diagnostics[1] {
                ValidationDiagnostic::Critical {
                    message,
                    instance_path,
                } => {
                    assert_eq!(message, "\"when\" is a required property");
                    assert_eq!(instance_path, "/values/0/conditions/1");
                }
                v => panic!("Unexpected variant {:?}", v),
            };
        };
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

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("the test string should be valid yaml");
        let json_schema = get_json_schema();
        let validator = SchemaValidator {
            schema: &json_schema,
        };
        let validation_diagnostics = validator.validate(&json_value);

        assert!(
            validation_diagnostics.is_ok(),
            "a valid check should return the Ok variant"
        );

        // FIXME: What does this really test?
        let deserialization_result = serde_yaml::from_str::<Check>(input);
        assert!(deserialization_result.is_ok());
    }
}
