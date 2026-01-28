use anyhow::{Result, anyhow};
use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;
use lazy_static::lazy_static;
use slog::{Logger, o, warn};
use std::{thread, time};

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{logging, protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::Settings;

lazy_static! {
    static ref LOG_DRAIN: Logger = Logger::root(
        logging::KubewardenDrain::new(),
        o!("policy" => "sleeping-policy")
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn extract_sleep_duration_from_annotation(payload: &[u8]) -> Result<Option<u64>> {
    let annotations_expr = jmespath::compile("request.object.metadata.annotations")?;

    let json = std::str::from_utf8(payload)?;
    let data = jmespath::Variable::from_json(json)
        .map_err(|e| anyhow!("cannot create jmespath variable: {}", e))?;

    let result = annotations_expr.search(data)?;
    if result.is_object() {
        let annotations = result.as_object().unwrap();
        match annotations.get("kubewarden.sleep_duration_milliseconds") {
            None => Ok(None),
            Some(var) => {
                if let Some(s) = var.as_string() {
                    Ok(Some(s.parse::<u64>()?))
                } else if let Some(n) = var.as_number() {
                    Ok(Some(n as u64))
                } else {
                    Ok(None)
                }
            }
        }
    } else {
        Ok(None)
    }
}

const MICRO_NAP_MILLIS: u64 = 100;

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;

    let sleep = extract_sleep_duration_from_annotation(payload)?
        .unwrap_or(validation_request.settings.sleep_milliseconds);

    let micro_naps_total = sleep / MICRO_NAP_MILLIS;
    let sleep_duration = time::Duration::from_millis(MICRO_NAP_MILLIS);

    warn!(LOG_DRAIN, "taking a nap"; "duration_milliseconds" => sleep);

    // Epoch interruption works by having wasmtime introduce some "check instructions"
    // in between the wasm module instructions.
    // Doing one single thread::sleep(SLEEP) invocation introduces only to
    // "check instructions": before the sleep and after. Because of that, the sleep
    // instruction will NOT be interrupted. This is a corner case that has to be
    // addressed in other ways by the host.
    //
    // To make this test more meaningful, we have to break the sleep into smaller naps.
    // That's closer to what a program does.
    for _iter in 0..micro_naps_total {
        thread::sleep(sleep_duration);
    }

    kubewarden::accept_request()
}
