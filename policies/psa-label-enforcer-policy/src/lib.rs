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

    match serde_json::from_value::<apicore::Namespace>(validation_request.request.object) {
        Ok(mut namespace) => {
            let mut labels = namespace.metadata.labels.clone().unwrap_or_default();
            if let Some(mode) = validation_request.settings.modes.enforce {
                labels.insert(
                    "pod-security.kubernetes.io/enforce".to_string(),
                    mode.to_string(),
                );
            }
            if let Some(version) = validation_request.settings.modes.enforce_version {
                labels.insert(
                    "pod-security.kubernetes.io/enforce-version".to_string(),
                    version,
                );
            }

            if let Some(mode) = validation_request.settings.modes.warn {
                labels.insert(
                    "pod-security.kubernetes.io/warn".to_string(),
                    mode.to_string(),
                );
            }
            if let Some(version) = validation_request.settings.modes.warn_version {
                labels.insert(
                    "pod-security.kubernetes.io/warn-version".to_string(),
                    version,
                );
            }

            if let Some(mode) = validation_request.settings.modes.audit {
                labels.insert(
                    "pod-security.kubernetes.io/audit".to_string(),
                    mode.to_string(),
                );
            }
            if let Some(version) = validation_request.settings.modes.audit_version {
                labels.insert(
                    "pod-security.kubernetes.io/audit-version".to_string(),
                    version,
                );
            }
            if labels != namespace.metadata.labels.unwrap_or_default() {
                namespace.metadata.labels = Some(labels);
                kubewarden::mutate_request(
                    serde_json::to_value(namespace).expect("cannot serialize mutated object"),
                )
            } else {
                kubewarden::accept_request()
            }
        }
        Err(_) => kubewarden::accept_request(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::Resource;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use kubewarden::request::{KubernetesAdmissionRequest, ValidationRequest};
    use kubewarden_policy_sdk::request::GroupVersionKind;
    use kubewarden_policy_sdk::response::ValidationResponse;
    use rstest::rstest;
    use serde_json::Value;
    use std::collections::BTreeMap;

    #[rstest]
    #[case::partial_label_update(
        Some(settings::Level::Baseline),
        Some("v1.25".to_string()),
        None, None, None, None,
        serde_json::json!({
            "pod-security.kubernetes.io/enforce": "privileged",
            "pod-security.kubernetes.io/enforce-version": "latest",
            "pod-security.kubernetes.io/audit": "baseline",
            "pod-security.kubernetes.io/audit-version": "v1.26",
            "pod-security.kubernetes.io/warn": "restricted",
            "pod-security.kubernetes.io/warn-version": "v1.27"
        })
    )]
    #[case::add_missing_labels(
        Some(settings::Level::Privileged),
        Some("latest".to_string()),
        Some(settings::Level::Baseline),
        Some("v1.25".to_string()),
        Some(settings::Level::Restricted),
        Some("v1.27".to_string()),
        serde_json::json!({ })
    )]
    #[case::update_all_labels(
        Some(settings::Level::Baseline),
        Some("v1.25".to_string()),
        Some(settings::Level::Restricted),
        Some("v1.27".to_string()),
        Some(settings::Level::Privileged),
        Some("latest".to_string()),
        serde_json::json!({
            "pod-security.kubernetes.io/enforce": "privileged",
            "pod-security.kubernetes.io/enforce-version": "latest",
            "pod-security.kubernetes.io/audit": "baseline",
            "pod-security.kubernetes.io/audit-version": "v1.26",
            "pod-security.kubernetes.io/warn": "restricted",
            "pod-security.kubernetes.io/warn-version": "v1.27"
        })
    )]
    fn mutation_tests(
        #[case] enforce: Option<settings::Level>,
        #[case] enforce_version: Option<String>,
        #[case] audit: Option<settings::Level>,
        #[case] audit_version: Option<String>,
        #[case] warn: Option<settings::Level>,
        #[case] warn_version: Option<String>,
        #[case] namespace_labels: Value,
    ) {
        let original_labels: BTreeMap<String, String> =
            serde_json::from_value(namespace_labels).expect("cannot deserialize labels");
        let namespace = apicore::Namespace {
            metadata: ObjectMeta {
                labels: Some(original_labels.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        let request = KubernetesAdmissionRequest {
            kind: GroupVersionKind {
                kind: apicore::Namespace::KIND.to_string(),
                ..Default::default()
            },
            object: serde_json::to_value(namespace).expect("Cannot serialize namespace object"),
            ..Default::default()
        };
        let settings = Settings {
            modes: crate::settings::Modes {
                enforce,
                enforce_version,
                audit,
                audit_version,
                warn,
                warn_version,
            },
        };
        let validation_request = ValidationRequest::<Settings> { settings, request };
        let payload = serde_json::to_string(&validation_request).expect("Cannot serialize payload");

        let response: ValidationResponse =
            serde_json::from_slice(&validate(payload.as_bytes()).expect("Validation failed"))
                .expect("Cannot parse response JSON");
        assert!(response.accepted);
        assert!(response.mutated_object.is_some());
        let mutated_object: apicore::Namespace =
            serde_json::from_value(response.mutated_object.unwrap())
                .expect("cannot deserialize mutated object");
        let labels = mutated_object.metadata.labels.unwrap_or_default();

        let mut expected_labels = original_labels;
        if let Some(enforce) = validation_request.settings.modes.enforce {
            expected_labels.insert(
                "pod-security.kubernetes.io/enforce".to_string(),
                enforce.to_string(),
            );
        }
        if let Some(enforce_version) = validation_request.settings.modes.enforce_version {
            expected_labels.insert(
                "pod-security.kubernetes.io/enforce-version".to_string(),
                enforce_version,
            );
        }
        if let Some(audit) = validation_request.settings.modes.audit {
            expected_labels.insert(
                "pod-security.kubernetes.io/audit".to_string(),
                audit.to_string(),
            );
        }
        if let Some(audit_version) = validation_request.settings.modes.audit_version {
            expected_labels.insert(
                "pod-security.kubernetes.io/audit-version".to_string(),
                audit_version,
            );
        }
        if let Some(warn) = validation_request.settings.modes.warn {
            expected_labels.insert(
                "pod-security.kubernetes.io/warn".to_string(),
                warn.to_string(),
            );
        }
        if let Some(warn_version) = validation_request.settings.modes.warn_version {
            expected_labels.insert(
                "pod-security.kubernetes.io/warn-version".to_string(),
                warn_version,
            );
        }
        assert_eq!(labels, expected_labels);
    }

    #[test]
    fn accept_valid_psa_labels_test() {
        let mut labels: BTreeMap<String, String> = BTreeMap::new();
        labels.insert(
            "pod-security.kubernetes.io/enforce".to_string(),
            "privileged".to_string(),
        );
        labels.insert(
            "pod-security.kubernetes.io/enforce-version".to_string(),
            "latest".to_string(),
        );
        labels.insert(
            "pod-security.kubernetes.io/audit".to_string(),
            "baseline".to_string(),
        );
        labels.insert(
            "pod-security.kubernetes.io/audit-version".to_string(),
            "v1.25".to_string(),
        );
        labels.insert(
            "pod-security.kubernetes.io/warn".to_string(),
            "restricted".to_string(),
        );
        labels.insert(
            "pod-security.kubernetes.io/warn-version".to_string(),
            "v1.27".to_string(),
        );

        let namespace = apicore::Namespace {
            metadata: ObjectMeta {
                labels: Some(labels),
                ..Default::default()
            },
            ..Default::default()
        };
        let request = KubernetesAdmissionRequest {
            kind: GroupVersionKind {
                kind: apicore::Namespace::KIND.to_string(),
                ..Default::default()
            },
            object: serde_json::to_value(namespace).expect("Cannot serialize namespace object"),
            ..Default::default()
        };
        let settings = Settings {
            modes: crate::settings::Modes {
                enforce: Some(settings::Level::Privileged),
                enforce_version: Some("latest".to_string()),
                audit: Some(settings::Level::Baseline),
                audit_version: Some("v1.25".to_string()),
                warn: Some(settings::Level::Restricted),
                warn_version: Some("v1.27".to_string()),
            },
        };
        let validation_request = ValidationRequest::<Settings> { settings, request };
        let payload = serde_json::to_string(&validation_request).expect("Cannot serialize payload");

        let response: ValidationResponse =
            serde_json::from_slice(&validate(payload.as_bytes()).expect("Validation failed"))
                .expect("Cannot parse response JSON");
        assert!(response.accepted);
        assert!(response.mutated_object.is_none());
    }
}
