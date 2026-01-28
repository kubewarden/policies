use anyhow::anyhow;
use kubewarden_policy_sdk::wapc_guest::prelude::*;
use std::convert::TryFrom;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{protocol_version_guest, validate_settings};

mod settings;
use settings::Settings;

enum Operation {
    Create,
    Update,
    Delete,
    Connect,
}

impl Operation {
    fn annotation_expr(&self) -> jmespath::Expression<'_> {
        match self {
            Operation::Create => jmespath::compile("request.object.metadata.annotations")
                .expect("create jmespath should not fail"),
            Operation::Update => jmespath::compile("request.oldObject.metadata.annotations")
                .expect("update jmespath should not fail"),
            Operation::Delete => jmespath::compile("request.oldObject.metadata.annotations")
                .expect("delete jmespath should not fail"),
            Operation::Connect => jmespath::compile("request.object.metadata.annotations")
                .expect("connect jmespath should not fail"),
        }
    }

    fn annotation_key(&self) -> &str {
        match self {
            Operation::Create => "io.kubewarden.policy.echo.create",
            Operation::Update => "io.kubewarden.policy.echo.update",
            Operation::Delete => "io.kubewarden.policy.echo.delete",
            Operation::Connect => "io.kubewarden.policy.echo.connect",
        }
    }
}

impl TryFrom<&String> for Operation {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "CREATE" => Ok(Operation::Create),
            "UPDATE" => Ok(Operation::Update),
            "DELETE" => Ok(Operation::Delete),
            "CONNECT" => Ok(Operation::Connect),
            _ => Err(format!("unknown type of operation: {value}")),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn validate(payload: &[u8]) -> CallResult {
    let request_expr = jmespath::compile("request")?;
    let operation_expr = jmespath::compile("request.operation")?;

    let json = std::str::from_utf8(payload)?;
    let data = jmespath::Variable::from_json(json)?;

    let operation = Operation::try_from(
        operation_expr
            .search(data.clone())?
            .as_string()
            .ok_or_else(|| anyhow!("jmespath didn't return a string"))?,
    )?;

    // Search the data with the compiled expression
    let result = operation.annotation_expr().search(data.clone())?;
    if result.is_object() {
        let annotations = result.as_object().unwrap();
        if annotations.contains_key(operation.annotation_key()) {
            let request = request_expr.search(data)?;

            return kubewarden::reject_request(Some(request.to_string()), Some(418), None, None);
        }
    }
    kubewarden::accept_request()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_include;
    use kubewarden_policy_sdk::test::Testcase;
    use std::fs;

    #[test]
    fn accept() {
        let test_cases = vec![
            Testcase {
                name: String::from("create"),
                fixture_file: String::from("test_data/create_accept.json"),
                expected_validation_result: true,
                settings: Settings {},
            },
            Testcase {
                name: String::from("update"),
                fixture_file: String::from("test_data/update_accept.json"),
                expected_validation_result: true,
                settings: Settings {},
            },
            Testcase {
                name: String::from("delete"),
                fixture_file: String::from("test_data/delete_accept.json"),
                expected_validation_result: true,
                settings: Settings {},
            },
        ];

        for tc in &test_cases {
            let res = tc
                .eval(validate)
                .expect("Validation should not raise errors");
            assert!(
                res.mutated_object.is_none(),
                "Something mutated with test case: {}",
                tc.name,
            );
        }
    }

    #[test]
    fn reject() {
        let test_cases = vec![
            Testcase {
                name: String::from("create"),
                fixture_file: String::from("test_data/create_reject.json"),
                expected_validation_result: false,
                settings: Settings {},
            },
            Testcase {
                name: String::from("update"),
                fixture_file: String::from("test_data/update_reject.json"),
                expected_validation_result: false,
                settings: Settings {},
            },
            Testcase {
                name: String::from("delete"),
                fixture_file: String::from("test_data/delete_reject.json"),
                expected_validation_result: false,
                settings: Settings {},
            },
        ];

        for tc in &test_cases {
            let res = tc
                .eval(validate)
                .expect("Validation should not raise errors");
            assert!(
                res.mutated_object.is_none(),
                "Something mutated with test case: {}",
                tc.name,
            );
            assert!(res.message.is_some());

            let expected_json: serde_json::Value =
                serde_json::from_str(fs::read_to_string(&tc.fixture_file).unwrap().as_str())
                    .unwrap();

            let request: serde_json::Value = serde_json::from_str(&res.message.unwrap())
                .expect("response message should contain a valid JSON document");

            assert_json_include!(actual: request, expected: expected_json);
        }
    }
}
