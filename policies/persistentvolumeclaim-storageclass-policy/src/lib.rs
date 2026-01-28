use std::collections::HashSet;

use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::api::core::v1::{PersistentVolumeClaim, PersistentVolumeClaimSpec};

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
    let settings = &validation_request.settings;

    let pvc =
        match serde_json::from_value::<PersistentVolumeClaim>(validation_request.request.object) {
            Ok(pvc) => pvc,
            Err(_) => {
                // Not PVC, so we don't need to validate it
                return kubewarden::accept_request();
            }
        };

    let storage_class_name = pvc
        .spec
        .as_ref()
        .and_then(|spec| spec.storage_class_name.clone())
        .unwrap_or_default();

    if let Some(allowed_storage_classes) = &settings.allowed_storage_classes {
        return validate_allowed_classes(
            pvc,
            storage_class_name,
            allowed_storage_classes,
            settings.fallback_storage_class.as_ref(),
        );
    }

    if let Some(denied_storage_classes) = &settings.denied_storage_classes {
        return validate_denied_classes(
            pvc,
            storage_class_name,
            denied_storage_classes,
            settings.fallback_storage_class.as_ref(),
        );
    }

    // this should never happen because settings validation should have failed
    unreachable!()
}

fn validate_allowed_classes(
    pvc: PersistentVolumeClaim,
    storage_class_name: String,
    allowed_storage_classes: &HashSet<String>,
    fallback_storage_class: Option<&String>,
) -> CallResult {
    if allowed_storage_classes.contains(&storage_class_name) {
        return kubewarden::accept_request();
    }
    mutate_or_reject(pvc, storage_class_name, fallback_storage_class)
}

fn validate_denied_classes(
    pvc: PersistentVolumeClaim,
    storage_class_name: String,
    denied_storage_classes: &HashSet<String>,
    fallback_storage_class: Option<&String>,
) -> CallResult {
    if denied_storage_classes.contains(&storage_class_name) {
        return mutate_or_reject(pvc, storage_class_name, fallback_storage_class);
    }
    kubewarden::accept_request()
}

// Function to mutate the PVC request to use the fallback storage class
fn mutate_pvc(pvc: PersistentVolumeClaim, fallback_storage_class: &str) -> PersistentVolumeClaim {
    PersistentVolumeClaim {
        spec: Some(PersistentVolumeClaimSpec {
            storage_class_name: Some(fallback_storage_class.to_string()),
            ..pvc.spec.unwrap_or_default()
        }),
        ..pvc
    }
}

// This function is the common logic to either allow or deny list a storage class.
// If a fallback storage class is set, the request will be mutated to use it.
// If not, the request will be rejected.
fn mutate_or_reject(
    pvc: PersistentVolumeClaim,
    storage_class_name: String,
    fallback_storage_class: Option<&String>,
) -> CallResult {
    if let Some(fallback_storage_class) = fallback_storage_class {
        let mutated_pvc = mutate_pvc(pvc, fallback_storage_class);
        let json_value = serde_json::to_value(&mutated_pvc)?;
        return kubewarden::mutate_request(json_value);
    }

    kubewarden::reject_request(
        Some(format!(
            "storage class \"{}\" is not allowed",
            storage_class_name,
        )),
        None,
        None,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden::response::ValidationResponse;
    use rstest::rstest;

    fn make_pvc(storage_class_name: &str) -> PersistentVolumeClaim {
        PersistentVolumeClaim {
            spec: Some(PersistentVolumeClaimSpec {
                storage_class_name: Some(storage_class_name.to_owned()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[rstest]
    #[case::reject_pvc_using_not_allowed_class(vec!["slow", "standard"], "fast", false, None)]
    #[case::accept_pvc_doing_a_mutation(vec!["slow", "standard"], "fast", true, Some("standard"))]
    #[case::accept_pvc_using_allowed_class(vec!["slow", "standard"], "slow", true, None)]
    fn validate_allowed_classes_test(
        #[case] allowed_storage_classes: Vec<&str>,
        #[case] storage_class_name: &str,
        #[case] should_accept: bool,
        #[case] fallback_storage_class: Option<&str>,
    ) {
        let allowed_storage_classes: HashSet<String> = allowed_storage_classes
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let pvc = make_pvc(storage_class_name);

        let result = validate_allowed_classes(
            pvc,
            storage_class_name.to_owned(),
            &allowed_storage_classes,
            fallback_storage_class.map(|s| s.to_owned()).as_ref(),
        )
        .expect("CallResult should be Ok");

        let response: ValidationResponse =
            serde_json::from_slice(result.as_slice()).expect("Response should be valid JSON");
        assert!(should_accept == response.accepted);

        if should_accept {
            let result = check_mutatation(response.mutated_object, fallback_storage_class);
            assert!(result.is_ok(), "{}", result.err().unwrap());
        }
    }

    #[rstest]
    #[case::reject_pvc_using_denied_class(vec!["slow", "standard"], "slow", false, None)]
    #[case::accept_pvc_doing_a_mutation(vec!["slow", "fast"], "slow", true, Some("standard"))]
    #[case::accept_pvc_using_not_denied_class(vec!["slow", "standard"], "fast", true, None)]
    fn validate_denied_classes_test(
        #[case] denied_storage_classes: Vec<&str>,
        #[case] storage_class_name: &str,
        #[case] should_accept: bool,
        #[case] fallback_storage_class: Option<&str>,
    ) {
        let denied_storage_classes: HashSet<String> = denied_storage_classes
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let pvc = make_pvc(storage_class_name);

        let result = validate_denied_classes(
            pvc,
            storage_class_name.to_owned(),
            &denied_storage_classes,
            fallback_storage_class.map(|s| s.to_owned()).as_ref(),
        )
        .expect("CallResult should be Ok");

        let response: ValidationResponse =
            serde_json::from_slice(result.as_slice()).expect("Response should be valid JSON");
        assert!(should_accept == response.accepted);

        if should_accept {
            let result = check_mutatation(response.mutated_object, fallback_storage_class);
            assert!(result.is_ok(), "{}", result.err().unwrap());
        }
    }

    fn check_mutatation(
        mutated_object: Option<serde_json::Value>,
        expected_storage_class: Option<&str>,
    ) -> Result<(), String> {
        if expected_storage_class.is_none() && mutated_object.is_none() {
            // no mutation expected and none found
            return Ok(());
        }

        if expected_storage_class.is_some() && mutated_object.is_none() {
            return Err("Expected a mutated object, but none was found".to_string());
        }

        if expected_storage_class.is_none() && mutated_object.is_some() {
            return Err("Did not expect a mutated object, but one was found".to_string());
        }

        let mutated_pvc = serde_json::from_value::<PersistentVolumeClaim>(mutated_object.unwrap())
            .expect("Mutated object should be a valid PVC");
        let mutated_pvc_storage_class = mutated_pvc
            .spec
            .expect("Missing spec")
            .storage_class_name
            .expect("Missing storage class name");

        let expected_storage_class = expected_storage_class.unwrap();

        if mutated_pvc_storage_class != expected_storage_class {
            return Err(format!(
                "Expected storage class \"{}\", but found \"{}\"",
                expected_storage_class, mutated_pvc_storage_class
            ));
        }

        Ok(())
    }
}
