use lazy_static::lazy_static;
use regex::RegexSet;

use serde::{Deserialize, Serialize};

lazy_static! {
    // Regex to allow the "latest" version and versions like v<major>.<minor>.
    // However, minor version cannot start with 0.
    static ref VERSION_REGEXES: RegexSet = RegexSet::new([r"^latest$", r"^v\d+\.(0|[123456789]\d*)$"]).unwrap();
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Level {
    Privileged,
    Baseline,
    Restricted,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Level::Privileged => {
                write!(f, "privileged")
            }
            Level::Baseline => {
                write!(f, "baseline")
            }
            Level::Restricted => {
                write!(f, "restricted")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) struct Modes {
    pub enforce: Option<Level>,
    pub enforce_version: Option<String>,
    pub audit: Option<Level>,
    pub audit_version: Option<String>,
    pub warn: Option<Level>,
    pub warn_version: Option<String>,
}

// Describe the settings your policy expects when
// loaded by the policy server.
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub(crate) struct Settings {
    pub modes: Modes,
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        if self.modes.enforce.is_none() && self.modes.audit.is_none() && self.modes.warn.is_none() {
            return Err(
                "At least one of the 'enforce', 'audit' or 'warn' configuration must be defined"
                    .to_string(),
            );
        }

        if let Some(version) = &self.modes.enforce_version {
            let matches: Vec<_> = VERSION_REGEXES.matches(version).into_iter().collect();
            if matches.is_empty() {
                return Err(format!(
                    "Version {version} is invalid. It must follow the v<major>.<minor> pattern or be 'latest' value"
                ));
            }
            if self.modes.enforce.is_none() {
                return Err("cannot define enforce version with no enforce mode.".to_string());
            }
        }
        if let Some(version) = &self.modes.audit_version {
            let matches: Vec<_> = VERSION_REGEXES.matches(version).into_iter().collect();
            if matches.is_empty() {
                return Err(format!(
                    "Version {version} is invalid. It must follow the v<major>.<minor> pattern or be 'latest' value"
                ));
            }
            if self.modes.audit.is_none() {
                return Err("cannot define audit version with no audit mode.".to_string());
            }
        }
        if let Some(version) = &self.modes.warn_version {
            let matches: Vec<_> = VERSION_REGEXES.matches(version).into_iter().collect();
            if matches.is_empty() {
                return Err(format!(
                    "Version {version} is invalid. It must follow the v<major>.<minor> pattern or be 'latest' value"
                ));
            }
            if self.modes.warn.is_none() {
                return Err("cannot define warn version with no warn mode.".to_string());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    use kubewarden_policy_sdk::settings::Validatable;

    #[test]
    fn version_with_no_mode_should_fail_test() {
        let mut settings = Settings {
            modes: Modes {
                enforce: None,
                enforce_version: Some("1.25.0".to_string()),
                audit: None,
                audit_version: None,
                warn: None,
                warn_version: None,
            },
        };
        assert!(settings.validate().is_err());
        settings = Settings {
            modes: Modes {
                enforce: None,
                enforce_version: Some("1.25.0".to_string()),
                audit: None,
                audit_version: None,
                warn: None,
                warn_version: None,
            },
        };
        assert!(settings.validate().is_err());

        settings = Settings {
            modes: Modes {
                enforce: None,
                enforce_version: None,
                audit: None,
                audit_version: Some("1.25.0".to_string()),
                warn: None,
                warn_version: None,
            },
        };
        assert!(settings.validate().is_err());

        settings = Settings {
            modes: Modes {
                enforce: None,
                enforce_version: None,
                audit: None,
                audit_version: None,
                warn: None,
                warn_version: Some("1.25.0".to_string()),
            },
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn user_must_define_at_least_one_mode_test() {
        let settings = Settings {
            modes: Modes {
                enforce: None,
                enforce_version: None,
                audit: None,
                audit_version: None,
                warn: None,
                warn_version: None,
            },
        };
        assert!(settings.validate().is_err());
    }

    #[rstest]
    #[case("v1.25", true)]
    #[case("v1.26", true)]
    #[case("v1.0", true)]
    #[case("latest", true)]
    #[case("v1", false)]
    #[case("v1.027", false)]
    #[case("1.0", false)]
    #[case("1.00", false)]
    #[case("foo", false)]
    #[case("1234", false)]
    fn versions_test(#[case] version: String, #[case] is_valid: bool) {
        let mut settings = Settings {
            modes: Modes {
                enforce: Some(Level::Baseline),
                enforce_version: Some(version.clone()),
                audit: Some(Level::Baseline),
                audit_version: Some(version.clone()),
                warn: Some(Level::Baseline),
                warn_version: Some(version.clone()),
            },
        };
        assert_eq!(settings.validate().is_ok(), is_valid);

        settings = Settings {
            modes: Modes {
                enforce: Some(Level::Baseline),
                enforce_version: Some(version.clone()),
                audit: None,
                audit_version: None,
                warn: None,
                warn_version: None,
            },
        };
        assert_eq!(settings.validate().is_ok(), is_valid);

        settings = Settings {
            modes: Modes {
                enforce: None,
                enforce_version: None,
                audit: Some(Level::Baseline),
                audit_version: Some(version.clone()),
                warn: None,
                warn_version: None,
            },
        };
        assert_eq!(settings.validate().is_ok(), is_valid);

        settings = Settings {
            modes: Modes {
                enforce: None,
                enforce_version: None,
                audit: None,
                audit_version: None,
                warn: Some(Level::Baseline),
                warn_version: Some(version.clone()),
            },
        };
        assert_eq!(settings.validate().is_ok(), is_valid);
    }
}
