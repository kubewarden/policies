extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{protocol_version_guest, request::ValidationRequest, validate_settings};

use guest::prelude::*;
use kubewarden::wapc_guest as guest;

use k8s_openapi::api::core::v1 as apicore;

mod settings;
use settings::Settings;

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn validate_pod_spec(
    pod_spec_opt: Option<apicore::PodSpec>,
    settings: &Settings,
) -> Result<(), String> {
    if pod_spec_opt.is_none() {
        return Ok(());
    }
    let pod_spec = pod_spec_opt.unwrap();
    if pod_spec.host_ipc.unwrap_or(false) && !settings.allow_host_ipc {
        return Err("Pod has IPC enabled, but this is not allowed".to_string());
    }

    if pod_spec.host_network.unwrap_or(false) && !settings.allow_host_network {
        return Err("Pod has host network enabled, but this is not allowed".to_string());
    }

    if pod_spec.host_pid.unwrap_or(false) && !settings.allow_host_pid {
        return Err("Pod has host PID enabled, but this is not allowed".to_string());
    }

    if !all_containers_allowed(
        &pod_spec.init_containers.unwrap_or_default(),
        &settings.allow_host_ports,
    ) {
        return Err("Pod is using unallowed host ports in init containers".to_string());
    }

    if !all_containers_allowed(&pod_spec.containers, &settings.allow_host_ports) {
        return Err("Pod is using unallowed host ports in containers".to_string());
    }

    Ok(())
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;
    match validation_request.extract_pod_spec_from_object() {
        Ok(pod_spec) => match validate_pod_spec(pod_spec, &validation_request.settings) {
            Ok(_) => kubewarden::accept_request(),
            Err(err) => kubewarden::reject_request(Some(err), None, None, None),
        },
        Err(_) => kubewarden::accept_request(),
    }
}

fn all_containers_allowed(
    containers: &[apicore::Container],
    allowed_host_ports: &[settings::PortRange],
) -> bool {
    containers.iter().all(|container| {
        container
            .ports
            .clone()
            .unwrap_or_default()
            .iter()
            .all(|port| match port.host_port {
                Some(host_port) => allowed_host_ports
                    .iter()
                    .any(|allowed_host_port| allowed_host_port.in_range(host_port)),
                None => true,
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn invalid_pod_spec() -> Option<apicore::PodSpec> {
        Some(apicore::PodSpec {
            host_ipc: Some(true),
            host_network: Some(true),
            host_pid: Some(true),
            containers: vec![apicore::Container {
                name: "test-container".to_string(),
                image: Some("test-image".to_string()),
                ports: Some(vec![apicore::ContainerPort {
                    container_port: 8080,
                    host_port: Some(8080),
                    ..Default::default()
                }]),
                ..Default::default()
            }],
            ..Default::default()
        })
    }

    fn valid_pod_spec() -> Option<apicore::PodSpec> {
        Some(apicore::PodSpec {
            host_ipc: Some(false),
            host_network: Some(false),
            host_pid: Some(false),
            ..Default::default()
        })
    }

    fn block_all() -> Settings {
        Settings {
            allow_host_ipc: false,
            allow_host_network: false,
            allow_host_pid: false,
            allow_host_ports: vec![settings::PortRange { min: 443, max: 443 }],
        }
    }

    fn allow_all() -> Settings {
        Settings {
            allow_host_ipc: true,
            allow_host_network: true,
            allow_host_pid: true,
            allow_host_ports: vec![settings::PortRange {
                min: 8080,
                max: 8080,
            }],
        }
    }

    #[rstest]
    #[case::no_pod_spec(None, block_all(), true)]
    #[case::valid_pod_spec(valid_pod_spec(), block_all(), true)]
    #[case::invalid_pod_spec(invalid_pod_spec(), block_all(), false)]
    #[case::invalid_pod_spec_no_validation(invalid_pod_spec(), allow_all(), true)]
    fn validate_pod_spec_test(
        #[case] pod_spec_opt: Option<apicore::PodSpec>,
        #[case] settings: Settings,
        #[case] allowed: bool,
    ) {
        let result = validate_pod_spec(pod_spec_opt, &settings);
        if allowed {
            result.expect("Expected validation to succeed");
        } else {
            result.expect_err("Expected validation to fail");
        }
    }
}
