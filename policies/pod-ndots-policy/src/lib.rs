use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::Resource;
use k8s_openapi::api::core::v1::{self as apicore, PodSpec};

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

    if validation_request.request.kind.kind != apicore::Pod::KIND {
        return kubewarden::accept_request();
    }
    let pod = serde_json::from_value::<apicore::Pod>(validation_request.request.object)?;

    let podspec = pod.spec.clone().unwrap_or_default();
    let podspec_patched = enforce_ndots(&validation_request.settings, &podspec);
    if podspec_patched != podspec {
        let patched_pod = apicore::Pod {
            spec: Some(podspec_patched),
            ..pod
        };
        return kubewarden::mutate_request(serde_json::to_value(&patched_pod)?);
    }

    kubewarden::accept_request()
}

fn enforce_ndots(settings: &Settings, podspec: &apicore::PodSpec) -> PodSpec {
    // preserve the order of the options to prevent needless updates
    let mut dns_options: Vec<apicore::PodDNSConfigOption> = podspec
        .dns_config
        .as_ref()
        .and_then(|dns_config| dns_config.options.clone())
        .unwrap_or_default()
        .iter()
        .map(|option| {
            if option.name == Some("ndots".to_string()) {
                apicore::PodDNSConfigOption {
                    name: Some("ndots".to_string()),
                    value: Some(settings.ndots.to_string()),
                }
            } else {
                option.clone()
            }
        })
        .collect();

    // ensure the option is added if it's not present
    if dns_options
        .iter()
        .all(|option| option.name != Some("ndots".to_string()))
    {
        dns_options.push(apicore::PodDNSConfigOption {
            name: Some("ndots".to_string()),
            value: Some(settings.ndots.to_string()),
        });
    }

    PodSpec {
        dns_config: Some(apicore::PodDNSConfig {
            nameservers: podspec
                .dns_config
                .as_ref()
                .and_then(|dns_config| dns_config.nameservers.clone()),
            searches: podspec
                .dns_config
                .as_ref()
                .and_then(|dns_config| dns_config.searches.clone()),
            options: Some(dns_options),
        }),
        ..podspec.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::test::Testcase;
    use rstest::*;

    fn build_pod_dns_config(ndots: Option<usize>) -> apicore::PodDNSConfig {
        let mut options = vec![apicore::PodDNSConfigOption {
            name: Some("timeout".to_string()),
            value: Some("5".to_string()),
        }];

        if let Some(ndots) = ndots {
            options.push(apicore::PodDNSConfigOption {
                name: Some("ndots".to_string()),
                value: Some(ndots.to_string()),
            });
        }

        apicore::PodDNSConfig {
            nameservers: Some(vec!["1.1.1.1".to_string()]),
            searches: Some(vec!["example.com".to_string()]),
            options: Some(options),
        }
    }

    #[rstest]
    #[case::no_dns_config(None, apicore::PodDNSConfig {
        options: Some(vec![apicore::PodDNSConfigOption{
          name: Some("ndots".to_string()),
          value: Some("5".to_string()),
        }]),
        ..Default::default()
    })]
    #[case::no_dns_config_option_about_ndots(
        Some(build_pod_dns_config(None)),
        build_pod_dns_config(Some(5))
    )]
    #[case::change_dns_config_option_about_ndots(
        Some(build_pod_dns_config(Some(1))),
        build_pod_dns_config(Some(5))
    )]
    fn enforce_ndots_preserve_other_options(
        #[case] dns_config: Option<apicore::PodDNSConfig>,
        #[case] expected_dns_config: apicore::PodDNSConfig,
    ) {
        let settings = Settings { ndots: 5 };
        let podspec = PodSpec {
            dns_config,
            containers: vec![apicore::Container {
                name: "nginx".to_string(),
                image: Some("nginx".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let expected_podspec = PodSpec {
            dns_config: Some(expected_dns_config),
            ..podspec.clone()
        };

        let podspec_patched = enforce_ndots(&settings, &podspec);
        assert_eq!(
            podspec_patched, expected_podspec,
            "got: {:?} instead of {:?}",
            podspec_patched, expected_podspec
        );
    }

    #[rstest]
    // Note: this test cares only about covering the switch statement of the resournce kind
    #[case::change_pod("test_data/pod_without_ndots.json", true)]
    #[case::do_not_change_pod("test_data/pod_with_5_ndots.json", false)]
    fn test_validate(#[case] fixture: &str, #[case] expect_mutated_object: bool) {
        let settings = Settings { ndots: 5 };

        let test_case = Testcase {
            name: "test".to_string(),
            fixture_file: fixture.to_string(),
            expected_validation_result: true,
            settings: settings.clone(),
        };

        let validation_response = test_case.eval(validate).expect("validation failed");
        if expect_mutated_object {
            assert!(validation_response.mutated_object.is_some());
            let pod =
                serde_json::from_value::<apicore::Pod>(validation_response.mutated_object.unwrap())
                    .expect("failed to parse mutated object");
            let dns_config_options = pod.spec.unwrap().dns_config.unwrap().options.unwrap();
            assert_eq!(dns_config_options.len(), 1);
            let option = dns_config_options[0].clone();
            assert_eq!(option.name, Some("ndots".to_string()));
            assert_eq!(option.value, Some(settings.ndots.to_string()));
        } else {
            assert!(validation_response.mutated_object.is_none());
        }
    }
}
