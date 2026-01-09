use const_format::formatcp;

#[cfg(not(any(feature = "env_var", feature = "labels", feature = "annotations")))]
compile_error!("At least one of the features: env_var, labels, or annotations must be enabled.");

#[cfg(any(
    all(feature = "env_var", feature = "labels"),
    all(feature = "env_var", feature = "annotations"),
    all(feature = "labels", feature = "annotations")
))]
compile_error!(
    "Only one of the features: env_var, labels, or annotations can be enabled at a time."
);

#[cfg(feature = "env_var")]
pub const RESOURCE_STR: &str = "environment variable";
#[cfg(feature = "labels")]
pub const RESOURCE_STR: &str = "label";
#[cfg(feature = "annotations")]
pub const RESOURCE_STR: &str = "annotation";

pub const CONTAINS_ANY_OF_ERROR_MSG: &str = formatcp!(
    "Resource must have at least one of the required {RESOURCE_STR}s specified by the validation rule. None of the expected {RESOURCE_STR}s were found:"
);
pub(crate) const DOES_NOT_CONTAIN_ANY_OF_ERROR_MSG: &str = formatcp!(
    "Resource must not have any of the {RESOURCE_STR}s specified in the validation rule. The following invalid {RESOURCE_STR}s were found:"
);
pub(crate) const CONTAINS_ALL_OF_ERROR_MSG: &str = formatcp!(
    "Resource is missing required {RESOURCE_STR}s as specified in the validation rules. The following {RESOURCE_STR}s are missing:",
);
pub(crate) const DOES_NOT_CONTAIN_ALL_OF_ERROR_MSG: &str = formatcp!(
    "Resource has conflicting {RESOURCE_STR}s set according to the validation rules. The following {RESOURCE_STR}s should not be set together:",
);
pub(crate) const CONTAINS_OTHER_THAN_ERROR_MSG: &str = formatcp!(
    "Resource must not have any {RESOURCE_STR}s other than those specified in the validation rule. The following {RESOURCE_STR}s were found that should not be present:"
);
pub(crate) const DOES_NOT_CONTAIN_OTHER_THAN_ERROR_MSG: &str = formatcp!(
    "Resource must have only {RESOURCE_STR}s from the validation rule. The following {RESOURCE_STR}s were found that should not be present:"
);
