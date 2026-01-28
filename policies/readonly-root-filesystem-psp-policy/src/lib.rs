use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::api::core::v1 as apicore;

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

#[derive(Debug, PartialEq)]
enum PolicyResponse {
    Accept,
    Reject(String),
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;

    let pod = match serde_json::from_value::<apicore::Pod>(validation_request.request.object) {
        Ok(pod) => pod,
        Err(_) => return kubewarden::accept_request(),
    };

    match do_validate(&pod) {
        PolicyResponse::Accept => kubewarden::accept_request(),
        PolicyResponse::Reject(msg) => kubewarden::reject_request(Some(msg), None, None, None),
    }
}

fn do_validate(pod: &apicore::Pod) -> PolicyResponse {
    if pod.spec.is_none() {
        return PolicyResponse::Accept;
    }

    let pod_spec = pod.spec.clone().unwrap();

    let init_containers_do_not_have_readonly_filesystem = match pod_spec.init_containers {
        Some(ic) => does_not_have_readonly_root_filesystem(&ic),
        None => false,
    };

    let containers_have_readonly_disabled =
        does_not_have_readonly_root_filesystem(&pod_spec.containers);

    let mut errors = vec![];
    if init_containers_do_not_have_readonly_filesystem {
        errors.push("One of the init containers does not have readOnlyRootFilesystem enabled");
    }
    if containers_have_readonly_disabled {
        errors.push("One of the containers does not have readOnlyRootFilesystem enabled");
    }

    if errors.is_empty() {
        PolicyResponse::Accept
    } else {
        PolicyResponse::Reject(errors.join(", "))
    }
}

fn does_not_have_readonly_root_filesystem(containers: &[apicore::Container]) -> bool {
    containers
        .iter()
        .any(|c| match c.security_context.as_ref() {
            Some(sc) => !sc.read_only_root_filesystem.unwrap_or_default(),
            None => true,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_pod_with_container_with_readonly_root() {
        let pod = apicore::Pod {
            spec: Some(apicore::PodSpec {
                containers: vec![apicore::Container {
                    name: "nginx".to_string(),
                    image: Some("nginx".to_string()),
                    security_context: Some(apicore::SecurityContext {
                        read_only_root_filesystem: Some(true),
                        ..apicore::SecurityContext::default()
                    }),
                    ..apicore::Container::default()
                }],
                ..apicore::PodSpec::default()
            }),
            ..apicore::Pod::default()
        };

        let actual = do_validate(&pod);
        assert_eq!(PolicyResponse::Accept, actual);
    }

    #[test]
    fn accept_pod_with_init_container_with_readonly_root() {
        let pod = apicore::Pod {
            spec: Some(apicore::PodSpec {
                init_containers: Some(vec![apicore::Container {
                    name: "init".to_string(),
                    image: Some("alpine".to_string()),
                    security_context: Some(apicore::SecurityContext {
                        read_only_root_filesystem: Some(true),
                        ..apicore::SecurityContext::default()
                    }),
                    ..apicore::Container::default()
                }]),
                containers: vec![apicore::Container {
                    name: "nginx".to_string(),
                    image: Some("nginx".to_string()),
                    security_context: Some(apicore::SecurityContext {
                        read_only_root_filesystem: Some(true),
                        ..apicore::SecurityContext::default()
                    }),
                    ..apicore::Container::default()
                }],
                ..apicore::PodSpec::default()
            }),
            ..apicore::Pod::default()
        };

        let actual = do_validate(&pod);
        assert_eq!(PolicyResponse::Accept, actual);
    }

    #[test]
    fn reject_pod_with_container_with_writable_root() {
        let pod = apicore::Pod {
            spec: Some(apicore::PodSpec {
                containers: vec![
                    apicore::Container {
                        name: "nginx".to_string(),
                        image: Some("nginx".to_string()),
                        security_context: Some(apicore::SecurityContext {
                            read_only_root_filesystem: Some(true),
                            ..apicore::SecurityContext::default()
                        }),
                        ..apicore::Container::default()
                    },
                    apicore::Container {
                        name: "db".to_string(),
                        image: Some("mariadb".to_string()),
                        // no security_context means root fs is writable
                        ..apicore::Container::default()
                    },
                ],
                ..apicore::PodSpec::default()
            }),
            ..apicore::Pod::default()
        };

        let actual = do_validate(&pod);
        assert_eq!(
            PolicyResponse::Reject(
                "One of the containers does not have readOnlyRootFilesystem enabled".to_string()
            ),
            actual
        );
    }

    #[test]
    fn reject_pod_with_init_container_with_writable_root() {
        let pod = apicore::Pod {
            spec: Some(apicore::PodSpec {
                init_containers: Some(vec![apicore::Container {
                    name: "init".to_string(),
                    image: Some("alpine".to_string()),
                    security_context: Some(apicore::SecurityContext {
                        read_only_root_filesystem: Some(false),
                        ..apicore::SecurityContext::default()
                    }),
                    ..apicore::Container::default()
                }]),
                containers: vec![apicore::Container {
                    name: "nginx".to_string(),
                    image: Some("nginx".to_string()),
                    security_context: Some(apicore::SecurityContext {
                        read_only_root_filesystem: Some(true),
                        ..apicore::SecurityContext::default()
                    }),
                    ..apicore::Container::default()
                }],
                ..apicore::PodSpec::default()
            }),
            ..apicore::Pod::default()
        };

        let actual = do_validate(&pod);
        assert_eq!(
            PolicyResponse::Reject(
                "One of the init containers does not have readOnlyRootFilesystem enabled"
                    .to_string()
            ),
            actual
        );
    }
}
