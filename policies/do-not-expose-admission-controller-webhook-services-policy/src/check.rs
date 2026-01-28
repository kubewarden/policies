use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::service_details::ServiceDetails;
use crate::service_finder::ServiceFinder;

#[cfg(test)]
use crate::check::tests::mock_kubernetes_sdk::list_resources_by_namespace;
use k8s_openapi::{Resource, api::networking::v1::Ingress};
use kubewarden::host_capabilities::kubernetes::ListResourcesByNamespaceRequest;
#[cfg(not(test))]
use kubewarden::host_capabilities::kubernetes::list_resources_by_namespace;

/// Given a list of services being used by (Validating|Mutating)WebhookConfiguration, find all
/// the ones that are exposed by an Ingress resource, or by NodePort/LoadBalancer services.
pub(crate) fn find_webhook_services_exposed(
    services: &HashSet<ServiceDetails>,
) -> Result<HashSet<ServiceDetails>> {
    // Group the services by namespace, this is done to optimize the number of queries done to the
    // kubernetes API.
    // The map has the namespace as the key and a set of services as the value.
    let mut webhook_svcs_by_namespace: HashMap<&str, HashSet<&ServiceDetails>> = HashMap::new();
    for svc in services.iter() {
        webhook_svcs_by_namespace
            .entry(svc.namespace.as_str())
            .and_modify(|svcs| {
                svcs.insert(svc);
            })
            .or_insert([svc].into());
    }

    // List of Services exposed by ingresses, nodeport, loadbalancer, regardless of the namespace
    let mut exposed_services_being_used = HashSet::new();

    for (namespace, webhook_services_inside_namespace) in webhook_svcs_by_namespace.iter() {
        let svcs_exposed_by_ingress = find_webhook_services_exposed_by_ingress_inside_of_namespace(
            webhook_services_inside_namespace,
            namespace,
        )?;
        let svcs_exposed_by_nodeport_loadbalancer =
            find_webhook_services_exposed_by_nodeport_loadbalancer_inside_of_namespace(
                webhook_services_inside_namespace,
                namespace,
            )?;
        exposed_services_being_used.extend(svcs_exposed_by_ingress);
        exposed_services_being_used.extend(svcs_exposed_by_nodeport_loadbalancer);
    }

    Ok(exposed_services_being_used)
}

/// Given a list of services being used by (Validating|Mutating)WebhookConfiguration, find all
/// the ones that are exposed by an Ingress resource in the given namespace.
fn find_webhook_services_exposed_by_ingress_inside_of_namespace(
    webhook_services: &HashSet<&ServiceDetails>,
    namespace: &str,
) -> Result<HashSet<ServiceDetails>> {
    // Get all ingresses in the namespace
    let ingresses = list_resources_by_namespace::<k8s_openapi::api::networking::v1::Ingress>(
        &ListResourcesByNamespaceRequest {
            namespace: namespace.to_string(),
            api_version: Ingress::API_VERSION.to_string(),
            kind: Ingress::KIND.to_string(),
            label_selector: None,
            field_selector: None,
        },
    )?;

    // each ingress can refer to multiple services, build a unique set of services
    let mut svcs_exposed_by_ingresses: HashSet<ServiceDetails> = HashSet::new();
    for ingress in ingresses.items.iter() {
        svcs_exposed_by_ingresses.extend(ingress.get_services());
    }

    let svcs_ptr: HashSet<&ServiceDetails> = svcs_exposed_by_ingresses.iter().collect();

    // return the intersection of the services and the services exposed by ingresses
    Ok(svcs_ptr
        .intersection(webhook_services)
        .map(|s| (**s).clone())
        .collect())
}

/// Given a list of services being used by (Validating|Mutating)WebhookConfiguration, find all
/// the ones that are exposed by a NodePort or LoadBalancer Service in the given namespace.
fn find_webhook_services_exposed_by_nodeport_loadbalancer_inside_of_namespace(
    webhook_services: &HashSet<&ServiceDetails>,
    namespace: &str,
) -> Result<HashSet<ServiceDetails>> {
    // Get all Services in the namespace
    let services = list_resources_by_namespace::<k8s_openapi::api::core::v1::Service>(
        &ListResourcesByNamespaceRequest {
            namespace: namespace.to_string(),
            api_version: k8s_openapi::api::core::v1::Service::API_VERSION.to_string(),
            kind: k8s_openapi::api::core::v1::Service::KIND.to_string(),
            label_selector: None,
            field_selector: None,
        },
    )?;

    // each service can refer to multiple ports, build unique set of all possible service-port
    // pairs to correctly compare against webhook_services
    let mut svcs_exposed: HashSet<ServiceDetails> = HashSet::new();
    for service in services.items.iter() {
        if let Some(spec) = &service.spec
            && let Some(ref type_) = spec.type_
            && (type_ == "NodePort" || type_ == "LoadBalancer")
        {
            svcs_exposed.extend(service.get_services());
        }
    }

    let svcs_ptr: HashSet<&ServiceDetails> = svcs_exposed.iter().collect();

    // return the intersection of the services and the services exposed by NodePort, LoadBalancer
    Ok(svcs_ptr
        .intersection(webhook_services)
        .map(|s| (**s).clone())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    use mockall::automock;
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

    #[test]
    #[serial]
    fn test_find_services_exposed_no_ingress_nor_service_defined() {
        let mut services = HashSet::new();
        let expected_namespace = "my-namespace";
        services.insert(ServiceDetails {
            name: "my-service".to_string(),
            namespace: expected_namespace.to_string(),
            port_number: Some(80),
        });

        let ctx_list_resources_by_namespace =
            mock_kubernetes_sdk::list_resources_by_namespace_context();
        ctx_list_resources_by_namespace
            .expect::<Ingress>()
            .times(1)
            .returning(move |req| {
                if req.namespace != expected_namespace {
                    Err(anyhow::anyhow!("namespace mismatch"))
                } else {
                    Ok(k8s_openapi::List::<Ingress> {
                        items: vec![],
                        ..Default::default()
                    })
                }
            });
        ctx_list_resources_by_namespace
            .expect::<k8s_openapi::api::core::v1::Service>()
            .times(1)
            .returning(move |_req| {
                Ok(k8s_openapi::List::<k8s_openapi::api::core::v1::Service> {
                    items: vec![],
                    ..Default::default()
                })
            });

        let result = find_webhook_services_exposed(&services);
        assert!(result.is_ok());
        let exposed_services = result.unwrap();
        assert!(exposed_services.is_empty());
    }

    #[test]
    #[serial]
    fn test_find_services_exposed_ingress_nodeport_defined_no_match() {
        let mut services = HashSet::new();
        let expected_namespace = "my-namespace";
        services.insert(ServiceDetails {
            name: "my-service".to_string(),
            namespace: expected_namespace.to_string(),
            port_number: Some(80),
        });

        let ingress = Ingress {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                namespace: Some(expected_namespace.to_string()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::networking::v1::IngressSpec {
                default_backend: Some(k8s_openapi::api::networking::v1::IngressBackend {
                    service: Some(k8s_openapi::api::networking::v1::IngressServiceBackend {
                        name: "other-service".to_string(),
                        port: Some(k8s_openapi::api::networking::v1::ServiceBackendPort {
                            number: Some(80),
                            name: None,
                        }),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let nodeport = k8s_openapi::api::core::v1::Service {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("yet-another-service".to_string()),
                namespace: Some(expected_namespace.to_string()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::core::v1::ServiceSpec {
                type_: Some("NodePort".to_string()),
                ports: Some(vec![k8s_openapi::api::core::v1::ServicePort {
                    port: 80,
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let ctx_list_resources_by_namespace =
            mock_kubernetes_sdk::list_resources_by_namespace_context();
        ctx_list_resources_by_namespace
            .expect::<Ingress>()
            .times(1)
            .returning(move |req| {
                if req.namespace != expected_namespace {
                    Err(anyhow::anyhow!("namespace mismatch"))
                } else {
                    Ok(k8s_openapi::List::<Ingress> {
                        items: vec![ingress.clone()],
                        ..Default::default()
                    })
                }
            });
        ctx_list_resources_by_namespace
            .expect::<k8s_openapi::api::core::v1::Service>()
            .times(1)
            .returning(move |_req| {
                Ok(k8s_openapi::List::<k8s_openapi::api::core::v1::Service> {
                    items: vec![nodeport.clone()],
                    ..Default::default()
                })
            });

        let result = find_webhook_services_exposed(&services);
        assert!(result.is_ok());
        let exposed_services = result.unwrap();
        assert!(exposed_services.is_empty());
    }

    #[test]
    #[serial]
    fn test_find_services_exposed_ingress_defined_match() {
        let mut services = HashSet::new();
        let service_name = "my-service";
        let namespace = "my-namespace";
        services.insert(ServiceDetails {
            name: service_name.to_string(),
            namespace: namespace.to_string(),
            port_number: Some(80),
        });

        let ingress = Ingress {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::networking::v1::IngressSpec {
                default_backend: Some(k8s_openapi::api::networking::v1::IngressBackend {
                    service: Some(k8s_openapi::api::networking::v1::IngressServiceBackend {
                        name: service_name.to_string(),
                        port: Some(k8s_openapi::api::networking::v1::ServiceBackendPort {
                            number: Some(80),
                            name: None,
                        }),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let ctx_list_resources_by_namespace =
            mock_kubernetes_sdk::list_resources_by_namespace_context();
        ctx_list_resources_by_namespace
            .expect::<Ingress>()
            .times(1)
            .returning(move |req| {
                if req.namespace != namespace {
                    Err(anyhow::anyhow!("namespace mismatch"))
                } else {
                    Ok(k8s_openapi::List::<Ingress> {
                        items: vec![ingress.clone()],
                        ..Default::default()
                    })
                }
            });
        ctx_list_resources_by_namespace
            .expect::<k8s_openapi::api::core::v1::Service>()
            .times(1)
            .returning(move |_req| {
                Ok(k8s_openapi::List::<k8s_openapi::api::core::v1::Service> {
                    items: vec![],
                    ..Default::default()
                })
            });

        let result = find_webhook_services_exposed(&services);
        assert!(result.is_ok());
        let exposed_services = result.unwrap();
        assert_eq!(exposed_services.len(), 1);
    }

    #[test]
    #[serial]
    fn test_find_services_exposed_nodeport_defined_match() {
        let mut services = HashSet::new();
        let service_name = "my-service";
        let expected_namespace = "my-namespace";
        services.insert(ServiceDetails {
            name: "my-service".to_string(),
            namespace: expected_namespace.to_string(),
            port_number: Some(80),
        });

        let nodeport = k8s_openapi::api::core::v1::Service {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(service_name.to_string()),
                namespace: Some(expected_namespace.to_string()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::core::v1::ServiceSpec {
                type_: Some("NodePort".to_string()),
                ports: Some(vec![
                    // this port should not match
                    k8s_openapi::api::core::v1::ServicePort {
                        port: 81,
                        ..Default::default()
                    },
                    // this port should match
                    k8s_openapi::api::core::v1::ServicePort {
                        port: 80,
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let ctx_list_resources_by_namespace =
            mock_kubernetes_sdk::list_resources_by_namespace_context();
        ctx_list_resources_by_namespace
            .expect::<Ingress>()
            .times(1)
            .returning(move |req| {
                if req.namespace != expected_namespace {
                    Err(anyhow::anyhow!("namespace mismatch"))
                } else {
                    Ok(k8s_openapi::List::<Ingress> {
                        items: vec![],
                        ..Default::default()
                    })
                }
            });
        ctx_list_resources_by_namespace
            .expect::<k8s_openapi::api::core::v1::Service>()
            .times(1)
            .returning(move |_req| {
                Ok(k8s_openapi::List::<k8s_openapi::api::core::v1::Service> {
                    items: vec![nodeport.clone()],
                    ..Default::default()
                })
            });

        let result = find_webhook_services_exposed(&services);
        assert!(result.is_ok());
        let exposed_services = result.unwrap();
        assert_eq!(exposed_services.len(), 1);
    }

    #[test]
    #[serial]
    fn test_find_services_exposed_loadbalancer_defined_match() {
        let mut services = HashSet::new();
        let service_name = "my-service";
        let expected_namespace = "my-namespace";
        services.insert(ServiceDetails {
            name: "my-service".to_string(),
            namespace: expected_namespace.to_string(),
            port_number: Some(80),
        });

        let loadbalancer = k8s_openapi::api::core::v1::Service {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(service_name.to_string()),
                namespace: Some(expected_namespace.to_string()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::core::v1::ServiceSpec {
                type_: Some("LoadBalancer".to_string()),
                ports: Some(vec![k8s_openapi::api::core::v1::ServicePort {
                    port: 80,
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let ctx_list_resources_by_namespace =
            mock_kubernetes_sdk::list_resources_by_namespace_context();
        ctx_list_resources_by_namespace
            .expect::<Ingress>()
            .times(1)
            .returning(move |req| {
                if req.namespace != expected_namespace {
                    Err(anyhow::anyhow!("namespace mismatch"))
                } else {
                    Ok(k8s_openapi::List::<Ingress> {
                        items: vec![],
                        ..Default::default()
                    })
                }
            });
        ctx_list_resources_by_namespace
            .expect::<k8s_openapi::api::core::v1::Service>()
            .times(1)
            .returning(move |_req| {
                Ok(k8s_openapi::List::<k8s_openapi::api::core::v1::Service> {
                    items: vec![loadbalancer.clone()],
                    ..Default::default()
                })
            });

        let result = find_webhook_services_exposed(&services);
        assert!(result.is_ok());
        let exposed_services = result.unwrap();
        assert_eq!(exposed_services.len(), 1);
    }
}
