use crate::dsl::types::{ValidationDiagnostic, Validator};
use jsonschema::{output::BasicOutput, JSONSchema};
use serde_json;

pub struct SchemaValidator<'a> {
    pub schema: &'a JSONSchema,
}

fn collect_deprecations(
    json_check: &serde_json::Value,
    check_id: &str,
    schema: &JSONSchema,
) -> Vec<ValidationDiagnostic> {
    match schema.apply(json_check).basic() {
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
                let err_description = match annotation.instance_location().last().unwrap() {
                    jsonschema::paths::PathChunk::Property(name) => format!("Property '{name}'"),
                    jsonschema::paths::PathChunk::Index(idx) => format!("Element[{idx}]"),
                    jsonschema::paths::PathChunk::Keyword(name) => format!("Keyword '{name}'"),
                };

                ValidationDiagnostic::Warning {
                    check_id: check_id.to_string(),
                    message: format!(
                        "{err_description} is deprecated and will be removed in the future"
                    ),
                    instance_path: annotation.instance_location().to_string(),
                }
            })
            .collect::<Vec<_>>(),

        BasicOutput::Invalid(_) => vec![],
    }
}

impl<'a> Validator for SchemaValidator<'a> {
    fn validate(
        &self,
        json_check: &serde_json::Value,
        check_id: &str,
    ) -> Vec<ValidationDiagnostic> {
        validate_schema(json_check, check_id, self.schema)
    }
}

fn validate_schema(
    json_check: &serde_json::Value,
    check_id: &str,
    schema: &JSONSchema,
) -> Vec<ValidationDiagnostic> {
    let deprecation_warnings = collect_deprecations(json_check, check_id, schema);

    let mut validation_errors = match schema.validate(json_check) {
        Ok(_) => vec![],
        Err(errors) => errors
            .map(|error| ValidationDiagnostic::Critical {
                check_id: check_id.to_string(),
                message: error.to_string(),
                instance_path: error.instance_path.to_string(),
            })
            .collect(),
    };

    validation_errors.extend(deprecation_warnings);
    validation_errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::types::Check;
    use crate::dsl::validation::get_json_schema;
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

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("the test string should be valid yaml");
        let json_schema = get_json_schema();
        let validator = SchemaValidator {
            schema: &json_schema,
        };

        let expected_check_id = "156F64";
        let diagnostics = validator.validate(&json_value, expected_check_id);

        assert!(diagnostics.len() == 2);

        match &diagnostics[0] {
            w @ ValidationDiagnostic::Warning { .. } => panic!("Unexpected variant {:?}", w),
            ValidationDiagnostic::Critical {
                message,
                instance_path,
                check_id,
            } => {
                assert_eq!(check_id, expected_check_id);
                assert_eq!(
                    message,
                    "Additional properties are not allowed ('whens' was unexpected)"
                );
                assert_eq!(instance_path, "/values/0/conditions/1");
            }
        };

        match &diagnostics[1] {
            w @ ValidationDiagnostic::Warning { .. } => panic!("Unexpected variant {:?}", w),
            ValidationDiagnostic::Critical {
                message,
                instance_path,
                check_id,
            } => {
                assert_eq!(check_id, expected_check_id);
                assert_eq!(message, "\"when\" is a required property");
                assert_eq!(instance_path, "/values/0/conditions/1");
            }
        };
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
        let validator = SchemaValidator {
            schema: &json_schema,
        };

        let expected_check_id = "156F64";
        let diagnostics = validator.validate(&json_value, expected_check_id);

        assert!(diagnostics.len() == 1);

        match &diagnostics[0] {
            ValidationDiagnostic::Warning {
                message,
                instance_path,
                check_id,
            } => {
                assert_eq!(check_id, expected_check_id);
                assert_eq!(
                    message,
                    "Property 'premium' is deprecated and will be removed in the future"
                );
                assert_eq!(instance_path, "/premium");
            }
            e @ ValidationDiagnostic::Critical { .. } => panic!("Unexpected variant {:?}", e),
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
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_result = validate_schema(&json_value, "156F64", &json_schema);

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        assert!(validation_result.is_empty());
        assert!(deserialization_result.is_ok());
    }
}
