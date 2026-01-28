use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use anyhow::{Result, anyhow};
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

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum PolicyResponse {
    Accept,
    Reject(String),
}

/* Trait to abstract apicore::Container and apicore::EphemeralContainer
 * This is used to allow using the same generic function to validate the
 * security context from both structs.
 */
trait Container {
    fn security_context(&self) -> Option<apicore::SecurityContext>;
}

impl Container for apicore::Container {
    fn security_context(&self) -> Option<apicore::SecurityContext> {
        self.security_context.clone()
    }
}

impl Container for apicore::EphemeralContainer {
    fn security_context(&self) -> Option<apicore::SecurityContext> {
        self.security_context.clone()
    }
}

/* Validates the pod annotations. The seccomp annotations should be defined in
 * the settings.
 */
fn do_validate_annotations(pod: &apicore::Pod, settings: &settings::Settings) -> Result<()> {
    if let Some(annotations) = &pod.metadata.annotations {
        let mut invalid_profiles = vec![];
        for (annotation, profile) in annotations {
            if (annotation.starts_with("container.seccomp.security.alpha.kubernetes.io")
                || annotation == "seccomp.security.alpha.kubernetes.io/pod")
                && !settings.allowed_profiles.contains(profile)
            {
                invalid_profiles.push(profile.clone())
            }
        }
        if !invalid_profiles.is_empty() {
            return Err(anyhow!(
                "Seccomp profiles '{}' are not allowed.",
                invalid_profiles.join(",")
            ));
        }
    }
    Ok(())
}

/* Function used to check if the seccomp profile type is defined in the
 * allowed_profiles settings
 */
fn allowed_profiles_has_profile_type(
    settings: &settings::Settings,
    seccomp_profile: &apicore::SeccompProfile,
) -> bool {
    // Unfortunately, we cannot store the allowed_profiles settings parsed to
    // avoid this iteration in every evaluation.
    for profile in &settings.allowed_profiles {
        if (seccomp_profile.type_ == "RuntimeDefault"
            && (profile == "runtime/default" || profile == "docker/default"))
            || seccomp_profile.type_ == "Localhost" && profile.starts_with("localhost/")
            || seccomp_profile.type_ == "Unconfined" && profile == "unconfined"
        {
            return true;
        }
    }
    false
}

/* Function used to check if the seccomp profile localhost profile is defined
 * in the allowed_profiles settings
 */
fn localhost_profile_defined_in_allowed_profiles(
    settings: &settings::Settings,
    seccomp_profile: &apicore::SeccompProfile,
) -> bool {
    // Unfortunately, we cannot store the allowed_profiles settings parsed to
    // avoid this iteration in every evaluation.
    for profile in &settings.allowed_profiles {
        if seccomp_profile.type_ == "Localhost"
            && profile.starts_with("localhost/")
            && let Some(localhost_profile) = &seccomp_profile.localhost_profile
            && profile.ends_with(localhost_profile)
        {
            return true;
        }
    }
    false
}

/* Validates the Seccomp configuration of the given containers.
 * Checks the type values and if the localhost profile is set when necessary
 */
fn do_validate_containers<C>(containers: &[C], settings: &settings::Settings) -> Result<()>
where
    C: Container,
{
    let mut invalid_seccomp_profile_types = vec![];
    let mut invalid_seccomp_profile = vec![];
    for container in containers {
        if let Some(security_context) = container.security_context()
            && let Some(seccomp_profile) = security_context.seccomp_profile.clone()
        {
            if !settings.profile_types.contains(&seccomp_profile.type_)
                && !allowed_profiles_has_profile_type(settings, &seccomp_profile)
            {
                invalid_seccomp_profile_types.push(seccomp_profile.type_.clone())
            }
            if seccomp_profile.type_ == "Localhost" {
                match &seccomp_profile.localhost_profile {
                    Some(localhost_profile) => {
                        if !settings.localhost_profiles.contains(localhost_profile)
                            && !localhost_profile_defined_in_allowed_profiles(
                                settings,
                                &seccomp_profile,
                            )
                        {
                            invalid_seccomp_profile.push(localhost_profile.clone())
                        }
                    }
                    None => {
                        return Err(anyhow!(
                            "The container localhost seccomp profile must be set.".to_string(),
                        ));
                    }
                }
            }
        }
    }
    let mut violations = vec![];
    if !invalid_seccomp_profile_types.is_empty() {
        violations.push(format!(
            "Invalid container seccomp profile types: {}",
            invalid_seccomp_profile_types.join(",")
        ));
    }
    if !invalid_seccomp_profile.is_empty() {
        violations.push(format!(
            "Invalid container seccomp profile: {}",
            invalid_seccomp_profile.join(",")
        ));
    }
    if violations.is_empty() {
        return Ok(());
    }
    Err(anyhow!(violations.join("; ")))
}

fn do_validate_pod_security_context(
    pod: &apicore::PodSpec,
    settings: &settings::Settings,
) -> Result<()> {
    if let Some(security_context) = &pod.security_context
        && let Some(seccomp_profile) = security_context.seccomp_profile.clone()
    {
        if !settings.profile_types.contains(&seccomp_profile.type_)
            && !allowed_profiles_has_profile_type(settings, &seccomp_profile)
        {
            return Err(anyhow!(format!(
                "Invalid podspec seccomp profile types: {}",
                &seccomp_profile.type_
            )));
        }
        if seccomp_profile.type_ == "Localhost" {
            match &seccomp_profile.localhost_profile {
                Some(localhost_profile) => {
                    if !settings.localhost_profiles.contains(localhost_profile)
                        && !localhost_profile_defined_in_allowed_profiles(
                            settings,
                            &seccomp_profile,
                        )
                    {
                        return Err(anyhow!(format!(
                            "Invalid podspec seccomp profile: {}",
                            &localhost_profile
                        )));
                    }
                }
                None => {
                    return Err(anyhow!(
                        "The podspec localhost seccomp profile must be set.".to_string(),
                    ));
                }
            }
        }
    }
    Ok(())
}

/* Do the pod validations. Checks pod annotations and containers Seccomp
 * security context configurations.
 */
fn do_validate(pod: &apicore::Pod, settings: &settings::Settings) -> Result<PolicyResponse> {
    let mut violations: Vec<String> = vec![];
    if let Err(error) = do_validate_annotations(pod, settings) {
        violations.push(error.to_string());
    }
    if let Some(podspec) = &pod.spec {
        if let Err(error) = do_validate_pod_security_context(podspec, settings) {
            violations.push(error.to_string());
        }

        if let Err(error) = do_validate_containers(&podspec.containers, settings) {
            violations.push(error.to_string());
        }

        if let Some(init_containers) = &podspec.init_containers
            && let Err(error) = do_validate_containers(init_containers, settings)
        {
            violations.push(error.to_string());
        }

        if let Some(ephemeral_containers) = &podspec.ephemeral_containers
            && let Err(error) = do_validate_containers(ephemeral_containers, settings)
        {
            violations.push(error.to_string());
        }
    }
    if violations.is_empty() {
        return Ok(PolicyResponse::Accept);
    }
    Ok(PolicyResponse::Reject(format!(
        "Resource violations: {}",
        violations.join(". ")
    )))
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;
    let pod = serde_json::from_value(validation_request.request.object.clone())
        .map_err(|e| anyhow!("Error deserializing Pod specification: {:?}", e))?;
    let settings = validation_request.settings;

    match do_validate(&pod, &settings).unwrap() {
        PolicyResponse::Reject(msg) => kubewarden::reject_request(Some(msg), None, None, None),
        PolicyResponse::Accept => kubewarden::accept_request(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1 as apimachinery;
    use std::collections::BTreeMap;

    macro_rules! configuration {
        (allowed_profiles: $allowed_profiles:expr, profile_types: $profile_types:expr, localhost_profiles: $localhost_profiles: expr) => {
            &Settings {
                allowed_profiles: $allowed_profiles.split(",").map(String::from).collect(),
                profile_types: $profile_types.split(",").map(String::from).collect(),
                localhost_profiles: $localhost_profiles.split(",").map(String::from).collect(),
            }
        };
    }
    #[test]
    fn pod_annotation_with_allowed_profile() -> Result<()> {
        let mut annotations = BTreeMap::new();
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container1".to_string(),
            "runtime/default".to_string(),
        );
        annotations.insert(
            "seccomp.security.alpha.kubernetes.io/pod".to_string(),
            "runtime/default".to_string(),
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    metadata: apimachinery::ObjectMeta {
                        annotations: Some(annotations.clone()),
                        ..apimachinery::ObjectMeta::default()
                    },
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        annotations.clear();
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container1".to_string(),
            "runtime/default".to_string(),
        );
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container2".to_string(),
            "docker/default".to_string(),
        );
        annotations.insert(
            "seccomp.security.alpha.kubernetes.io/pod".to_string(),
            "runtime/default".to_string(),
        );
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    metadata: apimachinery::ObjectMeta {
                        annotations: Some(annotations.clone()),
                        ..apimachinery::ObjectMeta::default()
                    },
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn pod_annotation_with_disallowed_profile() -> Result<()> {
        let mut annotations = BTreeMap::new();
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container1".to_string(),
            "unconfined".to_string(),
        );
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    metadata: apimachinery::ObjectMeta {
                        annotations: Some(annotations.clone()),
                        ..apimachinery::ObjectMeta::default()
                    },
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Seccomp profiles 'unconfined' are not allowed.".to_string()
            )
        );

        annotations.clear();
        annotations.insert(
            "seccomp.security.alpha.kubernetes.io/pod".to_string(),
            "dummy".to_string(),
        );
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    metadata: apimachinery::ObjectMeta {
                        annotations: Some(annotations.clone()),
                        ..apimachinery::ObjectMeta::default()
                    },
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Seccomp profiles 'dummy' are not allowed.".to_string()
            )
        );

        annotations.clear();
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container1".to_string(),
            "dummy".to_string(),
        );
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    metadata: apimachinery::ObjectMeta {
                        annotations: Some(annotations.clone()),
                        ..apimachinery::ObjectMeta::default()
                    },
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Seccomp profiles 'dummy' are not allowed.".to_string()
            )
        );

        annotations.clear();
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container1".to_string(),
            "dummy".to_string(),
        );
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container2".to_string(),
            "unconfined".to_string(),
        );
        annotations.insert(
            "seccomp.security.alpha.kubernetes.io/pod".to_string(),
            "dummy2".to_string(),
        );
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    metadata: apimachinery::ObjectMeta {
                        annotations: Some(annotations.clone()),
                        ..apimachinery::ObjectMeta::default()
                    },
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Seccomp profiles 'dummy,unconfined,dummy2' are not allowed."
                    .to_string()
            )
        );

        annotations.clear();
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container1".to_string(),
            "runtime/default".to_string(),
        );
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container2".to_string(),
            "unconfined".to_string(),
        );
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    metadata: apimachinery::ObjectMeta {
                        annotations: Some(annotations.clone()),
                        ..apimachinery::ObjectMeta::default()
                    },
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Seccomp profiles 'unconfined' are not allowed.".to_string()
            )
        );

        annotations.clear();
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container1".to_string(),
            "runtime/default".to_string(),
        );
        annotations.insert(
            "container.seccomp.security.alpha.kubernetes.io/container2".to_string(),
            "runtime/default".to_string(),
        );
        annotations.insert(
            "seccomp.security.alpha.kubernetes.io/pod".to_string(),
            "unconfined".to_string(),
        );
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    metadata: apimachinery::ObjectMeta {
                        annotations: Some(annotations.clone()),
                        ..apimachinery::ObjectMeta::default()
                    },
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Seccomp profiles 'unconfined' are not allowed.".to_string()
            )
        );

        Ok(())
    }

    #[test]
    fn container_seccomp_security_context() -> Result<()> {
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "RuntimeDefault".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "RuntimeDefault", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Localhost".to_string(),
                                    localhost_profile: Some("profile".to_string()),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Unconfined".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Unconfined", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "dummy".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile types: dummy".to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "dummy".to_string(),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            },
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "dummy2".to_string(),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            }
                        ],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile types: dummy,dummy2"
                    .to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "Localhost".to_string(),
                                        localhost_profile: Some("dummy".to_string()),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            },
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "Localhost".to_string(),
                                        localhost_profile: Some("dummy2".to_string()),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            }
                        ],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile: dummy,dummy2".to_string()
            )
        );

        Ok(())
    }

    #[test]
    fn seccomp_profile_must_be_set_if_its_type_is_localhost() -> Result<()> {
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Localhost".to_string(),
                                    localhost_profile: None,
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Reject(
                "Resource violations: The container localhost seccomp profile must be set."
                    .to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Localhost".to_string(),
                                    localhost_profile: Some("profile".to_string()),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn init_container_seccomp_security_context() -> Result<()> {
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        init_containers: Some(vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "RuntimeDefault".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "RuntimeDefault", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        init_containers: Some(vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Localhost".to_string(),
                                    localhost_profile: Some("profile".to_string()),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        init_containers: Some(vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Unconfined".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Unconfined", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        init_containers: Some(vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "dummy".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost,RuntimeDefault", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile types: dummy".to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        init_containers: Some(vec![
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "dummy".to_string(),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            },
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "dummy2".to_string(),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            }
                        ]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile types: dummy,dummy2"
                    .to_string()
            )
        );
        Ok(())
    }

    #[test]
    fn ephemeral_container_seccomp_security_context() -> Result<()> {
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        ephemeral_containers: Some(vec![apicore::EphemeralContainer {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "RuntimeDefault".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::EphemeralContainer::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "RuntimeDefault", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        ephemeral_containers: Some(vec![apicore::EphemeralContainer {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Localhost".to_string(),
                                    localhost_profile: Some("profile".to_string()),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::EphemeralContainer::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        ephemeral_containers: Some(vec![apicore::EphemeralContainer {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Unconfined".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::EphemeralContainer::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Unconfined", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        ephemeral_containers: Some(vec![apicore::EphemeralContainer {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "dummy".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::EphemeralContainer::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "RuntimeDefault,Localhost", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile types: dummy".to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        ephemeral_containers: Some(vec![
                            apicore::EphemeralContainer {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "dummy".to_string(),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::EphemeralContainer::default()
                            },
                            apicore::EphemeralContainer {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "dummy2".to_string(),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::EphemeralContainer::default()
                            }
                        ]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "Localhost,Runtime", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile types: dummy,dummy2"
                    .to_string()
            )
        );
        Ok(())
    }

    #[test]
    fn pod_seccomp_security_context() -> Result<()> {
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "RuntimeDefault".to_string(),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "", profile_types: "RuntimeDefault", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "Localhost".to_string(),
                                localhost_profile: Some("profile".to_string()),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "Unconfined".to_string(),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "", profile_types: "Unconfined", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "dummy".to_string(),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "", profile_types: "RuntimeDefault,Localhost", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid podspec seccomp profile types: dummy".to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "Localhost".to_string(),
                                localhost_profile: Some("dummy".to_string()),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "", profile_types: "Localhost", localhost_profiles: "profile"),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid podspec seccomp profile: dummy".to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "Localhost".to_string(),
                                localhost_profile: Some("profile2".to_string()),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "", profile_types: "Localhost", localhost_profiles: "profile,profile2"),
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn error_message_may_has_more_than_one_violation() -> Result<()> {
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "dummy".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }],
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "dummy2".to_string(),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "RuntimeDefault,Localhost", localhost_profiles: "profile"),
            )?,
              PolicyResponse::Reject(
                  "Resource violations: Invalid podspec seccomp profile types: dummy2. Invalid container seccomp profile types: dummy".to_string()
              )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![
                            apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "dummy_type".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        },
                            apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Localhost".to_string(),
                                    localhost_profile: Some("dummy_profile".to_string()),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }
                        ],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default", profile_types: "RuntimeDefault,Localhost", localhost_profiles: "profile"),
            )?,
              PolicyResponse::Reject(
                  "Resource violations: Invalid container seccomp profile types: dummy_type; Invalid container seccomp profile: dummy_profile".to_string()
              )
        );

        Ok(())
    }

    #[test]
    fn security_context_validation_should_not_fail_when_just_allowed_profiles_are_provided()
    -> Result<()> {
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        ephemeral_containers: Some(vec![apicore::EphemeralContainer {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "RuntimeDefault".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::EphemeralContainer::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "docker/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        ephemeral_containers: Some(vec![apicore::EphemeralContainer {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "RuntimeDefault".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::EphemeralContainer::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );
        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        ephemeral_containers: Some(vec![apicore::EphemeralContainer {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Localhost".to_string(),
                                    localhost_profile: Some("profile".to_string()),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::EphemeralContainer::default()
                        }]),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "localhost/profile", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "dummy".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default,localhost/profile", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile types: dummy".to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "dummy".to_string(),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            },
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "dummy2".to_string(),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            }
                        ],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default,localhost/profile", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile types: dummy,dummy2"
                    .to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "Localhost".to_string(),
                                        localhost_profile: Some("dummy".to_string()),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            },
                            apicore::Container {
                                security_context: Some(apicore::SecurityContext {
                                    seccomp_profile: Some(apicore::SeccompProfile {
                                        type_: "Localhost".to_string(),
                                        localhost_profile: Some("dummy2".to_string()),
                                        ..apicore::SeccompProfile::default()
                                    }),
                                    ..apicore::SecurityContext::default()
                                }),
                                ..apicore::Container::default()
                            }
                        ],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default,localhost/profile", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid container seccomp profile: dummy,dummy2".to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "RuntimeDefault".to_string(),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "Localhost".to_string(),
                                localhost_profile: Some("profile".to_string()),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "localhost/profile", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "Unconfined".to_string(),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "unconfined", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "Localhost".to_string(),
                                localhost_profile: Some("dummy".to_string()),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "localhost/profile", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Reject(
                "Resource violations: Invalid podspec seccomp profile: dummy".to_string()
            )
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            seccomp_profile: Some(apicore::SeccompProfile {
                                type_: "Localhost".to_string(),
                                localhost_profile: Some("profile2".to_string()),
                                ..apicore::SeccompProfile::default()
                            }),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "locahost/profile,localhost/profile2", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        assert_eq!(
            do_validate(
                &apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        containers: vec![apicore::Container {
                            security_context: Some(apicore::SecurityContext {
                                seccomp_profile: Some(apicore::SeccompProfile {
                                    type_: "Unconfined".to_string(),
                                    ..apicore::SeccompProfile::default()
                                }),
                                ..apicore::SecurityContext::default()
                            }),
                            ..apicore::Container::default()
                        }],
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                configuration!(allowed_profiles: "runtime/default,docker/default,unconfined", profile_types: "", localhost_profiles: ""),
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }
}
