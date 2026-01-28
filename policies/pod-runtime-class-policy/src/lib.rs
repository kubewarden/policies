use guest::prelude::*;
use k8s_openapi::api::core::v1::PodSpec;
use kubewarden_policy_sdk::wapc_guest as guest;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{protocol_version_guest, request::ValidationRequest, validate_settings};

use kubewarden::{accept_request, mutate_pod_spec_from_request, reject_request};

mod settings;
use settings::Settings;

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn mutate_or_reject(
    validation_request: ValidationRequest<Settings>,
    pod_spec: &PodSpec,
    error_message: String,
) -> CallResult {
    if let Some(ref fallback_runtime) = validation_request.settings.fallback_runtime {
        let mut new_pod_spec = pod_spec.clone();
        new_pod_spec.runtime_class_name = Some(fallback_runtime.to_string());
        return mutate_pod_spec_from_request(validation_request, new_pod_spec);
    }
    reject_request(Some(error_message), None, None, None)
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;
    let pod = validation_request.extract_pod_spec_from_object()?;
    if let Some(pod_spec) = pod {
        if let Some(ref runtime_class_name) = pod_spec.runtime_class_name
            && validation_request
                .settings
                .reserved_runtimes
                .contains(runtime_class_name)
        {
            return mutate_or_reject(
                validation_request,
                &pod_spec,
                format!("runtime '{runtime_class_name}' is reserved"),
            );
        }

        // The object does not define the runtime. Therefore, the default one will
        // be used. Update the object to use the fallback runtime.
        if let Some(default_runtime_reserved) = validation_request.settings.default_runtime_reserved
            && default_runtime_reserved
        {
            return mutate_or_reject(
                validation_request,
                &pod_spec,
                "Usage of the default runtime is reserved".to_string(),
            );
        }
        // The default runtime is not reserved, but a fallback is specified. Therefore
        // the Pod is mutated to use the fallback runtime.
        if let Some(ref fallback_runtime) = validation_request.settings.fallback_runtime {
            let mut new_pod_spec = pod_spec;
            new_pod_spec.runtime_class_name = Some(fallback_runtime.clone());
            return mutate_pod_spec_from_request(validation_request, new_pod_spec);
        }
    }
    accept_request()
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::Resource;
    use k8s_openapi::api::core::v1::{Pod, PodSpec};
    use kubewarden::request::{KubernetesAdmissionRequest, ValidationRequest};
    use kubewarden_policy_sdk::request::GroupVersionKind;
    use kubewarden_policy_sdk::response::ValidationResponse;
    use rstest::*;
    use std::collections::HashSet;

    fn create_settings(
        reserved_runtimes: HashSet<String>,
        default_runtime_reserved: bool,
        fallback_runtime: Option<String>,
    ) -> Settings {
        Settings {
            reserved_runtimes,
            default_runtime_reserved: Some(default_runtime_reserved),
            fallback_runtime,
        }
    }

    fn create_admission_request(runtime_class_name: Option<String>) -> KubernetesAdmissionRequest {
        let pod = serde_json::to_value(Pod {
            spec: Some(PodSpec {
                runtime_class_name,
                ..Default::default()
            }),
            ..Default::default()
        })
        .expect("Cannot serialize Pod spec");
        KubernetesAdmissionRequest {
            kind: GroupVersionKind {
                kind: Pod::KIND.to_string(),
                ..Default::default()
            },
            object: pod,
            ..Default::default()
        }
    }

    #[rstest]
    #[case::accept_request_not_using_reserved_runtimes(
        create_settings(HashSet::from([]), false, None),
        create_admission_request(Some("runC".to_string())),
        true,
        None,
        None
    )]
    #[case::reject_request_using_reserved_runtimes(
        create_settings(HashSet::from(["runC".to_string()]), false, None),
        create_admission_request(Some("runC".to_string())),
        false,
        None,
        None
    )]
    #[case::reject_request_using_reserved_default_runtime_with_no_fallback(
        create_settings(HashSet::from([]), true, None),
        create_admission_request(None),
        false,
        Some("Usage of the default runtime is reserved".to_string()),
        None
    )]
    #[case::mutate_request_using_reserved_runtimes_when_fallback_is_defined(
        create_settings(HashSet::from(["runC".to_string()]), false, Some("fallback".to_string())),
        create_admission_request(Some("runC".to_string())),
        true,
        None,
        Some("fallback".to_string())
    )]
    #[case::mutate_request_using_default_runtime_and_default_is_reserved(
        create_settings(HashSet::from([]), true, Some("fallback".to_string())),
        create_admission_request(None),
        true,
        None,
        Some("fallback".to_string())
    )]
    #[case::mutate_request_using_default_runtime_and_fallback_is_defined(
        create_settings(HashSet::from([]), false, Some("fallback".to_string())),
        create_admission_request(None),
        true,
        None,
        Some("fallback".to_string())
    )]
    fn validate_accept_reject_behaviour(
        #[case] settings: Settings,
        #[case] request: KubernetesAdmissionRequest,
        #[case] expected_accept_result: bool,
        #[case] expected_error_message: Option<String>,
        #[case] expected_new_runtime: Option<String>,
    ) {
        let validation_request = ValidationRequest::<Settings> { settings, request };
        let payload = serde_json::to_string(&validation_request).expect("Cannot serialize payload");

        let response: ValidationResponse =
            serde_json::from_slice(&validate(payload.as_bytes()).expect("Validation failed"))
                .expect("Cannot parse response JSON");
        assert_eq!(response.accepted, expected_accept_result);
        if let Some(expected_error_message) = expected_error_message {
            if let Some(message) = response.message {
                assert_eq!(message, expected_error_message)
            }
        }
        if let Some(expected_mutated_runtime) = expected_new_runtime {
            if let Some(mutated_pod_value) = response.mutated_object {
                let mutated_pod: Pod = serde_json::from_value(mutated_pod_value)
                    .expect("Cannot parse the mutated object");
                assert_eq!(
                    mutated_pod
                        .spec
                        .unwrap_or_default()
                        .runtime_class_name
                        .unwrap_or_default(),
                    expected_mutated_runtime
                );
            }
        }
    }
}
