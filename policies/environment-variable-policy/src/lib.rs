use std::collections::HashMap;

use anyhow::{Result, anyhow};
use criteria_policy_base::{
    kubewarden_policy_sdk::{
        accept_request, protocol_version_guest, reject_request, request::ValidationRequest,
        validate_settings, wapc_guest as guest,
    },
    validate::validate_values,
};
use guest::prelude::*;
use k8s_openapi::api::core::v1::{self as apicore, Container, EphemeralContainer};
use settings::Settings;

mod settings;

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<settings::Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn validate_envvar(settings: &Settings, env_vars: &[String]) -> Result<()> {
    validate_values(&settings.0, env_vars)
}

// Returns a map with container names as keys and their environment variable names as values
fn get_containers_env_vars(containers: &[Container]) -> HashMap<String, Vec<String>> {
    let mut results = HashMap::new();
    for container in containers {
        let envvar_names: Vec<String> = container
            .env
            .as_ref()
            .map(|envvars| envvars.iter().map(|e| e.name.clone()).collect())
            .unwrap_or_default();
        results.insert(container.name.clone(), envvar_names);
    }
    results
}

// Returns a map with ephemeral container names as keys and their environment variable names as values
fn get_ephemeral_containers_env_vars(
    containers: &[EphemeralContainer],
) -> HashMap<String, Vec<String>> {
    let mut results = HashMap::new();
    for container in containers {
        let envvar_names: Vec<String> = container
            .env
            .as_ref()
            .map(|envvars| envvars.iter().map(|e| e.name.clone()).collect())
            .unwrap_or_default();
        results.insert(container.name.clone(), envvar_names);
    }
    results
}

fn validate_environment_variables(
    pod: &apicore::PodSpec,
    settings: &settings::Settings,
) -> Result<(), Vec<String>> {
    let mut envvars = get_containers_env_vars(&pod.containers);
    envvars.extend(get_containers_env_vars(
        pod.init_containers.as_ref().unwrap_or(&vec![]),
    ));
    envvars.extend(get_ephemeral_containers_env_vars(
        pod.ephemeral_containers.as_ref().unwrap_or(&vec![]),
    ));
    let errors = envvars
        .iter()
        .filter_map(|(container_name, envvar)| {
            validate_envvar(settings, envvar)
                .map_err(|err| anyhow!("{}: {}", container_name, err))
                .err()
        })
        .map(|err| err.to_string())
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(())
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<settings::Settings> =
        ValidationRequest::new(payload)?;
    let pod_spec = validation_request.extract_pod_spec_from_object()?;
    if let Err(errors) =
        validate_environment_variables(&pod_spec.unwrap_or_default(), &validation_request.settings)
    {
        return reject_request(Some(errors.join(", ")), None, None, None);
    }
    accept_request()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashSet;

    use criteria_policy_base::{constants::CONTAINS_ANY_OF_ERROR_MSG, settings::BaseSettings};
    use rstest::rstest;

    #[rstest]
    #[case(
        vec![
            apicore::Container {
                name: "test-container1".to_string(),
                env: Some(vec![
                    apicore::EnvVar {
                        name: "a".to_string(),
                        ..Default::default()
                    },
                    apicore::EnvVar {
                        name: "b".to_string(),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
            apicore::Container {
                name: "test-container2".to_string(),
                env: Some(vec![
                    apicore::EnvVar {
                        name: "x".to_string(),
                        ..Default::default()
                    },
                    apicore::EnvVar {
                        name: "y".to_string(),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
        ],
        HashMap::from([
            ("test-container1".to_string(), vec!["a".to_owned(), "b".to_owned()]),
            ("test-container2".to_string(), vec!["x".to_owned(), "y".to_owned()]),
        ])
    )]
    #[case(
        vec![
            apicore::Container {
                name: "test-container3".to_string(),
                env: None,
                ..Default::default()
            },
        ],
        HashMap::from([("test-container3".to_string(), vec![])])
    )]
    fn test_get_containers_env_vars(
        #[case] containers: Vec<apicore::Container>,
        #[case] expected: HashMap<String, Vec<String>>,
    ) {
        let result = get_containers_env_vars(&containers);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        vec![
            apicore::EphemeralContainer {
                name: "test-ephemeral1".to_string(),
                env: Some(vec![
                    apicore::EnvVar {
                        name: "foo".to_owned(),
                        ..Default::default()
                    },
                    apicore::EnvVar {
                        name: "bar".to_owned(),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
            apicore::EphemeralContainer {
                name: "test-ephemeral2".to_string(),
                env: Some(vec![
                    apicore::EnvVar {
                        name: "baz".to_owned(),
                        ..Default::default()
                    },
                    apicore::EnvVar {
                        name: "qux".to_owned(),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
        ],
        HashMap::from([
            ("test-ephemeral1".to_string(), vec!["foo".to_owned(), "bar".to_owned()]),
            ("test-ephemeral2".to_string(), vec!["baz".to_owned(), "qux".to_owned()]),
        ])
    )]
    #[case(
        vec![
            apicore::EphemeralContainer {
                name: "test-ephemeral3".to_string(),
                env: None,
                ..Default::default()
            },
        ],
        HashMap::from([("test-ephemeral3".to_string(), vec![])])
    )]
    fn test_get_ephemeral_containers_env_vars(
        #[case] ephemeral_containers: Vec<apicore::EphemeralContainer>,
        #[case] expected: HashMap<String, Vec<String>>,
    ) {
        let result = get_ephemeral_containers_env_vars(&ephemeral_containers);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_container_error_message() {
        let settings = Settings(BaseSettings::ContainsAnyOf {
            values: HashSet::from(["a".to_owned(), "b".to_owned()]),
        });
        let container_envvar = Some(Vec::from([apicore::EnvVar {
            name: "c".to_owned(),
            ..Default::default()
        }]));

        let pod_spec = &apicore::PodSpec {
            containers: vec![
                apicore::Container {
                    name: "test-container1".to_string(),
                    env: container_envvar.clone(),
                    ..Default::default()
                },
                apicore::Container {
                    name: "test-container2".to_string(),
                    env: container_envvar.clone(),
                    ..Default::default()
                },
            ],
            init_containers: Some(vec![
                apicore::Container {
                    name: "test-container3".to_string(),
                    env: container_envvar.clone(),
                    ..Default::default()
                },
                apicore::Container {
                    name: "test-container4".to_string(),
                    env: container_envvar.clone(),
                    ..Default::default()
                },
            ]),
            ephemeral_containers: Some(vec![
                apicore::EphemeralContainer {
                    name: "test-container5".to_string(),
                    env: container_envvar.clone(),
                    ..Default::default()
                },
                apicore::EphemeralContainer {
                    name: "test-container6".to_string(),
                    env: container_envvar.clone(),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        // Ensure that all problematic containers are caught and there is an error for each
        let errors = validate_environment_variables(pod_spec, &settings)
            .expect_err("Expected validation to fail");
        assert_eq!(errors.len(), 6, "Expected 6 errors, got {}", errors.len());
        for c in 1..6 {
            assert!(
                errors.iter().any(|errmsg| errmsg
                    .starts_with(&format!("test-container{c}: {CONTAINS_ANY_OF_ERROR_MSG} "))),
                "Validation error message does not contain expected text"
            );
        }
    }
}
