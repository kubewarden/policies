use std::collections::HashSet;

use anyhow::{Result, anyhow};
use lazy_static::lazy_static;

use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::api::core::v1::{self as apicore, VolumeMount};

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{logging, protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::Settings;

use slog::{Logger, o};

lazy_static! {
    static ref LOG_DRAIN: Logger = Logger::root(
        logging::KubewardenDrain::new(),
        o!("policy" => "volumemounts-policy")
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
        Ok(pod_spec) => {
            if let Some(pod_spec) = pod_spec {
                return match validate_pod(&pod_spec, &validation_request.settings) {
                    Ok(_) => kubewarden::accept_request(),
                    Err(err) => kubewarden::reject_request(Some(err.to_string()), None, None, None),
                };
            };
            // If there is not pod spec, just accept it. There is no data to be
            // validated.
            kubewarden::accept_request()
        }
        Err(_) => kubewarden::reject_request(
            Some("Cannot parse validation request".to_string()),
            None,
            None,
            None,
        ),
    }
}

fn validate_pod(pod: &apicore::PodSpec, settings: &settings::Settings) -> Result<()> {
    let mut err_message = String::new();
    for container in &pod.containers {
        let container_valid = validate_container(container, settings);
        if let Err(e) = container_valid {
            err_message =
                err_message + &format!("container {} is invalid: {}\n", container.name, e);
        }
    }
    if let Some(init_containers) = &pod.init_containers {
        for container in init_containers {
            let container_valid = validate_container(container, settings);
            if let Err(e) = container_valid {
                err_message =
                    err_message + &format!("container {} is invalid: {}\n", container.name, e);
            }
        }
    }
    if let Some(ephemeral_containers) = &pod.ephemeral_containers {
        for container in ephemeral_containers {
            let container_valid = validate_ephemeral_container(container, settings);
            if let Err(e) = container_valid {
                err_message =
                    err_message + &format!("container {} is invalid: {}\n", container.name, e);
            }
        }
    }
    if err_message.is_empty() {
        return Ok(());
    }
    Err(anyhow!(err_message))
}

fn validate_ephemeral_container(
    container: &apicore::EphemeralContainer,
    settings: &settings::Settings,
) -> Result<()> {
    if let Some(volume_mounts) = &container.volume_mounts {
        return validate_volume_mounts(volume_mounts, settings);
    }
    Ok(())
}

fn validate_container(container: &apicore::Container, settings: &settings::Settings) -> Result<()> {
    if let Some(volume_mounts) = &container.volume_mounts {
        return validate_volume_mounts(volume_mounts, settings);
    }
    Ok(())
}

fn validate_volume_mounts(
    volume_mounts: &[VolumeMount],
    settings: &settings::Settings,
) -> Result<()> {
    let mut volume_mounts_names: HashSet<String> = HashSet::new();
    for mount in volume_mounts {
        volume_mounts_names.insert(mount.name.to_string());
    }
    match settings.operator {
        settings::Reject::AllAreUsed => {
            // cannot use both at once, only one or the other
            if settings.volume_mounts_names.is_subset(&volume_mounts_names) {
                let mut sorted_names = settings
                    .volume_mounts_names
                    .clone()
                    .into_iter()
                    .collect::<Vec<String>>();
                sorted_names.sort();
                return Err(anyhow!(
                    "volumeMount names not allowed together: {:?}",
                    sorted_names
                ));
            }
            Ok(())
        }
        settings::Reject::AnyIn => {
            // denylist
            let intersection: HashSet<_> = settings
                .volume_mounts_names
                .intersection(&volume_mounts_names)
                .collect();
            match intersection.is_empty() {
                true => Ok(()),
                false => {
                    let mut sorted_names =
                        intersection.into_iter().cloned().collect::<Vec<String>>();
                    sorted_names.sort();
                    Err(anyhow!("volumeMount names not allowed: {:?}", sorted_names))
                }
            }
        }
        settings::Reject::AnyNotIn => {
            // allowlist
            let difference: HashSet<_> = volume_mounts_names
                .difference(&settings.volume_mounts_names)
                .collect();
            match difference.is_empty() {
                true => Ok(()),
                false => {
                    let mut sorted_names = difference.into_iter().cloned().collect::<Vec<String>>();
                    sorted_names.sort();
                    Err(anyhow!("volumeMount names not allowed: {:?}", sorted_names))
                }
            }
        }
        settings::Reject::NotAllAreUsed => {
            // can use both at once, but not only one of them
            let difference: HashSet<_> = settings
                .volume_mounts_names
                .difference(&volume_mounts_names)
                .collect();
            match difference.is_empty() {
                true => Ok(()),
                false => {
                    let mut sorted_names = difference.into_iter().cloned().collect::<Vec<String>>();
                    sorted_names.sort();
                    Err(anyhow!("volumeMount names are missing: {:?}", sorted_names))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::test::Testcase;

    #[test]
    fn accept_pod_no_volume_mounts_default_settings() -> Result<(), ()> {
        let request_file = "test_data/pod_creation.json";
        let tc = Testcase {
            name: String::from("no volumeMounts"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings::default(),
        };

        tc.eval(validate).unwrap();
        Ok(())
    }

    #[test]
    fn accept_pod_with_no_volume_mounts_denylist() -> Result<(), ()> {
        let request_file = "test_data/pod_creation.json";
        let tc = Testcase {
            name: String::from("no volumeMounts"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {
                operator: settings::Reject::AnyIn,
                volume_mounts_names: HashSet::from([String::from("test1")]),
            },
        };

        tc.eval(validate).unwrap();
        Ok(())
    }

    #[test]
    fn reject_pod_with_denylist() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_volume_mounts.json";
        let tc = Testcase {
            name: String::from("denylist"),
            fixture_file: String::from(request_file),
            expected_validation_result: false,
            settings: Settings {
                operator: settings::Reject::AnyIn,
                volume_mounts_names: HashSet::from([
                    String::from("test-var"),
                    String::from("test-data"),
                ]),
            },
        };

        let result = tc.eval(validate);
        let expected = "container busybox is invalid: volumeMount names not allowed: [\"test-var\"]
container init-myservice is invalid: volumeMount names not allowed: [\"test-data\"]
container init-myservice2 is invalid: volumeMount names not allowed: [\"test-var\"]
";
        assert_eq!(expected, result.unwrap().message.unwrap());
        Ok(())
    }

    #[test]
    fn accept_pod_with_allowlist() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_volume_mounts.json";
        let tc = Testcase {
            name: String::from("allowlist"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {
                operator: settings::Reject::AnyNotIn,
                volume_mounts_names: HashSet::from([
                    String::from("test-var"),
                    String::from("test-data"),
                    String::from("test-var-local-aaa"),
                    String::from("kube-api-access-kplj9"),
                ]),
            },
        };

        tc.eval(validate).unwrap();
        Ok(())
    }

    #[test]
    fn reject_pod_with_allowlist() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_volume_mounts.json";
        let tc = Testcase {
            name: String::from("allowlist reject"),
            fixture_file: String::from(request_file),
            expected_validation_result: false,
            settings: Settings {
                operator: settings::Reject::AnyNotIn,
                volume_mounts_names: HashSet::from([String::from("unexistent")]),
            },
        };

        tc.eval(validate).unwrap();
        let result = tc.eval(validate);
        let expected = "container busybox is invalid: volumeMount names not allowed: [\"kube-api-access-kplj9\", \"test-var\", \"test-var-local-aaa\"]
container busybox2 is invalid: volumeMount names not allowed: [\"test-var-local-aaa\"]
container init-myservice is invalid: volumeMount names not allowed: [\"test-data\"]
container init-myservice2 is invalid: volumeMount names not allowed: [\"test-var\"]
";
        assert_eq!(expected, result.unwrap().message.unwrap());
        Ok(())
    }

    #[test]
    fn accept_pod_with_all_are_used() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_volume_mounts.json";
        let tc = Testcase {
            name: String::from("not_both_at_once"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {
                operator: settings::Reject::AllAreUsed,
                volume_mounts_names: HashSet::from([
                    String::from("test-var"),
                    String::from("unexistent"),
                ]),
            },
        };

        tc.eval(validate).unwrap();
        Ok(())
    }

    #[test]
    fn reject_pod_with_all_are_used() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_volume_mounts.json";
        let tc = Testcase {
            name: String::from("not_both_at_once reject"),
            fixture_file: String::from(request_file),
            expected_validation_result: false,
            settings: Settings {
                operator: settings::Reject::AllAreUsed,
                volume_mounts_names: HashSet::from([
                    String::from("test-var"),
                    String::from("test-var-local-aaa"),
                ]),
            },
        };

        tc.eval(validate).unwrap();
        let result = tc.eval(validate);
        let expected = "container busybox is invalid: volumeMount names not allowed together: [\"test-var\", \"test-var-local-aaa\"]\n";
        assert_eq!(expected, result.unwrap().message.unwrap());
        Ok(())
    }

    #[test]
    fn accept_pod_with_not_all_are_used() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_volume_mounts_only2.json";
        let tc = Testcase {
            name: String::from("only_both_at_once"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {
                operator: settings::Reject::NotAllAreUsed,
                volume_mounts_names: HashSet::from([
                    String::from("test-var"),
                    String::from("test-var-local-aaa"),
                ]),
            },
        };

        tc.eval(validate).unwrap();
        Ok(())
    }

    #[test]
    fn reject_pod_with_not_all_are_used() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_volume_mounts_only2.json";
        let tc = Testcase {
            name: String::from("only_both_at_once reject"),
            fixture_file: String::from(request_file),
            expected_validation_result: false,
            settings: Settings {
                operator: settings::Reject::NotAllAreUsed,
                volume_mounts_names: HashSet::from([
                    String::from("test-var"),
                    String::from("nonexistent"),
                ]),
            },
        };

        tc.eval(validate).unwrap();
        let result = tc.eval(validate);
        let expected =
            "container busybox is invalid: volumeMount names are missing: [\"nonexistent\"]\n";
        assert_eq!(expected, result.unwrap().message.unwrap());
        Ok(())
    }
}
