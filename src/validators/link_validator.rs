use crate::dsl::types::ValidationDiagnostic;
use crate::dsl::types::Validator;

#[cfg(not(target_arch = "wasm32"))]
use async_compat::Compat;
#[cfg(not(target_arch = "wasm32"))]
use lychee_lib::extract::Extractor;
#[cfg(not(target_arch = "wasm32"))]
use lychee_lib::{ErrorKind, FileType, InputContent, Response};

pub struct LinkValidator {}

impl Validator for LinkValidator {
    #[cfg(not(target_arch = "wasm32"))]
    fn validate(
        &self,
        json_check: &serde_json::Value,
        check_id: &str,
    ) -> Vec<ValidationDiagnostic> {
        let extractor = Extractor::default();
        let remediation = json_check
            .get("remediation")
            .map_or_else(|| String::new(), |v| v.to_string())
            .replace("\\n", " ");
        let content = InputContent::from_string(&remediation, FileType::Markdown);
        let remediation_links = extractor.extract(&content);

        let description = json_check
            .get("description")
            .map_or_else(|| String::new(), |v| v.to_string())
            .replace("\\n", " ");
        let content = InputContent::from_string(&description, FileType::Markdown);
        let description_links = extractor.extract(&content);

        let links = vec![remediation_links, description_links].concat();

        let link_check = smol::block_on(Compat::new(async {
            let mut checked = Vec::<Result<Response, ErrorKind>>::new();

            for url in links {
                let url = url.text;
                checked.push(lychee_lib::check(url).await);
            }

            checked
        }));

        let mut diagnostics = Vec::<ValidationDiagnostic>::new();

        for link_check in link_check {
            match link_check {
                Err(e) => diagnostics.push(ValidationDiagnostic::Critical {
                    check_id: check_id.to_string(),
                    message: format!("Failed to validate link in check: {}", e.to_string()),
                    instance_path: "N/A".to_owned(),
                }),
                Ok(r) => {
                    if !r.status().is_success() {
                        let details = r.status().details().unwrap_or_else(|| {
                            if r.status().is_unsupported() {
                                "Unsupported Format".to_owned()
                            } else {
                                r.status().code_as_string()
                            }
                        });

                        diagnostics.push(ValidationDiagnostic::Warning {
                            check_id: check_id.to_string(),
                            message: format!(
                                "Invalid link ({}): {}",
                                r.source().to_string(),
                                details
                            ),
                            instance_path: "N/A".to_owned(),
                        });
                    }
                }
            };
        }

        diagnostics
    }

    #[cfg(any(target_arch = "wasm32"))]
    fn validate(
        &self,
        json_check: &serde_json::Value,
        check_id: &str,
    ) -> Vec<ValidationDiagnostic> {
        _ = json_check;
        _ = check_id;
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_ok_check() {
        let input = r#"
            id: 156F64
            name: Corosync configuration file
            group: Corosync
            description: |
              Link to https://google.com, because the example dot com domain is excluded
            remediation: |
              ## Abstract
              The value of the Corosync `token` timeout is not set as recommended.
              Link to https://google.com, because the example dot com domain is excluded
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

        let validator = LinkValidator {};
        let validation_result = validator.validate(&json_value, "156F64");

        assert!(validation_result.is_empty());
    }

    #[test]
    fn validate_malformed_link_in_description() {
        let input = r#"
            id: 156F64
            name: Corosync configuration file
            group: Corosync
            description: |
              Corosync `token` timeout is set to expected value
              Link to https://google.com/404, because the example dot com domain is excluded
            remediation: |
              ## Abstract
              The value of the Corosync `token` timeout is not set as recommended.
              ## Remediation
              Link to https://google.com/404, because the example dot com domain is excluded
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

        let validator = LinkValidator {};
        let validation_result = validator.validate(&json_value, "156F64");

        assert_eq!(validation_result.len(), 2);
    }
}
