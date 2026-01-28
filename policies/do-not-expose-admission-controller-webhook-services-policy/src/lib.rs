use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::Resource;
use k8s_openapi::api::admissionregistration::v1::{
    MutatingWebhookConfiguration, ValidatingWebhookConfiguration,
};

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::Settings;

mod service_details;

mod service_finder;
use service_finder::ServiceFinder;

mod check;
use check::find_webhook_services_exposed;

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;

    let services = match validation_request.request.kind.kind.as_str() {
        ValidatingWebhookConfiguration::KIND => {
            let cfg: ValidatingWebhookConfiguration =
                serde_json::from_value(validation_request.request.object)?;
            cfg.get_services()
        }
        MutatingWebhookConfiguration::KIND => {
            let cfg: MutatingWebhookConfiguration =
                serde_json::from_value(validation_request.request.object)?;
            cfg.get_services()
        }
        _ => return kubewarden::accept_request(),
    };

    let exposed_services = find_webhook_services_exposed(&services)?;

    if exposed_services.is_empty() {
        // no services exposed by Ingress, NodePort, nor LoadBalancer
        return kubewarden::accept_request();
    }

    let msg = format!(
        "Webhook service(s) exposed by Ingress, NodePort, or LoadBalancer: {}",
        exposed_services
            .iter()
            .map(|svc| format!("{}/{}", svc.namespace, svc.name))
            .collect::<Vec<_>>()
            .join(", ")
    );

    kubewarden::reject_request(Some(msg), None, None, None)
}
