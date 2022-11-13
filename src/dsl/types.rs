use super::validation;
use super::validation::Validate;

#[derive(Debug)]
pub struct Check {
    pub id: String,
    pub name: String,
    pub group: String,
    pub description: String,
    pub remediation: String,
    pub facts: Vec<FactDeclaration>,
    pub expectations: Vec<Expectation>,
}

impl Validate for Check {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut outcomes = vec![];
        outcomes.push(validation::string_not_empty(
            &self.id,
            &format!("{} - id", validation::error_header(&self.id)),
        ));
        outcomes.push(validation::string_not_empty(
            &self.name,
            &format!("{} - name", validation::error_header(&self.id)),
        ));
        outcomes.push(validation::string_not_empty(
            &self.group,
            &format!("{} - group", validation::error_header(&self.id)),
        ));
        outcomes.push(validation::string_not_empty(
            &self.description,
            &format!("{} - description", validation::error_header(&self.id)),
        ));
        outcomes.push(validation::string_not_empty(
            &self.remediation,
            &format!("{} - remediation", validation::error_header(&self.id)),
        ));
        outcomes.push(validation::list_not_empty(
            &self.facts,
            &format!("{} - facts", validation::error_header(&self.id)),
        ));
        outcomes.push(validation::list_not_empty(
            &self.expectations,
            &format!("{} - expectations", validation::error_header(&self.id)),
        ));

        let _ = &self.facts.iter().for_each(|fact| {
            let fact_validation_result = match fact.validate() {
                Err(validation_errors) => {
                    let error_string = format!(
                        "{} - facts - {}",
                        validation::error_header(&self.id),
                        validation_errors.join(" - ")
                    );
                    Err(error_string)
                }
                Ok(result) => Ok(result),
            };
            outcomes.push(fact_validation_result);
        });

        let (_, failed_validations): (Vec<_>, Vec<_>) =
            outcomes.into_iter().partition(Result::is_ok);

        let errors: Vec<String> = failed_validations
            .into_iter()
            .map(Result::unwrap_err)
            .collect();

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }
}

#[derive(Debug)]
pub struct FactDeclaration {
    pub fact_name: String,
    pub gatherer: String,
    pub arguments: Vec<String>,
}

impl Validate for FactDeclaration {
    fn validate(&self) -> Result<(), Vec<String>> {
        let outcomes = vec![
            validation::string_not_empty(&self.fact_name, "fact_name"),
            validation::string_not_empty(&self.gatherer, "gatherer"),
            validation::list_not_empty(&self.arguments, "arguments"),
        ];

        let (_, failed_validations): (Vec<_>, Vec<_>) =
            outcomes.into_iter().partition(Result::is_ok);

        let errors: Vec<String> = failed_validations
            .into_iter()
            .map(Result::unwrap_err)
            .collect();

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }
}

#[derive(Debug)]
pub struct Fact {
    pub name: String,
    pub content: String,
}

#[derive(Debug)]
pub struct Expectation {
    pub name: String,
    pub variable: String,
    pub predicate: Predicate,
}

#[derive(Debug)]
pub enum Predicate {
    String,
    Bool,
    U64,
}

#[derive(Debug)]
pub struct ParsingError {
    pub check_id: String,
    pub error: String,
}
