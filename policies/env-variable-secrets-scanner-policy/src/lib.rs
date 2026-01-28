use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;
extern crate kubewarden_policy_sdk as kubewarden;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STD_ENGINE};
use k8s_openapi::api::core::v1::{EnvVar, PodSpec};
use kubewarden::{protocol_version_guest, request::ValidationRequest, validate_settings};
use rusty_hog_scanner::{SecretScanner, SecretScannerBuilder};
use std::{collections::HashSet, fmt, string::String};

mod settings;
use settings::Settings;

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

/// Represents a secret that has been found in an env var
#[derive(Eq, Hash, PartialEq, Debug)]
struct EnvVarFinding {
    /// name of the container where the secret was found
    container: String,
    /// reason of rejection. It describes the secret that it was found
    reason: String,
    /// key of the env var
    key: String,
}

impl fmt::Display for EnvVarFinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "container: {}, key: {}, reason: {}. ",
            self.container, self.key, self.reason
        )
    }
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;

    match validation_request.extract_pod_spec_from_object() {
        Ok(pod_spec) => {
            if let Some(pod_spec) = pod_spec {
                return validate_pod_spec(pod_spec);
            }
            // If there is no pod spec, just accept it. There is no data to be validated.
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

/// Scan all containers in containers, init_containers and ephemeral_containers.
/// Returns reject_request if any secret var was found in an env_var in any container or
/// accept_request otherwise.
fn validate_pod_spec(pod_spec: PodSpec) -> CallResult {
    let mut findings: HashSet<EnvVarFinding> = HashSet::new();
    let secret_scanner = SecretScannerBuilder::new().build();

    for container in pod_spec.containers {
        if let Some(secrets_in_container) =
            scan_env_vars(container.env, &secret_scanner, &container.name)
        {
            findings.extend(secrets_in_container);
        }
    }

    if let Some(init_containers) = pod_spec.init_containers {
        for container in init_containers {
            if let Some(secrets_in_container) =
                scan_env_vars(container.env, &secret_scanner, &container.name)
            {
                findings.extend(secrets_in_container);
            }
        }
    }

    if let Some(ephemeral_containers) = pod_spec.ephemeral_containers {
        for container in ephemeral_containers {
            if let Some(secrets_in_container) =
                scan_env_vars(container.env, &secret_scanner, &container.name)
            {
                findings.extend(secrets_in_container);
            }
        }
    }

    if !findings.is_empty() {
        return kubewarden::reject_request(
            Some(format!(
                "The following secrets were found in environment variables -> {}",
                create_error_message(findings)
            )),
            None,
            None,
            None,
        );
    }

    kubewarden::accept_request()
}

fn scan_env_vars(
    env_vars: Option<Vec<EnvVar>>,
    secret_scanner: &SecretScanner,
    container_name: &str,
) -> Option<HashSet<EnvVarFinding>> {
    let mut findings: HashSet<EnvVarFinding> = HashSet::new();

    if let Some(env_vars) = env_vars {
        for env_var in env_vars {
            if let Some(value) = env_var.value {
                findings.extend(scan_env_var(
                    value.as_bytes().to_vec(),
                    secret_scanner,
                    env_var.name.as_str(),
                    container_name,
                ));
            }
        }
    }

    if findings.is_empty() {
        None
    } else {
        Some(findings)
    }
}

fn scan_env_var(
    input: Vec<u8>,
    secret_scanner: &SecretScanner,
    key: &str,
    container: &str,
) -> HashSet<EnvVarFinding> {
    let mut findings = scan_text(&input, secret_scanner, key, container);

    // try decoding content from base64 if no secret was found
    if findings.is_empty() {
        let input = BASE64_STD_ENGINE.decode(input);
        if let Ok(input) = input {
            findings = scan_text(&input, secret_scanner, key, container);
        }
    }

    findings
}

fn scan_text(
    input: &[u8],
    secret_scanner: &SecretScanner,
    key: &str,
    container: &str,
) -> HashSet<EnvVarFinding> {
    let mut findings: HashSet<EnvVarFinding> = HashSet::new();
    let lines = input.split(|&x| (x as char) == '\n');

    for new_line in lines.into_iter() {
        let results = secret_scanner.matches(new_line);
        for (reason, matches) in results {
            for _ in matches {
                findings.insert(EnvVarFinding {
                    reason: reason.to_string(),
                    key: key.to_string(),
                    container: container.to_string(),
                });
            }
        }
    }

    findings
}

fn create_error_message(secrets: HashSet<EnvVarFinding>) -> String {
    let mut message = String::new();
    for secret in secrets {
        message.push_str(secret.to_string().as_str())
    }

    message
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::test::Testcase;

    #[test]
    fn reject_pod_with_secrets() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_with_secrets.json";
        let tc = Testcase {
            name: String::from("pod with secrets"),
            fixture_file: String::from(request_file),
            expected_validation_result: false,
            settings: Settings {},
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.message
                .clone()
                .unwrap_or_default()
                .contains("The following secrets were found in environment variables"),
        );
        assert!(
            res.message
                .clone()
                .unwrap_or_default()
                .contains("container: nginx, key: email, reason: Email address"),
        );
        assert!(
            res.message
                .clone()
                .unwrap_or_default()
                .contains("container: nginx, key: rsa, reason: RSA private key"),
        );

        assert!(
            res.mutated_object.is_none(),
            "Something mutated with test case: {}",
            tc.name,
        );

        Ok(())
    }

    #[test]
    fn reject_pod_with_secrets_base64() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_with_secrets_base64.json";
        let tc = Testcase {
            name: String::from("pod with secrets"),
            fixture_file: String::from(request_file),
            expected_validation_result: false,
            settings: Settings {},
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.message
                .clone()
                .unwrap_or_default()
                .contains("The following secrets were found in environment variables"),
        );
        assert!(
            res.message
                .clone()
                .unwrap_or_default()
                .contains("container: nginx, key: rsa, reason: RSA private key"),
        );

        assert!(
            res.mutated_object.is_none(),
            "Something mutated with test case: {}",
            tc.name,
        );

        Ok(())
    }

    #[test]
    fn reject_pod_with_secrets_init_and_ephemeral_containers() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_with_secrets_init_and_ephemeral_containers.json";
        let tc = Testcase {
            name: String::from("pod with secrets in init and ephemeral containers"),
            fixture_file: String::from(request_file),
            expected_validation_result: false,
            settings: Settings {},
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.message
                .clone()
                .unwrap_or_default()
                .contains("The following secrets were found in environment variables"),
        );
        assert!(
            res.message
                .clone()
                .unwrap_or_default()
                .contains("container: busybox, key: email, reason: Email address"),
        );
        assert!(
            res.message
                .clone()
                .unwrap_or_default()
                .contains("container: nginx, key: rsa, reason: RSA private key"),
        );

        assert!(
            res.mutated_object.is_none(),
            "Something mutated with test case: {}",
            tc.name,
        );

        Ok(())
    }

    #[test]
    fn accept_pod_creation_without_secrets() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_without_secrets.json";
        let tc = Testcase {
            name: String::from("pod without secrets"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {},
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.mutated_object.is_none(),
            "Something mutated with test case: {}",
            tc.name,
        );

        Ok(())
    }
}
