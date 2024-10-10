use super::types::{ValidationError, Validator};
use crate::validators::expectation_validator::ExpectationValidator;
use crate::validators::schema_validator::SchemaValidator;
use crate::validators::value_validator::ValueValidator;
use colored::*;
use jsonschema::{Draft, JSONSchema};
use rhai::Engine;

const SCHEMA: &str = include_str!("../../check_definition.schema.json");

pub fn error_header(head: &str) -> String {
    format!("  {}  ", head).on_red().black().to_string()
}

pub fn validate(
    json_check: &serde_json::Value,
    check_id: &str,
    schema: &JSONSchema,
    engine: &Engine,
) -> Result<(), Vec<ValidationError>> {
    let schema_validator = SchemaValidator { schema };
    let expectation_validator = ExpectationValidator { engine };
    let value_validator = ValueValidator { engine };

    let validators: Vec<&dyn Validator> =
        vec![&schema_validator, &expectation_validator, &value_validator];

    let errors: Vec<ValidationError> = validators
        .iter()
        .flat_map(|validator| validator.validate(json_check, check_id))
        .collect();

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
                failure_message: some critical message
                warning_message: some warning message
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
    fn validate_check_failure_and_warning_message_expect_enum_ok() {
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
                failure_message: Expectation not met. Timeout value is ${facts.corosync_token_timeout}
                warning_message: Warning! Timeout value is ${values.expected_warning_value}
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
    fn validate_deprecated_property() {
        let input = r#"
            id: 156f64
            name: corosync configuration file
            group: corosync
            description: |
              corosync `token` timeout is set to expected value
            remediation: |
              ## abstract
              the value of the corosync `token` timeout is not set as recommended.
              ## remediation
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
            serde_yaml::from_str(input).expect("unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156f64", &json_schema, &engine);

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        assert!(validation_result.is_err());
        assert!(deserialization_result.is_ok());
    }

    #[test]
    fn validate_deprecated_property_and_invalid_check() {
        let input = r#"
            id: 156F64
            name: Corosync configuration file
            group: Corosync
            descriptio: |
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
                failure_message: Expectation not met. Timeout value is ${facts.corosync_token_timeout}
                warning_message: Warning! Timeout value is ${values.expected_warning_value}
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_result = validate(&json_value, "156f64", &json_schema, &engine);

        assert!(validation_result.is_err());
        if let Err(results) = validation_result {
            assert_eq!(results.len(), 2);
        }
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

    #[test]
    fn validate_invalid_warning_message() {
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
            failure_message: some critical message
            warning_message: some warning message with ${facts.corosync_token_timeout
    "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_errors = validate(&json_value, "156F64", &json_schema, &engine).unwrap_err();
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(
            validation_errors[0].error,
            "Open string is not terminated (line 1, position 58)"
        );
        assert_eq!(validation_errors[0].instance_path, "/expectations/0");
    }

    #[test]
    fn validate_warning_message_only_expect_enum() {
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
                expect: facts.corosync_token_timeout == values.expected_token_timeout
                warning_message: some message
              - name: timeout_same
                expect_same: facts.corosync_token_timeout
                warning_message: some message
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_errors = validate(&json_value, "156F64", &json_schema, &engine).unwrap_err();
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(
            validation_errors[0].error,
            "warning_message is only available for expect_enum expectations"
        );
        assert_eq!(validation_errors[0].instance_path, "/expectations/0");

        assert_eq!(validation_errors[1].check_id, "156F64");
        assert_eq!(
            validation_errors[1].error,
            "warning_message is only available for expect_enum expectations"
        );
        assert_eq!(validation_errors[1].instance_path, "/expectations/1");
    }

    #[test]
    fn validate_invalid_expect_enum_without_returns() {
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
            expectations:
              - name: timeout
                expect_enum: facts.corosync_token_timeout == values.expected_passing_value
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let json_schema = get_json_schema();
        let validation_errors = validate(&json_value, "156F64", &json_schema, &engine).unwrap_err();
        assert_eq!(validation_errors[0].check_id, "156F64");
        assert_eq!(validation_errors[0].error, "passing return value not found");
        assert_eq!(validation_errors[0].instance_path, "/expectations/0");
        assert_eq!(
          validation_errors[1].error,
          "warning return value not found. Consider using `expect` expression if a warning return is not needed"
      );
        assert_eq!(validation_errors[1].instance_path, "/expectations/0");
    }
}
