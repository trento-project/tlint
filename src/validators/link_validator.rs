use crate::dsl::types::ValidationDiagnostic;
use crate::dsl::types::Validator;
use async_compat::Compat;
use lychee_lib::extract::Extractor;
use lychee_lib::{ErrorKind, FileType, InputContent, Response};

pub struct LinkValidator {}

impl Validator for LinkValidator {
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
                                "N/A".to_owned()
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
}
