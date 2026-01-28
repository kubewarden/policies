use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use anyhow::{Result, anyhow};
use k8s_openapi::api::core::v1 as apicore;
use lazy_static::lazy_static;
use slog::{Logger, o, warn};
use std::collections::BTreeMap;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{
    host_capabilities::kubernetes::GetResourceRequest, logging, protocol_version_guest,
    request::ValidationRequest, validate_settings,
};

#[cfg(test)]
use crate::tests::mock_kubernetes_sdk::get_resource;
#[cfg(not(test))]
use kubewarden::host_capabilities::kubernetes::get_resource;

mod custom_resources;
use custom_resources::Project;

mod settings;
use settings::Settings;

const RANCHER_PROJECT_ID_LABEL: &str = "field.cattle.io/projectId";

lazy_static! {
    static ref LOG_DRAIN: Logger = Logger::root(
        logging::KubewardenDrain::new(),
        o!("policy" => "rancher-project-propagate-labels")
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

    let namespace =
        serde_json::from_value::<apicore::Namespace>(validation_request.request.object)?;

    let cluster_project_tuple: Option<(String, String)> = namespace
        .metadata
        .annotations
        .as_ref()
        .and_then(|annotations| annotations.get(RANCHER_PROJECT_ID_LABEL).cloned())
        .map(|annotation| {
            let tokens: Vec<&str> = annotation.split(':').collect();
            if tokens.len() != 2 {
                Err(anyhow!(
                    "malformed value of {} annotation",
                    RANCHER_PROJECT_ID_LABEL
                ))
            } else {
                Ok((tokens[0].to_owned(), tokens[1].to_owned()))
            }
        })
        .transpose()?;

    match cluster_project_tuple {
        None => kubewarden::accept_request(),
        Some((cluster_id, project_id)) => propagate_labels(
            &cluster_id,
            &project_id,
            &namespace,
            &validation_request.settings,
        ),
    }
}

fn propagate_labels(
    cluster_id: &str,
    project_id: &str,
    namespace: &apicore::Namespace,
    settings: &Settings,
) -> CallResult {
    if cluster_id != "local" {
        let msg = "Namespace belongs to project defined inside of an upstream cluster. This is not supported yet, accepting the request";

        let namespace_name = namespace
            .metadata
            .name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| String::from("NOT SET"));
        warn!(
            LOG_DRAIN,
            "{}", msg;
            "namespace" => namespace_name,
            "project_id" => project_id,
            "cluster_id" => cluster_id,
        );

        return match settings.downstream_cluster_failure_mode {
            settings::FailureMode::Fail => {
                kubewarden::reject_request(Some(msg.to_string()), None, None, None)
            }
            settings::FailureMode::Ignore => kubewarden::accept_request(),
        };
    }

    let req = GetResourceRequest {
        api_version: "management.cattle.io/v3".to_string(),
        kind: "Project".to_string(),
        name: project_id.to_string(),
        namespace: Some(cluster_id.to_string()),
        disable_cache: true,
    };
    let project: Project = get_resource(&req)?;

    match merge_labels(
        &project.metadata.labels.unwrap_or_default(),
        namespace.metadata.labels.as_ref(),
    )? {
        Some(new_labels) => {
            let mut patched_namespace = namespace.clone();
            patched_namespace.metadata.labels = Some(new_labels);
            kubewarden::mutate_request(serde_json::to_value(patched_namespace)?)
        }
        None => kubewarden::accept_request(),
    }
}

fn merge_labels(
    project_labels: &BTreeMap<String, String>,
    namespace_labels: Option<&BTreeMap<String, String>>,
) -> Result<Option<BTreeMap<String, String>>> {
    let mut labels_changed = false;
    let mut namespace_labels = match namespace_labels {
        Some(labels) => labels.to_owned(),
        None => BTreeMap::<String, String>::new(),
    };

    for (key, value) in project_labels.iter() {
        if key.starts_with("propagate.") {
            let patched_key = key
                .strip_prefix("propagate.")
                .ok_or_else(|| anyhow!("strip prefix should always return something"))?;
            namespace_labels
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

    if labels_changed {
        Ok(Some(namespace_labels))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::FailureMode;

    use super::*;
    use kubewarden::{request::KubernetesAdmissionRequest, response::ValidationResponse};
    use mockall::automock;
    use rstest::*;
    use serde_json::json;
    use serial_test::serial;

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
        // prj label is already defined inside of ns with the same value
        json!({
            "propagate.hello": "world",
            "foo": "bar",
        }),
        Some(json!({
            "hello": "world",
            "ciao": "mondo",
        })),
        None,
    )]
    #[case(
        // prj label is already defined inside of ns but with different value
        json!({
            "propagate.hello": "world",
            "foo": "bar",
        }),
        Some(json!({
            "hello": "world2",
            "ciao": "mondo",
        })),
        Some(json!({
            "hello": "world",
            "ciao": "mondo",
        })),
    )]
    #[case(
        // no labels to propagate from the prj
        json!({
            "foo": "bar",
        }),
        Some(json!({
            "ciao": "mondo",
        })),
        None,
    )]
    #[case(
        // label is missing from the ns
        json!({
            "propagate.hi": "world",
            "foo": "bar",
        }),
        None,
        Some(json!({
            "hi": "world",
        })),
    )]
    fn test_merge_labels(
        #[case] prj_labels: serde_json::Value,
        #[case] ns_labels: Option<serde_json::Value>,
        #[case] expected: Option<serde_json::Value>,
    ) {
        let project_labels: BTreeMap<String, String> =
            serde_json::from_value(prj_labels).expect("cannot deserialize project labels");

        let namespace_labels: Option<BTreeMap<String, String>> = ns_labels.map(|labels| {
            serde_json::from_value(labels).expect("cannot deserialize namespace labels")
        });

        let expected_labels: Option<BTreeMap<String, String>> = expected.map(|labels| {
            serde_json::from_value(labels).expect("cannot deserialize expected labels")
        });

        let actual = merge_labels(&project_labels, namespace_labels.as_ref())
            .expect("merge should not fail");

        assert_eq!(expected_labels, actual);
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
    fn local_cluster(
        #[case] prj_labels: serde_json::Value,
        #[case] ns_labels: serde_json::Value,
        #[case] should_mutate: bool,
    ) {
        let prj_name = "test-namespace".to_string();
        let cluster_id = "local";

        let project_labels: BTreeMap<String, String> =
            serde_json::from_value(prj_labels).expect("cannot deserialize ns labels");

        let project = Project {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(prj_name.clone()),
                labels: Some(project_labels.clone()),
                ..Default::default()
            },
            ..Default::default()
        };

        let namespace_labels: BTreeMap<String, String> =
            serde_json::from_value(ns_labels).expect("cannot deserialize pod labels");

        let mut namespace_annotations: BTreeMap<String, String> = BTreeMap::new();
        namespace_annotations.insert(
            RANCHER_PROJECT_ID_LABEL.to_string(),
            format!("{cluster_id}:{prj_name}"),
        );

        let namespace = apicore::Namespace {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("testing-namespace".to_string()),
                labels: Some(namespace_labels),
                annotations: Some(namespace_annotations),
                ..Default::default()
            },
            ..Default::default()
        };

        let settings = Settings::default();
        let request = KubernetesAdmissionRequest {
            object: serde_json::to_value(namespace).expect("cannot serialize Namespace"),
            ..Default::default()
        };
        let validation_request = ValidationRequest::<Settings> { settings, request };
        let payload = serde_json::to_string(&validation_request)
            .expect("cannot serialize validation request");

        let ctx_get_resource = mock_kubernetes_sdk::get_resource_context();
        ctx_get_resource
            .expect::<Project>()
            .times(1)
            .returning(move |req| {
                if req.name != prj_name {
                    Err(anyhow!("it's not searching the expected Project"))
                } else {
                    Ok(project.clone())
                }
            });

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

    #[rstest]
    #[case(FailureMode::Ignore, true)]
    #[case(FailureMode::Fail, false)]
    #[serial]
    fn downstream_cluster(#[case] failure_mode: FailureMode, #[case] accepted: bool) {
        let prj_name = "test-namespace".to_string();
        let cluster_id = "downstream-cluster-1";

        let mut namespace_annotations: BTreeMap<String, String> = BTreeMap::new();
        namespace_annotations.insert(
            RANCHER_PROJECT_ID_LABEL.to_string(),
            format!("{cluster_id}:{prj_name}"),
        );

        let namespace = apicore::Namespace {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("testing-namespace".to_string()),
                annotations: Some(namespace_annotations),
                ..Default::default()
            },
            ..Default::default()
        };

        let settings = Settings {
            downstream_cluster_failure_mode: failure_mode,
        };
        let request = KubernetesAdmissionRequest {
            object: serde_json::to_value(namespace).expect("cannot serialize Namespace"),
            ..Default::default()
        };
        let validation_request = ValidationRequest::<Settings> { settings, request };
        let payload = serde_json::to_string(&validation_request)
            .expect("cannot serialize validation request");

        let ctx_get_resource = mock_kubernetes_sdk::get_resource_context();
        ctx_get_resource.expect::<Project>().times(0);

        let response = validate(payload.as_bytes());
        assert!(response.is_ok());
        let validation_response: ValidationResponse = serde_json::from_slice(&response.unwrap())
            .expect("cannot deserialize validation_response");

        assert_eq!(accepted, validation_response.accepted);
    }
}
