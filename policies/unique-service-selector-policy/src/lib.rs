use k8s_openapi::Resource;
use k8s_openapi::api::core::v1::{self as apicore, Service};
use kubewarden_policy_sdk::host_capabilities::kubernetes::{
    ListResourcesByNamespaceRequest, list_resources_by_namespace,
};

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::Settings;

use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn list_services(namespace: String) -> Result<Vec<Service>, String> {
    let request = ListResourcesByNamespaceRequest {
        api_version: "v1".to_owned(),
        kind: Service::KIND.to_owned(),
        namespace,
        label_selector: None,
        field_selector: None,
    };
    match list_resources_by_namespace(&request) {
        Ok(response) => Ok(response.items),
        Err(_) => Err("cannot list current services".to_owned()),
    }
}

// Checks if the given services share the same set of selectors. Returns true
// if both service the same selectors.
// Note: two services are allowed at the same time if they don't have any selector specified
fn services_have_same_selectors(svc1: &Service, svc2: &Service) -> bool {
    let selector_svc1 = svc1
        .spec
        .clone()
        .unwrap_or_default()
        .selector
        .unwrap_or_default();
    let selector_svc2 = svc2
        .spec
        .clone()
        .unwrap_or_default()
        .selector
        .unwrap_or_default();

    if selector_svc1.is_empty() && selector_svc2.is_empty() {
        return false;
    }
    selector_svc1 == selector_svc2
}

// Find the services that has the same selectors of the given service.
// Returns the names of the services that have the same selectors as the given service
fn find_services_with_duplicate_selectors(
    service: &Service,
    current_services: Vec<Service>,
) -> Vec<String> {
    current_services
        .iter()
        .filter_map(|svc| {
            if services_have_same_selectors(svc, service) {
                svc.metadata.name.clone()
            } else {
                None
            }
        })
        .collect()
}

fn validate_service(service: &Service) -> CallResult {
    match list_services(service.metadata.namespace.clone().unwrap_or_default()) {
        Ok(current_services) => {
            let duplicate_services =
                find_services_with_duplicate_selectors(service, current_services);
            if !duplicate_services.is_empty() {
                return kubewarden::reject_request(
                    Some(format!(
                        "service is using selector(s) already defined by these services: {duplicate_services:?}"
                    )),
                    None,
                    None,
                    None,
                );
            }
            kubewarden::accept_request()
        }
        Err(error) => kubewarden::reject_request(Some(error), None, None, None),
    }
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;

    match serde_json::from_value::<apicore::Service>(validation_request.request.object) {
        Ok(service) => validate_service(&service),
        Err(_) => {
            // we didn't get a Service object as expected, let's just accept this request
            kubewarden::accept_request()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use k8s_openapi::api::core::v1::Service;
    use k8s_openapi::api::core::v1::ServiceSpec;
    use k8s_openapi::apimachinery;
    use rstest::*;

    fn build_service_list(
        services_selectors_values: Vec<Vec<(String, String)>>,
        allow_empty_selectors: bool,
    ) -> Vec<Service> {
        let mut services = Vec::new();
        for selectors_values in services_selectors_values.iter() {
            services.push(build_service(
                selectors_values.to_vec(),
                allow_empty_selectors,
            ))
        }
        services.push(Service {
            metadata: apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("myservice-with-empty-selectors".to_owned()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: Some(BTreeMap::new()),
                ..Default::default()
            }),
            ..Default::default()
        });
        services
    }

    fn build_service(
        selectors_values: Vec<(String, String)>,
        allow_empty_selectors: bool,
    ) -> Service {
        let mut selectors_map = BTreeMap::new();
        let mut selectors = None;
        if !selectors_values.is_empty() || allow_empty_selectors {
            for (key, value) in selectors_values.iter() {
                selectors_map.insert(key.to_owned(), value.to_owned());
            }
            selectors = Some(selectors_map)
        }
        Service {
            metadata: apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("myservice".to_owned()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: selectors,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[rstest]
    #[case(build_service(vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())], false), build_service_list(vec![vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())]], false), vec!["myservice".to_owned()])]
    #[case(build_service(vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())], false), build_service_list(vec![vec![("app.kubernetes.io/other".to_owned(),"myapp".to_owned())]], false), vec![])]
    #[case(build_service(vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())], false), build_service_list(vec![vec![("app.kubernetes.io/name".to_owned(),"other".to_owned())]], false), vec![])]
    #[case(build_service(vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned()), ("otherlabel".to_owned(), "othervalue".to_owned())],false), build_service_list(vec![vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())]], false), vec![])]
    #[case(build_service(vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())], false), build_service_list(vec![vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned()), ("env".to_owned(),"production".to_owned())]], false), vec![])]
    #[case(build_service(vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())],false), build_service_list(vec![vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned()), ("env".to_owned(),"production".to_owned())],vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())]], false), vec!["myservice".to_owned()])]
    #[case(build_service(vec![], false), build_service_list(vec![Vec::<(String,String)>::new()], false), vec![])]
    #[case(build_service(vec![],false), build_service_list(vec![vec![("app.kubernetes.io/other".to_owned(),"myapp".to_owned())]], false), vec![])]
    #[case(build_service(vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())],false), build_service_list(vec![Vec::<(String,String)>::new()], false), vec![])]
    #[case(build_service(vec![], true), build_service_list(vec![Vec::<(String,String)>::new()], true), vec![])]
    #[case(build_service(vec![],true), build_service_list(vec![vec![("app.kubernetes.io/other".to_owned(),"myapp".to_owned())]], false), vec![])]
    #[case(build_service(vec![("app.kubernetes.io/name".to_owned(),"myapp".to_owned())],false), build_service_list(vec![Vec::<(String,String)>::new()], true), vec![])]
    fn find_duplicate_service_selectors(
        #[case] service: Service,
        #[case] current_services: Vec<Service>,
        #[case] duplicate_service_names: Vec<String>,
    ) {
        let found_duplicate_services =
            find_services_with_duplicate_selectors(&service, current_services);
        assert_eq!(found_duplicate_services, duplicate_service_names)
    }
}
