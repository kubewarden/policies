use guest::prelude::*;
use k8s_openapi::api::core::v1 as apicore;
use kubewarden_policy_sdk::wapc_guest as guest;
use lazy_static::lazy_static;
extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{logging, protocol_version_guest, request::ValidationRequest, validate_settings};
use slog::{Logger, error, o, warn};

use settings::Settings;
mod settings;

#[derive(PartialEq, Debug)]
enum PodSpecMutationState {
    Mutated,
    NotMutated,
}

lazy_static! {
    static ref LOG_DRAIN: Logger = Logger::root(
        logging::KubewardenDrain::new(),
        o!("policy" => "priority-class-policy")
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;
    match validation_request.extract_pod_spec_from_object() {
        Ok(Some(mut pod_spec)) => {
            match validate_pod_priority_class(&mut pod_spec, &validation_request.settings) {
                Ok(PodSpecMutationState::NotMutated) => kubewarden::accept_request(),
                Ok(PodSpecMutationState::Mutated) => {
                    kubewarden::mutate_pod_spec_from_request(validation_request, pod_spec)
                }
                Err(err) => kubewarden::reject_request(Some(err.to_owned()), None, None, None),
            }
        }
        Ok(None) => {
            warn!(LOG_DRAIN, "no PodSpec found");
            kubewarden::accept_request()
        }
        Err(err) => {
            error!(LOG_DRAIN, "Priority class policy failed to extract PodSpec from the request"; "err" => %err);
            kubewarden::reject_request(
                Some(format!(
                    "Priority class policy failed to extract PodSpec from the request : {err}"
                )),
                None,
                None,
                None,
            )
        }
    }
}

fn validate_pod_priority_class(
    pod: &mut apicore::PodSpec,
    settings: &Settings,
) -> Result<PodSpecMutationState, String> {
    if pod.priority_class_name.is_none() && settings.default_priority_class.is_none() {
        return Ok(PodSpecMutationState::NotMutated);
    }
    if pod.priority_class_name.is_none() && settings.default_priority_class.is_some() {
        pod.priority_class_name = settings.default_priority_class.clone();
        return Ok(PodSpecMutationState::Mutated);
    }
    let priority_class_name = pod.priority_class_name.as_ref().unwrap();

    if !settings.denied_priority_classes.is_empty()
        && settings
            .denied_priority_classes
            .contains(priority_class_name)
    {
        return Err(format!(
            "Priority class \"{priority_class_name}\" is denied"
        ));
    }

    if !settings.allowed_priority_classes.is_empty()
        && !settings
            .allowed_priority_classes
            .contains(priority_class_name)
    {
        return Err(format!(
            "Priority class \"{priority_class_name}\" is not allowed"
        ));
    }

    Ok(PodSpecMutationState::NotMutated)
}

#[cfg(test)]
mod tests {
    use super::Settings;
    use super::validate_pod_priority_class;
    use super::*;
    use rstest::rstest;

    use k8s_openapi::api::core::v1::PodSpec;

    #[rstest]
    #[case::pod_priority_allowed(Some("low-priority"), &["high-priority", "low-priority"], &[],None, true)]
    #[case::pod_priority_not_allowed(Some("no-priority"), &["high-priority", "low-priority"],&[], None, false)]
    #[case::pod_priority_not_blocked(Some("no-priority"), &[], &["high-priority", "low-priority"], None, true)]
    #[case::pod_priority_blocked(Some("low-priority"), &[], &["high-priority", "low-priority"], None, false)]
    #[case::pod_priority_missing_with_no_default_class(None, &["high-priority", "low-priority"],&[], None, true)]
    // Tests with default priority class
    #[case::pod_priority_allowed_with_default(Some("high-priority"), &["high-priority", "low-priority"],&[], Some("low-priority"), true)]
    #[case::pod_priority_not_allowed_with_default(Some("no-priority"), &["high-priority", "low-priority"],&[], Some("low-priority"), false)]
    #[case::pod_priotiry_not_blocked_with_default(Some("other-priority"), &[], &["high-priority", "low-priority"],Some("no-priority"), true)]
    #[case::pod_priority_blocked_with_default(Some("low-priority"), &[], &["high-priority", "low-priority"], Some("no-priority"), false)]
    fn test_pod_validation(
        #[case] pod_priority_class: Option<&str>,
        #[case] allowed_classes: &[&str],
        #[case] denied_classes: &[&str],
        #[case] default_priority_class: Option<&str>,
        #[case] should_succeed: bool,
    ) {
        let mut pod = PodSpec {
            priority_class_name: pod_priority_class.map(|s| s.to_owned()).clone(),
            ..Default::default()
        };
        let settings = Settings {
            allowed_priority_classes: allowed_classes.iter().map(|s| s.to_string()).collect(),
            denied_priority_classes: denied_classes.iter().map(|s| s.to_string()).collect(),
            default_priority_class: default_priority_class.map(|s| s.to_owned()).clone(),
        };
        let result = validate_pod_priority_class(&mut pod, &settings);
        assert_eq!(result.is_ok(), should_succeed);
        if !should_succeed {
            return;
        }
        assert_eq!(result.unwrap(), PodSpecMutationState::NotMutated);
    }

    #[rstest]
    #[case::missing_pod_priority_with_allow_list(&["high-priority", "low-priority"],&[], Some("low-priority"))]
    #[case::missing_pod_priority_with_deny_list(&[], &["high-priority", "low-priority"],Some("no-priority"))]
    fn test_pod_mutation(
        #[case] allowed_classes: &[&str],
        #[case] denied_classes: &[&str],
        #[case] default_priority_class: Option<&str>,
    ) {
        let mut pod = PodSpec {
            priority_class_name: None,
            ..Default::default()
        };
        let expected_default_priority_class = default_priority_class.map(|s| s.to_owned()).clone();
        let settings = Settings {
            allowed_priority_classes: allowed_classes.iter().map(|s| s.to_string()).collect(),
            denied_priority_classes: denied_classes.iter().map(|s| s.to_string()).collect(),
            default_priority_class: expected_default_priority_class.clone(),
        };
        let result = validate_pod_priority_class(&mut pod, &settings);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PodSpecMutationState::Mutated);
        assert_eq!(pod.priority_class_name, expected_default_priority_class);
    }
}
