use std::collections::HashSet;

use k8s_openapi::api::{
    admissionregistration::v1::{MutatingWebhookConfiguration, ValidatingWebhookConfiguration},
    core::v1::Service,
    networking::v1::Ingress,
};

use crate::service_details::ServiceDetails;

pub(crate) trait ServiceFinder {
    /// Find all the services that are defined inside of the object
    ///
    /// Return a unique list of ServiceDetails objects
    fn get_services(&self) -> HashSet<ServiceDetails>;
}

impl ServiceFinder for ValidatingWebhookConfiguration {
    fn get_services(&self) -> HashSet<ServiceDetails> {
        self.webhooks
            .as_ref()
            .map(|webhooks| {
                webhooks
                    .iter()
                    .filter_map(|webhook| {
                        webhook.client_config.service.as_ref().map(|svc| svc.into())
                    })
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default()
    }
}

impl ServiceFinder for MutatingWebhookConfiguration {
    fn get_services(&self) -> HashSet<ServiceDetails> {
        self.webhooks
            .as_ref()
            .map(|webhooks| {
                webhooks
                    .iter()
                    .filter_map(|webhook| {
                        webhook.client_config.service.as_ref().map(|svc| svc.into())
                    })
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default()
    }
}

impl ServiceFinder for Ingress {
    /// Returns a HashSet of ServiceDetails for all backend services referenced by this Ingress.
    /// This includes services referenced in the default backend and in all HTTP rules.
    fn get_services(&self) -> HashSet<ServiceDetails> {
        let mut services: HashSet<ServiceDetails> = HashSet::new();
        if self.spec.is_none() {
            return services;
        }

        let namespace = self.metadata.namespace.clone().unwrap_or_default();
        let spec = self.spec.as_ref().unwrap();

        if let Some(service_backend) = spec
            .default_backend
            .as_ref()
            .and_then(|default_backend| default_backend.service.as_ref())
        {
            let service_details = ServiceDetails::from_service_backend(&namespace, service_backend);
            services.insert(service_details);
        }

        if spec.rules.is_none() {
            return services;
        }
        let rules = spec.rules.as_ref().unwrap();

        for rule in rules {
            let ingress_svcs = rule
                .http
                .as_ref()
                .map(|http| {
                    http.paths
                        .iter()
                        .filter_map(|path| path.backend.service.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            services.extend(ingress_svcs.iter().map(|service_backend| {
                ServiceDetails::from_service_backend(&namespace, service_backend)
            }));
        }

        services
    }
}

impl ServiceFinder for Service {
    /// Returns a HashSet of ServiceDetails, one for each port defined in the Service, creating all
    /// possible service-port combinations that may be exposed.
    fn get_services(&self) -> HashSet<ServiceDetails> {
        let mut services = HashSet::new();
        let namespace = self.metadata.namespace.clone().unwrap_or_default();
        if let Some(spec) = &self.spec
            && let Some(ports) = &spec.ports
        {
            for port in ports {
                services.insert(ServiceDetails {
                    name: self.metadata.name.clone().unwrap_or_default(),
                    namespace: namespace.clone(),
                    port_number: Some(port.port),
                });
            }
        }
        services
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::networking::v1::IngressServiceBackend;

    #[test]
    fn find_services_used_by_ingress() {
        let service_backend = IngressServiceBackend {
            name: "test-service".to_string(),
            port: Some(k8s_openapi::api::networking::v1::ServiceBackendPort {
                number: Some(80),
                ..Default::default()
            }),
        };
        let default_service_backend = IngressServiceBackend {
            name: "default-service".to_string(),
            port: Some(k8s_openapi::api::networking::v1::ServiceBackendPort {
                number: Some(8080),
                ..Default::default()
            }),
        };

        let ingress = Ingress {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                namespace: Some("test-namespace".to_string()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::networking::v1::IngressSpec {
                default_backend: Some(k8s_openapi::api::networking::v1::IngressBackend {
                    service: Some(default_service_backend.clone()),
                    ..Default::default()
                }),
                rules: Some(vec![k8s_openapi::api::networking::v1::IngressRule {
                    http: Some(k8s_openapi::api::networking::v1::HTTPIngressRuleValue {
                        paths: vec![
                            k8s_openapi::api::networking::v1::HTTPIngressPath {
                                backend: k8s_openapi::api::networking::v1::IngressBackend {
                                    service: Some(service_backend.clone()),
                                    ..Default::default()
                                },
                                path_type: "Exact".to_string(),
                                path: Some("/one".to_string()),
                            },
                            k8s_openapi::api::networking::v1::HTTPIngressPath {
                                backend: k8s_openapi::api::networking::v1::IngressBackend {
                                    service: Some(service_backend.clone()),
                                    ..Default::default()
                                },
                                path_type: "Exact".to_string(),
                                path: Some("/two".to_string()),
                            },
                        ],
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let expected_service_details = ServiceDetails {
            name: "test-service".to_string(),
            namespace: "test-namespace".to_string(),
            port_number: Some(80),
        };
        let expected_default_service_details = ServiceDetails {
            name: "default-service".to_string(),
            namespace: "test-namespace".to_string(),
            port_number: Some(8080),
        };

        let services = ingress.get_services();
        assert_eq!(services.len(), 2);
        assert!(services.contains(&expected_service_details));
        assert!(services.contains(&expected_default_service_details));
    }

    #[test]
    fn find_services_used_by_validating_webhook_configuration() {
        let webhook_service_backend =
            k8s_openapi::api::admissionregistration::v1::ServiceReference {
                name: "webhook-service".to_string(),
                namespace: "webhook-namespace".to_string(),
                port: Some(443),
                path: None,
            };

        let validating_webhook_configuration = ValidatingWebhookConfiguration {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                namespace: Some("webhook-namespace".to_string()),
                ..Default::default()
            },
            webhooks: Some(vec![
                k8s_openapi::api::admissionregistration::v1::ValidatingWebhook {
                    client_config:
                        k8s_openapi::api::admissionregistration::v1::WebhookClientConfig {
                            service: Some(webhook_service_backend.clone()),
                            ..Default::default()
                        },
                    ..Default::default()
                },
            ]),
        };

        let expected_service_details = ServiceDetails {
            name: "webhook-service".to_string(),
            namespace: "webhook-namespace".to_string(),
            port_number: Some(443),
        };

        let services = validating_webhook_configuration.get_services();
        assert_eq!(services.len(), 1);
        assert!(services.contains(&expected_service_details));
    }

    #[test]
    fn find_services_used_by_mutating_webhook_configuration() {
        let webhook_service_backend =
            k8s_openapi::api::admissionregistration::v1::ServiceReference {
                name: "webhook-service".to_string(),
                namespace: "webhook-namespace".to_string(),
                port: Some(443),
                path: None,
            };

        let validating_webhook_configuration = MutatingWebhookConfiguration {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                namespace: Some("webhook-namespace".to_string()),
                ..Default::default()
            },
            webhooks: Some(vec![
                k8s_openapi::api::admissionregistration::v1::MutatingWebhook {
                    client_config:
                        k8s_openapi::api::admissionregistration::v1::WebhookClientConfig {
                            service: Some(webhook_service_backend.clone()),
                            ..Default::default()
                        },
                    ..Default::default()
                },
            ]),
        };

        let expected_service_details = ServiceDetails {
            name: "webhook-service".to_string(),
            namespace: "webhook-namespace".to_string(),
            port_number: Some(443),
        };

        let services = validating_webhook_configuration.get_services();
        assert_eq!(services.len(), 1);
        assert!(services.contains(&expected_service_details));
    }
}
