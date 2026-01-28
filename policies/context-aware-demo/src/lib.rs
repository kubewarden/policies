use anyhow::{Result, anyhow};
use guest::prelude::*;
use k8s_openapi::{
    List,
    api::core::v1::{Namespace, Pod, Secret},
};
use kubewarden_policy_sdk::wapc_guest as guest;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{
    host_capabilities::kubernetes::{GetResourceRequest, ListResourcesByNamespaceRequest},
    protocol_version_guest,
    request::ValidationRequest,
    validate_settings,
};

#[cfg(test)]
use crate::tests::mock_kubernetes_sdk::{get_resource, list_resources_by_namespace};
#[cfg(not(test))]
use kubewarden::host_capabilities::kubernetes::{get_resource, list_resources_by_namespace};

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

    let mut pod = serde_json::from_value::<Pod>(validation_request.request.object)?;

    let kube_request = GetResourceRequest {
        name: validation_request.request.namespace,
        api_version: "v1".to_string(),
        kind: "Namespace".to_string(),
        namespace: None,
        disable_cache: false,
    };

    let namespace: Namespace = get_resource(&kube_request)?;

    let namespace_annotations = namespace.metadata.annotations.unwrap_or_default();
    let mut labels_changed = false;
    let mut pod_labels = pod.metadata.labels.unwrap_or_default();

    for (key, value) in namespace_annotations.iter() {
        if key.starts_with("propagate.") {
            let patched_key = key
                .strip_prefix("propagate.")
                .ok_or_else(|| anyhow!("strip prefix should always return something"))?;
            pod_labels
                .entry(patched_key.to_owned())
                .and_modify(|v| {
                    if v != value {
                        value.clone_into(v);
                        labels_changed = true;
                    }
                })
                .or_insert_with(|| {
                    labels_changed = true;
                    value.to_owned()
                });
        }
    }

    if validation_request.settings.evil {
        let _err = steal_secrets();
        // we don't complain about what might go wrong while we do
        // evil things!
    }

    if labels_changed {
        pod.metadata.labels = Some(pod_labels);
        kubewarden::mutate_request(serde_json::to_value(pod)?)
    } else {
        kubewarden::accept_request()
    }
}

fn steal_secrets() -> Result<()> {
    println!("🦹: going to steal secrets");

    let kube_request = ListResourcesByNamespaceRequest {
        api_version: "v1".to_string(),
        kind: "Secret".to_string(),
        namespace: "kube-system".to_string(),
        label_selector: None,
        field_selector: None,
    };

    let secrets: List<Secret> = list_resources_by_namespace(&kube_request)?;
    let out = serde_json::to_string_pretty(&secrets)?;
    println!("🦹: Secrets `kube-system`:\n{out}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kubewarden::{request::KubernetesAdmissionRequest, response::ValidationResponse};
    use mockall::automock;
    use rstest::*;
    use serde_json::json;
    use serial_test::serial;
    use std::collections::BTreeMap;

    #[automock]
    pub mod kubernetes_sdk {
        use kubewarden::host_capabilities::kubernetes::{
            GetResourceRequest, ListResourcesByNamespaceRequest,
        };

        #[allow(dead_code)]
        pub fn get_resource<T: 'static>(_req: &GetResourceRequest) -> anyhow::Result<T> {
            Err(anyhow::anyhow!("not mocked"))
        }

        #[allow(dead_code)]
        pub fn list_resources_by_namespace<T>(
            _req: &ListResourcesByNamespaceRequest,
        ) -> anyhow::Result<k8s_openapi::List<T>>
        where
            T: k8s_openapi::ListableResource + serde::de::DeserializeOwned + Clone + 'static,
        {
            Err(anyhow::anyhow!("not mocked"))
        }
    }

    #[rstest]
    #[case(
        json!({
            "propagate.hello": "world",
            "foo": "bar",
        }),
        json!({
            "hello": "world",
            "ciao": "mondo",
        }),
        false,
    )]
    #[case(
        json!({
            "foo": "bar",
        }),
        json!({
            "ciao": "mondo",
        }),
        false,
    )]
    #[case(
        json!({
            "propagate.hello": "world",
            "foo": "bar",
        }),
        json!({
            "ciao": "mondo",
        }),
        true,
    )]
    #[case(
        json!({
            "propagate.hello": "world",
            "foo": "bar",
        }),
        json!({
            "hello": "mondo",
        }),
        true,
    )]
    #[case(
        json!({
            "propagate.hello": "world",
            "foo": "bar",
        }),
        json!({
        }),
        true,
    )]
    #[case(
        json!({
        }),
        json!({
            "ciao": "mondo",
        }),
        false,
    )]
    #[serial]
    fn no_mutation_no_evil(
        #[case] ns_annotations: serde_json::Value,
        #[case] pod_labels: serde_json::Value,
        #[case] should_mutate: bool,
    ) {
        let namespace_name = "test-namespace".to_string();

        let ns_annotations: BTreeMap<String, String> =
            serde_json::from_value(ns_annotations).expect("cannot deserialize ns labels");

        let namespace = Namespace {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(namespace_name.clone()),
                annotations: Some(ns_annotations.clone()),
                ..Default::default()
            },
            ..Default::default()
        };

        let pod_labels: BTreeMap<String, String> =
            serde_json::from_value(pod_labels).expect("cannot deserialize pod labels");
        let pod = Pod {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("testing-pod".to_string()),
                labels: Some(pod_labels),
                ..Default::default()
            },
            ..Default::default()
        };

        let settings = Settings::default();
        let request = KubernetesAdmissionRequest {
            namespace: namespace_name.clone(),
            object: serde_json::to_value(pod).expect("cannot serialize Pod"),
            ..Default::default()
        };
        let validation_request = ValidationRequest::<Settings> { settings, request };
        let payload = serde_json::to_string(&validation_request)
            .expect("cannot serialize validation request");

        let ctx_get_resource = mock_kubernetes_sdk::get_resource_context();
        ctx_get_resource
            .expect::<Namespace>()
            .times(1)
            .returning(move |req| {
                if req.name != namespace_name {
                    Err(anyhow!("it's not searching the expected Namespace"))
                } else {
                    Ok(namespace.clone())
                }
            });
        let ctx_list_resources = mock_kubernetes_sdk::list_resources_by_namespace_context();
        ctx_list_resources.expect::<Secret>().times(0);

        let response = validate(payload.as_bytes());
        assert!(response.is_ok());
        let validation_response: ValidationResponse = serde_json::from_slice(&response.unwrap())
            .expect("cannot deserialize validation_response");

        assert!(validation_response.accepted);
        if should_mutate && validation_response.mutated_object.is_none() {
            panic!("should have been mutated");
        }
        if !should_mutate && validation_response.mutated_object.is_some() {
            panic!("should not have been mutated");
        }
    }

    #[test]
    #[serial]
    fn evil() {
        let namespace_name = "test-namespace".to_string();

        let namespace = Namespace {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(namespace_name.clone()),
                ..Default::default()
            },
            ..Default::default()
        };

        let pod = Pod {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("testing-pod".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let settings = Settings { evil: true };
        let request = KubernetesAdmissionRequest {
            namespace: namespace_name.clone(),
            object: serde_json::to_value(pod).expect("cannot serialize Pod"),
            ..Default::default()
        };
        let validation_request = ValidationRequest::<Settings> { settings, request };
        let payload = serde_json::to_string(&validation_request)
            .expect("cannot serialize validation request");

        let ctx_get_resource = mock_kubernetes_sdk::get_resource_context();
        ctx_get_resource
            .expect::<Namespace>()
            .times(1)
            .returning(move |req| {
                if req.name != namespace_name {
                    Err(anyhow!("it's not searching the expected Namespace"))
                } else {
                    Ok(namespace.clone())
                }
            });
        let ctx_list_resources = mock_kubernetes_sdk::list_resources_by_namespace_context();
        ctx_list_resources
            .expect::<Secret>()
            .times(1)
            .returning(|_req| Err(anyhow!("this is going to be blocked")));

        // we don't care about the returned value, we just want to be sure the
        // secrets are listed. The assertion is done by mockall
        let _response = validate(payload.as_bytes());
    }
}
