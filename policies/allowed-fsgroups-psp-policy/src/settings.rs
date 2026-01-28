use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{fmt, fmt::Display};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub(crate) struct Range {
    pub min: i64,
    pub max: i64,
}

impl Range {
    fn check(&self) -> Result<()> {
        if self.min > self.max {
            return Err(anyhow!("min on range cannot be greater than max",));
        };
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "rule")]
pub(crate) enum Rule {
    MustRunAs(Ranges),
    MayRunAs(Ranges),
    RunAsAny,
}

impl Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rule = match *self {
            Rule::MustRunAs(_) => "MustRunAs",
            Rule::MayRunAs(_) => "MayRunAs",
            Rule::RunAsAny => "RunAsAny",
        };
        write!(f, "{rule}")
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct Ranges {
    pub ranges: Vec<Range>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Settings {
    #[serde(flatten)]
    pub rule: Rule,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            rule: Rule::RunAsAny,
        }
    }
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        match &self.rule {
            Rule::MustRunAs(ranges) | Rule::MayRunAs(ranges) => {
                if ranges.ranges.is_empty() {
                    return Err(format!("{} must contain at least one range", self.rule));
                }
                if !ranges.ranges.iter().all(|range| range.check().is_ok()) {
                    return Err("all ranges must be valid".to_string());
                }
                Ok(())
            }
            Rule::RunAsAny => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::settings::Validatable;

    #[test]
    fn valid_settings() -> Result<()> {
        assert_eq!(
            Settings {
                rule: Rule::MayRunAs(Ranges { ranges: Vec::new() })
            }
            .validate(),
            Err("MayRunAs must contain at least one range".to_string())
        );

        assert_eq!(
            Settings {
                rule: Rule::MayRunAs(Ranges {
                    ranges: vec![Range {
                        min: 1000,
                        max: 1000
                    }],
                })
            }
            .validate(),
            Ok(())
        );

        assert_eq!(
            Settings {
                rule: Rule::MayRunAs(Ranges {
                    ranges: vec![Range {
                        min: 1000,
                        max: 500
                    }],
                })
            }
            .validate(),
            Err("all ranges must be valid".to_string())
        );

        assert_eq!(
            Settings {
                rule: Rule::MustRunAs(Ranges { ranges: Vec::new() })
            }
            .validate(),
            Err("MustRunAs must contain at least one range".to_string())
        );

        assert_eq!(
            Settings {
                rule: Rule::MustRunAs(Ranges {
                    ranges: vec![Range {
                        min: 1000,
                        max: 1000
                    }],
                })
            }
            .validate(),
            Ok(())
        );

        assert_eq!(
            Settings {
                rule: Rule::MustRunAs(Ranges {
                    ranges: vec![Range {
                        min: 1000,
                        max: 500
                    }],
                })
            }
            .validate(),
            Err("all ranges must be valid".to_string())
        );

        Ok(())
    }
}
