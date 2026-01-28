use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Error message constants
const ERR_ONE_OF_DENIED_OR_ALLOWED_MUST_BE_SET: &str =
    "One of deniedStorageClasses or allowedStorageClasses must be set";
const ERR_ONLY_ONE_OF_DENIED_OR_ALLOWED_CAN_BE_SET: &str =
    "Only one of deniedStorageClasses or allowedStorageClasses can be set";
const ERR_DENIED_CANNOT_BE_EMPTY: &str = "deniedStorageClasses cannot be empty";
const ERR_FALLBACK_IN_DENIED: &str = "fallbackStorageClass cannot be in deniedStorageClasses";
const ERR_ALLOWED_CANNOT_BE_EMPTY: &str = "allowedStorageClasses cannot be empty";
const ERR_FALLBACK_NOT_IN_ALLOWED: &str = "fallbackStorageClass must be in allowedStorageClasses";

// Describe the settings your policy expects when
// loaded by the policy server.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    pub denied_storage_classes: Option<HashSet<String>>,
    pub fallback_storage_class: Option<String>,
    pub allowed_storage_classes: Option<HashSet<String>>,
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        if self.denied_storage_classes.is_none() && self.allowed_storage_classes.is_none() {
            return Err(ERR_ONE_OF_DENIED_OR_ALLOWED_MUST_BE_SET.to_string());
        }

        if self.denied_storage_classes.is_some() && self.allowed_storage_classes.is_some() {
            return Err(ERR_ONLY_ONE_OF_DENIED_OR_ALLOWED_CAN_BE_SET.to_string());
        }

        if let Some(denied_storage_classes) = &self.denied_storage_classes {
            if denied_storage_classes.is_empty() {
                return Err(ERR_DENIED_CANNOT_BE_EMPTY.to_string());
            }

            if let Some(fallback_storage_class) = &self.fallback_storage_class
                && denied_storage_classes.contains(fallback_storage_class)
            {
                return Err(ERR_FALLBACK_IN_DENIED.to_string());
            }
        }

        if let Some(allowed_storage_classes) = &self.allowed_storage_classes {
            if allowed_storage_classes.is_empty() {
                return Err(ERR_ALLOWED_CANNOT_BE_EMPTY.to_string());
            }

            if let Some(fallback_storage_class) = &self.fallback_storage_class
                && !allowed_storage_classes.contains(fallback_storage_class)
            {
                return Err(ERR_FALLBACK_NOT_IN_ALLOWED.to_string());
            }
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
    #[case::empty_settings_is_not_valid(
        None,
        None,
        None,
        Some(ERR_ONE_OF_DENIED_OR_ALLOWED_MUST_BE_SET)
    )]
    #[case::using_deny_and_allow_lists_at_the_same_time_is_not_valid(
        Some(HashSet::from(["foo", "bar"])),
        Some(HashSet::from(["foo", "bar"])),
        None,
        Some(ERR_ONLY_ONE_OF_DENIED_OR_ALLOWED_CAN_BE_SET)
    )]
    #[case::empty_denied_list_is_not_valid(
        None,
        Some(HashSet::new()),
        None,
        Some(ERR_DENIED_CANNOT_BE_EMPTY)
    )]
    #[case::empty_allowed_list_is_not_valid(
        Some(HashSet::new()),
        None,
        None,
        Some(ERR_ALLOWED_CANNOT_BE_EMPTY)
    )]
    #[case::denied_list_with_some_item_is_valid(
        None,
        Some(HashSet::from(["foo", "bar"])), 
        None,
        None)]
    #[case::allowed_list_with_some_item_is_valid(
        Some(HashSet::from(["foo", "bar"])),
        None,
        None,
        None,
    )]
    #[case::fallback_in_denied_list_is_not_valid(
        None,
        Some(HashSet::from(["foo", "bar"])), 
        Some("bar"), 
        Some(ERR_FALLBACK_IN_DENIED))]
    #[case::fallback_not_in_denied_list_is_valid(
        None,
        Some(HashSet::from(["foo", "bar"])), 
        Some("baz"), 
        None)]
    #[case::fallback_class_not_in_allowed_list_is_not_valid(
        Some(HashSet::from(["foo", "bar"])),
        None,
        Some("baz"), 
        Some(ERR_FALLBACK_NOT_IN_ALLOWED))]
    #[case::fallback_class_in_allowed_list_is_valid(
        Some(HashSet::from(["foo", "bar"])),
        None,
        Some("foo"), 
        None)]
    fn settings_validation(
        #[case] allowed_storage_classes: Option<HashSet<&str>>,
        #[case] denied_storage_classes: Option<HashSet<&str>>,
        #[case] fallback_storage_class: Option<&str>,
        #[case] expected_error: Option<&str>,
    ) {
        let settings = Settings {
            denied_storage_classes: denied_storage_classes
                .map(|set| set.into_iter().map(String::from).collect()),
            allowed_storage_classes: allowed_storage_classes
                .map(|set| set.into_iter().map(String::from).collect()),
            fallback_storage_class: fallback_storage_class.map(String::from),
        };

        let validation_result = settings.validate();

        if let Some(expected_error) = expected_error {
            assert_eq!(
                validation_result.expect_err("Missing validation error"),
                expected_error
            );
        } else {
            assert!(validation_result.is_ok());
        }
    }
}
