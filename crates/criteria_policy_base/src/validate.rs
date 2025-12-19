use std::collections::HashSet;

use anyhow::Result;

use crate::{
    operators::{
        contains_all_of, contains_any_of, contains_other_than, does_not_contain_all_of,
        does_not_contain_any_of, does_not_contain_other_than,
    },
    settings::BaseSettings,
};

/// Validates the input values against the specified settings.
pub fn validate_values(settings: &BaseSettings, input_values: &[String]) -> Result<()> {
    let input_values: HashSet<String> = input_values.iter().cloned().collect();
    match settings {
        BaseSettings::ContainsAllOf { values } => contains_all_of(values, &input_values),
        BaseSettings::DoesNotContainAllOf { values } => {
            does_not_contain_all_of(values, &input_values)
        }
        BaseSettings::ContainsAnyOf { values } => contains_any_of(values, &input_values),
        BaseSettings::DoesNotContainAnyOf { values } => {
            does_not_contain_any_of(values, &input_values)
        }
        BaseSettings::ContainsOtherThan { values } => contains_other_than(values, &input_values),
        BaseSettings::DoesNotContainOtherThan { values } => {
            does_not_contain_other_than(values, &input_values)
        }
    }
}
