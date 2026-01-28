use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::api::core::v1::PodSpec;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::Settings;

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn is_pod_sharing_pid_namespace(pod_spec: PodSpec) -> bool {
    if let Some(is_sharing_pid_namespace) = pod_spec.share_process_namespace {
        return is_sharing_pid_namespace;
    }
    false
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;
    match validation_request.extract_pod_spec_from_object() {
        Ok(pod_spec) => {
            if is_pod_sharing_pid_namespace(pod_spec.unwrap_or_default()) {
                return kubewarden::reject_request(
                    Some("pod cannot share host PID namespace".to_owned()),
                    None,
                    None,
                    None,
                );
            }
            kubewarden::accept_request()
        }
        Err(_) => kubewarden::reject_request(
            Some("cannot parse validation request".to_owned()),
            None,
            None,
            None,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(PodSpec{
        ..Default::default()
    }, false)]
    #[case(PodSpec{
        share_process_namespace:Some(false),
        ..Default::default()
    }, false)]
    #[case(PodSpec{
        share_process_namespace:Some(true),
        ..Default::default()
    }, true)]
    #[case(PodSpec{
        share_process_namespace:None,
        ..Default::default()
    }, false)]
    fn test_pod_spec_sharing_pid_namespace(
        #[case] pod_spec: PodSpec,
        #[case] expected_validation_result: bool,
    ) {
        let validation_result = is_pod_sharing_pid_namespace(pod_spec);
        assert_eq!(validation_result, expected_validation_result)
    }
}
