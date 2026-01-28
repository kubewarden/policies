use lazy_static::lazy_static;

use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{logging, protocol_version_guest, validate_settings};

mod request;
mod settings;
use request::RawValidationRequest;
use settings::Settings;

use slog::{Logger, info, o};

lazy_static! {
    static ref LOG_DRAIN: Logger = Logger::root(
        logging::KubewardenDrain::new(),
        o!("policy" => "raw-validation-policy")
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: RawValidationRequest =
        if let Ok(validation_request) = serde_json::from_slice(payload) {
            validation_request
        } else {
            return kubewarden::reject_request(
                Some("cannot unmarshal request".to_string()),
                None,
                None,
                None,
            );
        };

    info!(LOG_DRAIN, "starting validation");

    let request = validation_request.request;
    let settings = validation_request.settings;

    if settings.valid_users.contains(&request.user)
        && settings.valid_actions.contains(&request.action)
        && settings.valid_resources.contains(&request.resource)
    {
        info!(LOG_DRAIN, "accepting resource");
        kubewarden::accept_request()
    } else {
        kubewarden::reject_request(
            Some("this request is not accepted".to_string()),
            None,
            None,
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden::response::ValidationResponse;
    use kubewarden_policy_sdk::test::Testcase;

    #[test]
    fn accept_valid_request() {
        let request_file = "test_data/valid.json";
        let test_case = Testcase {
            name: String::from("Valid request"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {
                valid_users: vec!["tonio".to_string(), "wanda".to_string()],
                valid_actions: vec!["eats".to_string(), "likes".to_string()],
                valid_resources: vec!["banana".to_string(), "hay".to_string()],
            },
        };

        let response = test_case.eval(validate).unwrap();
        assert!(
            response.mutated_object.is_none(),
            "Something mutated with test case: {}",
            test_case.name,
        );
    }

    #[test]
    fn reject_invalid_request() {
        let request_file = "test_data/invalid.json";
        let test_case = Testcase {
            name: String::from("Invalid request"),
            fixture_file: String::from(request_file),
            expected_validation_result: false,
            settings: Settings {
                valid_users: vec!["tonio".to_string(), "wanda".to_string()],
                valid_actions: vec!["eats".to_string(), "likes".to_string()],
                valid_resources: vec!["banana".to_string(), "hay".to_string()],
            },
        };

        let response = test_case.eval(validate).unwrap();
        assert!(
            response.mutated_object.is_none(),
            "Something mutated with test case: {}",
            test_case.name,
        );
    }

    #[test]
    fn reject_invalid_payload() {
        let payload = r#"{"invalid": "payload"}"#;

        let raw_result = validate(payload.as_bytes()).unwrap();
        let response: ValidationResponse = serde_json::from_slice(&raw_result).unwrap();

        assert!(!response.accepted);
        assert!(response.message == Some("cannot unmarshal request".to_owned()));
        assert!(response.mutated_object.is_none());
    }
}
