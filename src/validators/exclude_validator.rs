// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

use crate::dsl::types::{ValidationDiagnostic, Validator};
use rhai::Engine;

pub struct ExcludeValidator<'a> {
    pub engine: &'a Engine,
}

impl<'a> Validator for ExcludeValidator<'a> {
    fn validate(
        &self,
        json_check: &serde_json::Value,
        check_id: &str,
    ) -> Vec<ValidationDiagnostic> {
        validate_exclude(json_check, check_id, self.engine)
    }
}

fn validate_exclude(
    json_check: &serde_json::Value,
    check_id: &str,
    engine: &Engine,
) -> Vec<ValidationDiagnostic> {
    let Some(exclude) = json_check.get("exclude") else {
        return vec![];
    };

    let Some(exclude_expression) = exclude.as_str() else {
        return vec![ValidationDiagnostic::Critical {
            check_id: check_id.to_string(),
            message: "exclude must be a string expression".to_string(),
            instance_path: "/exclude".to_string(),
        }];
    };

    match engine.compile(exclude_expression) {
        Ok(_) => vec![],
        Err(error) => vec![ValidationDiagnostic::Critical {
            check_id: check_id.to_string(),
            message: error.to_string(),
            instance_path: "/exclude".to_string(),
        }],
    }
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
            exclude: host.is_majority_maker == true
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
                argument: totem.token
            expectations:
              - name: timeout
                expect: facts.corosync_token_timeout == 5000
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let validation_result = validate_exclude(&json_value, "156F64", &engine);

        let deserialization_result = serde_yaml::from_str::<Check>(input);

        assert!(validation_result.is_empty());
        assert!(deserialization_result.is_ok());
    }

    #[test]
    fn validate_check_without_exclude() {
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
            expectations:
              - name: timeout
                expect: facts.corosync_token_timeout == 5000
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let validation_result = validate_exclude(&json_value, "156F64", &engine);

        assert!(validation_result.is_empty());
    }

    #[test]
    fn validate_check_multi_line_exclude() {
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
            exclude: |
                if host.is_majority_maker == true {
                    return true;
                } else {
                    return false;
                }
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
                argument: totem.token
            expectations:
              - name: timeout
                expect: facts.corosync_token_timeout == 5000
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let validation_result = validate_exclude(&json_value, "156F64", &engine);

        assert!(validation_result.is_empty());
    }

    #[test]
    fn validate_invalid_exclude() {
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
            exclude: kekw?
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
                argument: totem.token
            expectations:
              - name: timeout
                expect: facts.corosync_token_timeout == 5000
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let validation_errors = validate_exclude(&json_value, "156F64", &engine);

        assert!(validation_errors.len() == 1);
        match &validation_errors[0] {
            w @ ValidationDiagnostic::Warning { .. } => panic!("Unexpected variant {:?}", w),
            ValidationDiagnostic::Critical {
                check_id,
                message,
                instance_path,
            } => {
                assert_eq!(check_id, "156F64");
                assert_eq!(message, "Unknown operator: '?' (line 1, position 5)");
                assert_eq!(instance_path, "/exclude");
            }
        }
    }

    #[test]
    fn validate_non_string_exclude() {
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
            exclude: true
            facts:
              - name: corosync_token_timeout
                gatherer: corosync.conf
                argument: totem.token
            expectations:
              - name: timeout
                expect: facts.corosync_token_timeout == 5000
        "#;

        let engine = Engine::new();

        let json_value: serde_json::Value =
            serde_yaml::from_str(input).expect("Unable to parse yaml");
        let validation_errors = validate_exclude(&json_value, "156F64", &engine);

        assert!(validation_errors.len() == 1);
        match &validation_errors[0] {
            w @ ValidationDiagnostic::Warning { .. } => panic!("Unexpected variant {:?}", w),
            ValidationDiagnostic::Critical {
                check_id,
                message,
                instance_path,
            } => {
                assert_eq!(check_id, "156F64");
                assert_eq!(message, "exclude must be a string expression");
                assert_eq!(instance_path, "/exclude");
            }
        }
    }
}
