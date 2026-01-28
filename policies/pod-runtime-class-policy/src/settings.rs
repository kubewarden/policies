use std::collections::HashSet;

use serde::{Deserialize, Serialize};

// Describe the settings your policy expects when
// loaded by the policy server.
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    pub reserved_runtimes: HashSet<String>,
    pub fallback_runtime: Option<String>,
    pub default_runtime_reserved: Option<bool>,
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        if let Some(fallback_runtime) = &self.fallback_runtime
            && self.reserved_runtimes.contains(fallback_runtime)
        {
            return Err(format!(
                "fallback runtime {fallback_runtime} cannot be part of the reserved runtimes"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubewarden::settings::Validatable;
    use rstest::rstest;

    #[rstest]
    #[case("runc", "runc", false)]
    #[case("runc", "gvisor", true)]
    fn validation_test(
        #[case] reserved_runtime: String,
        #[case] fallback_runtime: String,
        #[case] is_ok: bool,
    ) {
        let settings = Settings {
            reserved_runtimes: HashSet::from([reserved_runtime]),
            fallback_runtime: Some(fallback_runtime),
            ..Default::default()
        };
        let validation_result = settings.validate();
        assert_eq!(validation_result.is_ok(), is_ok)
    }
}
