use guest::prelude::*;
use k8s_openapi::api::{
    apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet},
    batch::v1::{CronJob, Job},
    core::v1::{Pod, ReplicationController},
};
use kubewarden_policy_sdk::{
    accept_request, logging, protocol_version_guest, request::ValidationRequest, validate_settings,
};
use kubewarden_policy_sdk::{response::ValidationResponse, wapc_guest as guest};
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use slog::{Logger, o, warn};

mod validation_result;

mod validation;
use validation::validate_pod_spec;

mod validating_resource;
use validating_resource::ValidatingResource;

pub(crate) mod matchers;

mod settings;
use settings::Settings;

lazy_static! {
    static ref LOG_DRAIN: Logger = Logger::root(
        logging::KubewardenDrain::new(),
        o!("policy" => "trusted-repos")
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

    match validation_request.request.kind.kind.as_str() {
        "Deployment" => validate_resource::<Deployment>(validation_request),
        "ReplicaSet" => validate_resource::<ReplicaSet>(validation_request),
        "StatefulSet" => validate_resource::<StatefulSet>(validation_request),
        "DaemonSet" => validate_resource::<DaemonSet>(validation_request),
        "ReplicationController" => validate_resource::<ReplicationController>(validation_request),
        "Job" => validate_resource::<Job>(validation_request),
        "CronJob" => validate_resource::<CronJob>(validation_request),
        "Pod" => validate_resource::<Pod>(validation_request),
        _ => {
            // We were forwarded a request we cannot unmarshal or
            // understand, just accept it
            warn!(
                LOG_DRAIN,
                "cannot unmarshal resource: this policy does not know how to evaluate this resource; accept it"
            );
            accept_request()
        }
    }
}

// validate any resource that contains a Pod. e.g. Deployment, StatefulSet, ...
fn validate_resource<T: ValidatingResource + DeserializeOwned>(
    validation_request: ValidationRequest<Settings>,
) -> CallResult {
    let resource = match serde_json::from_value::<T>(validation_request.request.object.clone()) {
        Ok(resource) => resource,
        Err(_) => {
            // We were forwarded a request we cannot unmarshal or
            // understand, just accept it
            warn!(
                LOG_DRAIN,
                "cannot unmarshal resource: this policy does not know how to evaluate this resource; accept it"
            );
            return accept_request();
        }
    };

    let spec = match resource.spec() {
        Some(spec) => spec,
        None => {
            return accept_request();
        }
    };

    let validation_response: ValidationResponse =
        validate_pod_spec(&spec, &validation_request.settings).into();
    Ok(serde_json::to_vec(&validation_response)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::matchers::{registry::RegistryMatcher, string::StringMatcher};
    use crate::settings::Registries;

    use k8s_openapi::api::{
        apps::v1::{DaemonSetSpec, DeploymentSpec, ReplicaSetSpec, StatefulSetSpec},
        batch::v1::{CronJobSpec, JobSpec, JobTemplateSpec},
        core::v1::{Container, PodSpec, PodTemplateSpec, ReplicationControllerSpec},
    };
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use serde::Serialize;
    use serde_json::json;

    // A single allowed image (nginx:1.0.0 — docker.io, not in the reject list)
    // is used as the container image for all resource-kind cases.
    // The ingress case uses a raw JSON object to exercise the unknown-kind path.

    fn pod_spec_with(image: &str) -> PodSpec {
        PodSpec {
            containers: vec![Container {
                name: "app".to_string(),
                image: Some(image.to_string()),
                ..Container::default()
            }],
            ..PodSpec::default()
        }
    }

    fn pod_template(image: &str) -> PodTemplateSpec {
        PodTemplateSpec {
            spec: Some(pod_spec_with(image)),
            ..PodTemplateSpec::default()
        }
    }

    /// Build a raw `validate` payload from a serializable k8s object,
    /// specifying the admission request `kind` metadata manually.
    fn make_payload<T: Serialize>(
        object: &T,
        kind: &str,
        group: &str,
        settings: &Settings,
    ) -> Vec<u8> {
        let payload = json!({
            "settings": settings,
            "request": {
                "uid": "test-uid",
                "kind":        { "group": group, "kind": kind, "version": "v1" },
                "resource":    { "group": group, "version": "v1", "resource": kind.to_lowercase() },
                "requestKind": { "group": group, "kind": kind, "version": "v1" },
                "operation": "CREATE",
                "userInfo": {
                    "username": "alice",
                    "uid": "alice-uid",
                    "groups": ["system:authenticated"]
                },
                "object": serde_json::to_value(object).unwrap()
            }
        });
        serde_json::to_vec(&payload).unwrap()
    }

    fn assert_validate(payload: Vec<u8>, expected_accepted: bool) {
        let raw = validate(&payload).unwrap();
        let response: ValidationResponse = serde_json::from_slice(&raw).unwrap();
        assert_eq!(
            response.accepted, expected_accepted,
            "expected accepted={expected_accepted}, got {:?}",
            response
        );
    }

    fn reject_ghcr_and_docker_settings() -> Settings {
        Settings {
            registries: Registries {
                reject: vec![
                    RegistryMatcher(StringMatcher::Exact("ghcr.io".to_string())),
                    RegistryMatcher(StringMatcher::Exact("docker.io".to_string())),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    // --- resource-kind dispatch tests ---
    // Each test exercises one arm of the `match kind.kind.as_str()` switch.
    // The image nginx:1.0.0 normalises to docker.io/library/nginx:1.0.0, but
    // the registry extracted by the OCI parser is "docker.io", which IS in the
    // reject list — so expected_accepted is false for all workload kinds.
    // The ingress case hits the `_ =>` arm and is always accepted.

    #[test]
    fn test_validate_deployment() {
        let obj = Deployment {
            metadata: ObjectMeta {
                name: Some("nginx".to_string()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                template: pod_template("nginx:1.0.0"),
                ..DeploymentSpec::default()
            }),
            ..Default::default()
        };
        let settings = reject_ghcr_and_docker_settings();
        assert_validate(make_payload(&obj, "Deployment", "apps", &settings), false);
    }

    #[test]
    fn test_validate_replicaset() {
        let obj = ReplicaSet {
            metadata: ObjectMeta {
                name: Some("nginx".to_string()),
                ..Default::default()
            },
            spec: Some(ReplicaSetSpec {
                template: Some(pod_template("nginx:1.0.0")),
                ..ReplicaSetSpec::default()
            }),
            ..Default::default()
        };
        let settings = reject_ghcr_and_docker_settings();
        assert_validate(make_payload(&obj, "ReplicaSet", "apps", &settings), false);
    }

    #[test]
    fn test_validate_statefulset() {
        let obj = StatefulSet {
            metadata: ObjectMeta {
                name: Some("nginx".to_string()),
                ..Default::default()
            },
            spec: Some(StatefulSetSpec {
                template: pod_template("nginx:1.0.0"),
                ..StatefulSetSpec::default()
            }),
            ..Default::default()
        };
        let settings = reject_ghcr_and_docker_settings();
        assert_validate(make_payload(&obj, "StatefulSet", "apps", &settings), false);
    }

    #[test]
    fn test_validate_daemonset() {
        let obj = DaemonSet {
            metadata: ObjectMeta {
                name: Some("nginx".to_string()),
                ..Default::default()
            },
            spec: Some(DaemonSetSpec {
                template: pod_template("nginx:1.0.0"),
                ..DaemonSetSpec::default()
            }),
            ..Default::default()
        };
        let settings = reject_ghcr_and_docker_settings();
        assert_validate(make_payload(&obj, "DaemonSet", "apps", &settings), false);
    }

    #[test]
    fn test_validate_replicationcontroller() {
        let obj = ReplicationController {
            metadata: ObjectMeta {
                name: Some("nginx".to_string()),
                ..Default::default()
            },
            spec: Some(ReplicationControllerSpec {
                template: Some(pod_template("nginx:1.0.0")),
                ..ReplicationControllerSpec::default()
            }),
            ..Default::default()
        };
        let settings = reject_ghcr_and_docker_settings();
        assert_validate(
            make_payload(&obj, "ReplicationController", "", &settings),
            false,
        );
    }

    #[test]
    fn test_validate_job() {
        let obj = Job {
            metadata: ObjectMeta {
                name: Some("nginx".to_string()),
                ..Default::default()
            },
            spec: Some(JobSpec {
                template: pod_template("nginx:1.0.0"),
                ..JobSpec::default()
            }),
            ..Default::default()
        };
        let settings = reject_ghcr_and_docker_settings();
        assert_validate(make_payload(&obj, "Job", "batch", &settings), false);
    }

    #[test]
    fn test_validate_cronjob() {
        let obj = CronJob {
            metadata: ObjectMeta {
                name: Some("nginx".to_string()),
                ..Default::default()
            },
            spec: Some(CronJobSpec {
                schedule: "* * * * *".to_string(),
                job_template: JobTemplateSpec {
                    spec: Some(JobSpec {
                        template: pod_template("nginx:1.0.0"),
                        ..JobSpec::default()
                    }),
                    ..JobTemplateSpec::default()
                },
                ..CronJobSpec::default()
            }),
            ..Default::default()
        };
        let settings = reject_ghcr_and_docker_settings();
        assert_validate(make_payload(&obj, "CronJob", "batch", &settings), false);
    }

    #[test]
    fn test_validate_pod() {
        let obj = Pod {
            metadata: ObjectMeta {
                name: Some("nginx".to_string()),
                ..Default::default()
            },
            spec: Some(pod_spec_with("nginx:1.0.0")),
            ..Default::default()
        };
        let settings = reject_ghcr_and_docker_settings();
        assert_validate(make_payload(&obj, "Pod", "", &settings), false);
    }

    #[test]
    fn test_validate_unknown_kind_is_accepted() {
        // Ingress is not handled by this policy; the _ arm always accepts it.
        let obj = json!({
            "metadata": { "name": "my-ingress" },
            "spec": { "rules": [] }
        });
        let settings = reject_ghcr_and_docker_settings();
        assert_validate(
            make_payload(&obj, "Ingress", "networking.k8s.io", &settings),
            true,
        );
    }
}
