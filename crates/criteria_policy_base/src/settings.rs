use std::collections::HashSet;

use crate::kubewarden_policy_sdk as kubewarden;

use serde::{Deserialize, Serialize};

/// Represents the base settings for the policy, which can be used to
/// define criteria for matching environment variables, labels, or annotations.
///
/// The real policy has just to embed this enum in its settings struct.
///
/// This enum implements the Validatable trait, which means it can be used
/// to validate the settings provided by the user.
/// The real policy must call `validate()` from within its own Settings::validate() method.
/// This enum makes sure the user provided some values for the policy to match against.
///
/// Limitation: the real policy must not require other types of configuration.
/// We could find workaround for this limitation, like trying to use `serde(flatten)`,
/// but we don't know if that would actually work. Luckily, the real policy
/// does not require any other configuration, so we can use this enum directly.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "criteria")]
#[allow(clippy::enum_variant_names)]
pub enum BaseSettings {
    /// Enforces that the resource has at least one of the `values` present.
    ContainsAnyOf { values: HashSet<String> },
    /// Enforces that the resource does not have any of the `values` present (denylist).
    DoesNotContainAnyOf { values: HashSet<String> },
    /// Enforces that all of the `values` are present in the resource.
    ContainsAllOf { values: HashSet<String> },
    /// Enforces that not all of the `values` are present together in the resource.
    DoesNotContainAllOf { values: HashSet<String> },
    /// Enforces that the resource contains at least one environment variable not in `values`.
    ContainsOtherThan { values: HashSet<String> },
    /// Enforces that the resource contains only environment variables from `values` (allowlist).
    DoesNotContainOtherThan { values: HashSet<String> },
}

// It's not possible to use the Default in the derive macro because we cannot
// set a #[default] attribute to enum item that is no unit enums.
impl Default for BaseSettings {
    fn default() -> Self {
        BaseSettings::ContainsAnyOf {
            values: HashSet::new(),
        }
    }
}

impl BaseSettings {
    /// Returns the set of values that the policy will use to match against
    pub fn values(&self) -> &HashSet<String> {
        match self {
            BaseSettings::ContainsAllOf { values } => values,
            BaseSettings::DoesNotContainAllOf { values } => values,
            BaseSettings::ContainsAnyOf { values } => values,
            BaseSettings::DoesNotContainAnyOf { values } => values,
            BaseSettings::ContainsOtherThan { values } => values,
            BaseSettings::DoesNotContainOtherThan { values } => values,
        }
    }
}

impl kubewarden::settings::Validatable for BaseSettings {
    fn validate(&self) -> Result<(), String> {
        let values = self.values();
        if values.is_empty() {
            return Err(format!(
                "Empty {} list is not allowed",
                crate::constants::RESOURCE_STR
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::settings::Validatable;
    use rstest::rstest;

    #[rstest]
    #[case::contains_all_of(BaseSettings::ContainsAllOf { values: HashSet::new() })]
    #[case::does_not_contain_all_of(BaseSettings::DoesNotContainAllOf{ values: HashSet::new() })]
    #[case::contains_any_of(BaseSettings::ContainsAnyOf { values: HashSet::new() })]
    #[case::does_not_contain_any_of(BaseSettings::DoesNotContainAnyOf { values: HashSet::new() })]
    #[case::contains_other_than(BaseSettings::ContainsOtherThan { values: HashSet::new() })]
    #[case::does_not_contain_other_than(BaseSettings::DoesNotContainOtherThan { values: HashSet::new() })]
    fn empty_settings_not_allowed(#[case] settings: BaseSettings) {
        assert!(settings.validate().is_err());
    }
}
