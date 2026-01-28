use anyhow::{Result, anyhow};

use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::api::core::v1 as apicore;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::{Ranges, Rule, Settings};

#[unsafe(no_mangle)]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

#[derive(Debug, PartialEq)]
enum PolicyResponse {
    Accept,
    Reject(String),
    Mutate(serde_json::Value),
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;

    let pod = match serde_json::from_value::<apicore::Pod>(validation_request.request.object) {
        Ok(pod) => pod,
        Err(_) => return kubewarden::accept_request(),
    };

    let settings = validation_request.settings;

    match do_validate(pod, settings)? {
        PolicyResponse::Accept => kubewarden::accept_request(),
        PolicyResponse::Reject(message) => {
            kubewarden::reject_request(Some(message), None, None, None)
        }
        PolicyResponse::Mutate(mutated_object) => kubewarden::mutate_request(mutated_object),
    }
}

fn do_validate(pod: apicore::Pod, settings: settings::Settings) -> Result<PolicyResponse> {
    let pod_spec = pod.spec.ok_or_else(|| anyhow!("invalid pod spec"))?;

    match settings.rule {
        Rule::MustRunAs(ranges) => {
            let pod_with_defaulted_fs_group = apicore::Pod {
                spec: Some(apicore::PodSpec {
                    security_context: Some(apicore::PodSecurityContext {
                        fs_group: Some(
                            ranges.ranges.first().unwrap().min, // It is safe to unwrap here because the settings
                                                                // validation ensure that there is at least one range
                                                                // in the list
                        ),
                        ..apicore::PodSecurityContext::default()
                    }),
                    ..pod_spec
                }),
                ..pod
            };
            if let Some(security_context) = pod_spec.security_context {
                match security_context.fs_group {
                    Some(fs_group) => Ok(validate_fs_group(fs_group, ranges)),
                    None => Ok(PolicyResponse::Mutate(serde_json::to_value(
                        pod_with_defaulted_fs_group,
                    )?)),
                }
            } else {
                Ok(PolicyResponse::Mutate(serde_json::to_value(
                    pod_with_defaulted_fs_group,
                )?))
            }
        }
        Rule::MayRunAs(ranges) => {
            if let Some(security_context) = pod_spec.security_context {
                match security_context.fs_group {
                    Some(fs_group) => Ok(validate_fs_group(fs_group, ranges)),
                    None => Ok(PolicyResponse::Accept),
                }
            } else {
                Ok(PolicyResponse::Accept)
            }
        }
        Rule::RunAsAny => Ok(PolicyResponse::Accept),
    }
}

fn validate_fs_group(fs_group: i64, ranges: Ranges) -> PolicyResponse {
    if ranges
        .ranges
        .iter()
        .any(|range| fs_group >= range.min && fs_group <= range.max)
    {
        PolicyResponse::Accept
    } else {
        PolicyResponse::Reject(format!("fsGroup {fs_group} is not included in any range"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use settings::Range;

    #[test]
    fn run_as_any_always_accepts() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec::default()),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::RunAsAny
                }
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn may_run_as_accepts_with_empty_security_context() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec::default()),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MayRunAs(Ranges {
                        ranges: vec![Range {
                            min: 1000,
                            max: 2000,
                        }]
                    })
                }
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn may_run_as_accepts_with_empty_fsgroup() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext::default()),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MayRunAs(Ranges {
                        ranges: vec![Range {
                            min: 1000,
                            max: 2000,
                        }]
                    })
                }
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn may_run_as_accepts_with_fsgroup_in_range() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            fs_group: Some(1000),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MayRunAs(Ranges {
                        ranges: vec![Range {
                            min: 1000,
                            max: 2000,
                        }]
                    })
                }
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn may_run_as_accepts_with_fsgroup_in_some_range() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            fs_group: Some(1000),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MayRunAs(Ranges {
                        ranges: vec![
                            Range { min: 100, max: 200 },
                            Range {
                                min: 1000,
                                max: 2000,
                            }
                        ]
                    })
                }
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn may_run_as_rejects_with_fsgroup_in_no_range() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            fs_group: Some(100),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MayRunAs(Ranges {
                        ranges: vec![Range {
                            min: 1000,
                            max: 2000,
                        }]
                    })
                }
            )?,
            PolicyResponse::Reject("fsGroup 100 is not included in any range".to_string())
        );

        Ok(())
    }

    #[test]
    fn must_run_as_mutates_with_empty_security_context() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec::default()),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MustRunAs(Ranges {
                        ranges: vec![Range {
                            min: 1000,
                            max: 2000,
                        }]
                    })
                }
            )?,
            PolicyResponse::Mutate(serde_json::to_value(apicore::Pod {
                spec: Some(apicore::PodSpec {
                    security_context: Some(apicore::PodSecurityContext {
                        fs_group: Some(1000),
                        ..apicore::PodSecurityContext::default()
                    }),
                    ..apicore::PodSpec::default()
                }),
                ..apicore::Pod::default()
            })?)
        );

        Ok(())
    }

    #[test]
    fn must_run_as_mutates_with_empty_security_context_and_unordered_ranges() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec::default()),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MustRunAs(Ranges {
                        ranges: vec![
                            Range {
                                min: 3000,
                                max: 4000,
                            },
                            Range {
                                min: 1000,
                                max: 2000,
                            }
                        ]
                    })
                }
            )?,
            PolicyResponse::Mutate(serde_json::to_value(apicore::Pod {
                spec: Some(apicore::PodSpec {
                    security_context: Some(apicore::PodSecurityContext {
                        fs_group: Some(3000),
                        ..apicore::PodSecurityContext::default()
                    }),
                    ..apicore::PodSpec::default()
                }),
                ..apicore::Pod::default()
            })?)
        );

        Ok(())
    }

    #[test]
    fn must_run_as_mutates_with_empty_fsgroup() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext::default()),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MustRunAs(Ranges {
                        ranges: vec![Range {
                            min: 1000,
                            max: 2000,
                        }]
                    })
                }
            )?,
            PolicyResponse::Mutate(serde_json::to_value(apicore::Pod {
                spec: Some(apicore::PodSpec {
                    security_context: Some(apicore::PodSecurityContext {
                        fs_group: Some(1000),
                        ..apicore::PodSecurityContext::default()
                    }),
                    ..apicore::PodSpec::default()
                }),
                ..apicore::Pod::default()
            })?)
        );

        Ok(())
    }

    #[test]
    fn must_run_as_mutates_with_empty_fsgroup_and_unordered_ranges() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext::default()),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MustRunAs(Ranges {
                        ranges: vec![
                            Range {
                                min: 3000,
                                max: 4000,
                            },
                            Range {
                                min: 1000,
                                max: 2000,
                            }
                        ]
                    })
                }
            )?,
            PolicyResponse::Mutate(serde_json::to_value(apicore::Pod {
                spec: Some(apicore::PodSpec {
                    security_context: Some(apicore::PodSecurityContext {
                        fs_group: Some(3000),
                        ..apicore::PodSecurityContext::default()
                    }),
                    ..apicore::PodSpec::default()
                }),
                ..apicore::Pod::default()
            })?)
        );

        Ok(())
    }

    #[test]
    fn must_run_as_accepts_with_fsgroup_in_range() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            fs_group: Some(1000),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MustRunAs(Ranges {
                        ranges: vec![Range {
                            min: 1000,
                            max: 2000,
                        }]
                    })
                }
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn must_run_as_accepts_with_fsgroup_in_some_range() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            fs_group: Some(1000),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MustRunAs(Ranges {
                        ranges: vec![
                            Range { min: 100, max: 200 },
                            Range {
                                min: 1000,
                                max: 2000,
                            }
                        ]
                    })
                }
            )?,
            PolicyResponse::Accept
        );

        Ok(())
    }

    #[test]
    fn must_run_as_rejects_with_fsgroup_in_no_range() -> Result<()> {
        assert_eq!(
            do_validate(
                apicore::Pod {
                    spec: Some(apicore::PodSpec {
                        security_context: Some(apicore::PodSecurityContext {
                            fs_group: Some(100),
                            ..apicore::PodSecurityContext::default()
                        }),
                        ..apicore::PodSpec::default()
                    }),
                    ..apicore::Pod::default()
                },
                Settings {
                    rule: Rule::MustRunAs(Ranges {
                        ranges: vec![Range {
                            min: 1000,
                            max: 2000,
                        }]
                    })
                }
            )?,
            PolicyResponse::Reject("fsGroup 100 is not included in any range".to_string())
        );

        Ok(())
    }
}
