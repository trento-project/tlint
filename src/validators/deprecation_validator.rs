use jsonschema::{BasicOutput, JSONSchema};

use crate::dsl::types::{ValidationDiagnostic, Validator};

pub struct DeprecationValidator<'a> {
    pub schema: &'a JSONSchema,
}

impl<'a> DeprecationValidator<'a> {
    pub fn validate(
        &self,
        json_check: &serde_json::Value,
    ) -> Result<(), Vec<ValidationDiagnostic>> {
        let diagnostics = match self.schema.apply(json_check).basic() {
            // FIXME: crate jsonschema does not resolve "$ref" to type definitions and therefore can
            // not detect deprecations in linked types
            BasicOutput::Valid(annotations) => annotations
                .into_iter()
                .filter(|annotation| match annotation.value().get("deprecated") {
                    Some(val) => match val.as_bool() {
                        Some(is_deprecated) => is_deprecated,
                        None => false,
                    },
                    None => false,
                })
                .map(|annotation| {
                    let err_description = match annotation.instance_location().last() {
                        Some(jsonschema::paths::PathChunk::Property(name)) => {
                            format!("Property '{}'", name)
                        }
                        Some(jsonschema::paths::PathChunk::Index(idx)) => {
                            format!("Element[{}]", idx)
                        }
                        Some(jsonschema::paths::PathChunk::Keyword(name)) => {
                            format!("Keyword '{}'", name)
                        }
                        None => "<unknown type>".to_string(),
                    };

                    ValidationDiagnostic::Warning {
                        message: format!(
                            "{} is deprecated and will be removed in the future",
                            err_description
                        ),
                        instance_path: annotation.instance_location().to_string(),
                    }
                })
                .collect::<Vec<_>>(),

            BasicOutput::Invalid(_) => Vec::new(),
        };

        if diagnostics.len() > 0 {
            return Err(diagnostics);
        }

        Ok(())
    }
}

impl<'a> Validator for DeprecationValidator<'a> {
    fn validate(&self, json_check: &serde_json::Value) -> Result<(), Vec<ValidationDiagnostic>> {
        self.validate(json_check)
    }
}

#[cfg(test)]
mod tests {
    use crate::dsl::validation::get_json_schema;

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
            premium: true
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
        let validator = DeprecationValidator {
            schema: &json_schema,
        };
        let diagnostics = validator.validate(&json_value);

        assert!(
            diagnostics.is_ok(),
            "an invalid check can not raise deprecation warnings"
        );
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
        let validator = DeprecationValidator {
            schema: &json_schema,
        };
        let diagnostics = validator.validate(&json_value);
        assert!(
            diagnostics.is_ok(),
            "a valid check can not raise deprecation warnings"
        );
    }

    #[test]
    fn validate_deprecated_check() {
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

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("the test string should be valid yaml");
        let json_schema = get_json_schema();
        let validator = DeprecationValidator {
            schema: &json_schema,
        };
        let diagnostics = validator.validate(&json_value);

        assert!(diagnostics.as_ref().is_err_and(|d| d.len() == 1));

        if let Err(diagnostics) = diagnostics {
            match &diagnostics[0] {
                ValidationDiagnostic::Warning {
                    message,
                    instance_path,
                } => {
                    assert_eq!(
                        message,
                        "Property 'premium' is deprecated and will be removed in the future"
                    );
                    assert_eq!(instance_path, "/premium");
                }
                v => panic!("Unexpected variant {:?}", v),
            };
        };
    }
}
