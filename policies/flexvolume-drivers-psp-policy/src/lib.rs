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

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;

    let pod = match serde_json::from_value::<apicore::Pod>(validation_request.request.object) {
        Ok(pod) => pod,
        Err(_) => return kubewarden::accept_request(),
    };

    let pod_spec = pod.spec.ok_or("invalid pod spec")?;
    let settings = validation_request.settings;

    if any_invalid_flexvolume_driver(
        pod_spec,
        settings
            .allowed_flex_volumes
            .iter()
            .map(|flex_volume| flex_volume.driver.clone())
            .collect(),
    ) {
        return kubewarden::reject_request(
            Some("Pod has at least one invalid flex volume driver".to_string()),
            None,
            None,
            None,
        );
    }

    kubewarden::accept_request()
}

fn any_invalid_flexvolume_driver(
    pod_spec: apicore::PodSpec,
    allowed_flex_volumes: Vec<String>,
) -> bool {
    match &pod_spec.volumes {
        Some(volumes) => volumes.iter().any(|volume| match &volume.flex_volume {
            Some(flex_volume) => !allowed_flex_volumes.contains(&flex_volume.driver),
            None => false,
        }),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_volumes_accepts_with_no_containers() {
        let allowed_flex_volumes = vec!["example/lvm".to_string(), "example/cifs".to_string()];

        assert!(!any_invalid_flexvolume_driver(
            apicore::PodSpec::default(),
            allowed_flex_volumes
        ));
    }

    #[test]
    fn a_valid_volume_accepts() {
        let allowed_flex_volumes = vec!["example/lvm".to_string(), "example/cifs".to_string()];

        assert!(!any_invalid_flexvolume_driver(
            apicore::PodSpec {
                volumes: Some(vec![apicore::Volume {
                    flex_volume: Some(apicore::FlexVolumeSource {
                        driver: "example/lvm".to_string(),
                        ..apicore::FlexVolumeSource::default()
                    }),
                    ..apicore::Volume::default()
                }]),
                ..apicore::PodSpec::default()
            },
            allowed_flex_volumes
        ));
    }

    #[test]
    fn an_invalid_volume_rejects() {
        let allowed_flex_volumes = vec!["example/lvm".to_string(), "example/cifs".to_string()];

        assert!(any_invalid_flexvolume_driver(
            apicore::PodSpec {
                volumes: Some(vec![apicore::Volume {
                    flex_volume: Some(apicore::FlexVolumeSource {
                        driver: "example/other".to_string(),
                        ..apicore::FlexVolumeSource::default()
                    }),
                    ..apicore::Volume::default()
                }]),
                ..apicore::PodSpec::default()
            },
            allowed_flex_volumes
        ));
    }

    #[test]
    fn some_invalid_volume_rejects() {
        let allowed_flex_volumes = vec!["example/lvm".to_string(), "example/cifs".to_string()];

        assert!(any_invalid_flexvolume_driver(
            apicore::PodSpec {
                volumes: Some(vec![
                    apicore::Volume {
                        flex_volume: Some(apicore::FlexVolumeSource {
                            driver: "example/cifs".to_string(),
                            ..apicore::FlexVolumeSource::default()
                        }),
                        ..apicore::Volume::default()
                    },
                    apicore::Volume {
                        flex_volume: Some(apicore::FlexVolumeSource {
                            driver: "example/other".to_string(),
                            ..apicore::FlexVolumeSource::default()
                        }),
                        ..apicore::Volume::default()
                    }
                ]),
                ..apicore::PodSpec::default()
            },
            allowed_flex_volumes
        ));
    }

    #[test]
    fn no_invalid_volume_accepts() {
        let allowed_flex_volumes = vec!["example/lvm".to_string(), "example/cifs".to_string()];

        assert!(!any_invalid_flexvolume_driver(
            apicore::PodSpec {
                volumes: Some(vec![
                    apicore::Volume {
                        flex_volume: Some(apicore::FlexVolumeSource {
                            driver: "example/cifs".to_string(),
                            ..apicore::FlexVolumeSource::default()
                        }),
                        ..apicore::Volume::default()
                    },
                    apicore::Volume {
                        flex_volume: Some(apicore::FlexVolumeSource {
                            driver: "example/lvm".to_string(),
                            ..apicore::FlexVolumeSource::default()
                        }),
                        ..apicore::Volume::default()
                    }
                ]),
                ..apicore::PodSpec::default()
            },
            allowed_flex_volumes
        ));
    }
}
