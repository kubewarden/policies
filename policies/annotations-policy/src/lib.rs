use std::collections::HashSet;

use anyhow::Result;
use criteria_policy_base::{
    kubewarden_policy_sdk::{
        accept_request, protocol_version_guest, reject_request, request::ValidationRequest,
        validate_settings, wapc_guest as guest,
    },
    validate::validate_values,
};
use guest::prelude::*;
use settings::Settings;

mod settings;

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<settings::Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn validate_annotations(
    settings: &Settings,
    resource_annots: &HashSet<String>,
) -> Result<(), Vec<String>> {
    validate_values(
        &settings.0,
        &resource_annots.iter().cloned().collect::<Vec<_>>(),
    )
    .map_err(|e| vec![e.to_string()])
}

fn get_resource_annotation_keys(
    validation_request: &ValidationRequest<Settings>,
) -> HashSet<String> {
    validation_request
        .request
        .object
        .get("metadata")
        .and_then(|m| m.get("annotations"))
        .and_then(|a| a.as_object())
        .map(|annots| annots.keys().cloned().collect())
        .unwrap_or_default()
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<settings::Settings> =
        ValidationRequest::new(payload)?;
    let annots = get_resource_annotation_keys(&validation_request);

    if let Err(errors) = validate_annotations(&validation_request.settings, &annots) {
        return reject_request(Some(errors.join(", ")), None, None, None);
    }
    accept_request()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::{BTreeMap, HashSet};

    use crate::settings::Settings;
    use criteria_policy_base::kubewarden_policy_sdk::request::{
        KubernetesAdmissionRequest, ValidationRequest,
    };
    use criteria_policy_base::kubewarden_policy_sdk::settings::Validatable;

    use criteria_policy_base::settings::BaseSettings;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    use k8s_openapi::api::apps::v1::Deployment;
    use k8s_openapi::api::networking::v1::Ingress;

    use rstest::rstest;
    use serde_json::to_value;

    #[rstest]
    #[case(
        // Deployment without annotations
        Deployment {
            metadata: ObjectMeta {
                annotations: None,
                ..Default::default()
            },
            ..Default::default()
        },
        HashSet::new()
    )]
    #[case(
        // Deployment with annotations
        {
            let mut annots = BTreeMap::new();
            annots.insert("foo".to_string(), "bar".to_string());
            annots.insert("baz".to_string(), "qux".to_string());
            Deployment {
                metadata: ObjectMeta {
                    annotations: Some(annots.clone()),
                    ..Default::default()
                },
                ..Default::default()
            }
        },
        {
            let mut set = HashSet::new();
            set.insert("foo".to_string());
            set.insert("baz".to_string());
            set
        }
    )]
    fn test_get_resource_annotation_keys_deployment(
        #[case] deployment: Deployment,
        #[case] expected: HashSet<String>,
    ) {
        let req = ValidationRequest {
            request: KubernetesAdmissionRequest {
                object: to_value(&deployment).unwrap(),
                ..Default::default()
            },
            settings: Settings(BaseSettings::ContainsAnyOf {
                values: HashSet::new(),
            }),
        };
        let result = get_resource_annotation_keys(&req);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        // Settings require two annotations, Ingress with those annotations
        {
            let mut set = HashSet::new();
            set.insert("foo".to_string());
            set.insert("bar".to_string());
            Settings(BaseSettings::ContainsAllOf { values: set })
        },
        {
            use Ingress;
            use ObjectMeta;
            let mut annots = BTreeMap::new();
            annots.insert("foo".to_string(), "x".to_string());
            annots.insert("bar".to_string(), "y".to_string());
            Ingress {
                metadata: ObjectMeta {
                    annotations: Some(annots),
                    ..Default::default()
                },
                ..Default::default()
            }
        },
        true
    )]
    fn test_settings_validate_ingress_settings(
        #[case] settings: Settings,
        #[case] ingress: Ingress,
        #[case] expected: bool,
    ) {
        // Validate settings structure itself
        assert!(settings.validate().is_ok());

        // Prepare ValidationRequest with the ingress object
        let req = ValidationRequest {
            request: KubernetesAdmissionRequest {
                object: to_value(&ingress).unwrap(),
                ..Default::default()
            },
            settings: settings.clone(),
        };

        // Extract annotation keys from ingress
        let annots = get_resource_annotation_keys(&req);

        // Validate the annotation keys against the settings
        let result = crate::validate_annotations(&settings.clone(), &annots).is_ok();
        assert_eq!(result, expected);
    }
}
