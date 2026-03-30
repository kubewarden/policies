use std::{fmt, hash::Hash};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::matchers::string::StringMatcher;

#[derive(Debug, Clone)]
pub struct RegistryMatcher(pub(crate) StringMatcher);

impl RegistryMatcher {
    pub(crate) fn matches(&self, s: &str) -> bool {
        self.0.matches(s)
    }
}

impl PartialEq for RegistryMatcher {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for RegistryMatcher {}

impl Hash for RegistryMatcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'de> Deserialize<'de> for RegistryMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(RegistryMatcher(StringMatcher::deserialize(deserializer)?))
    }
}

impl Serialize for RegistryMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl fmt::Display for RegistryMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::*;
    use wildmatch::WildMatch;

    use crate::settings::Registries;

    fn exact_registry(s: &str) -> RegistryMatcher {
        RegistryMatcher(StringMatcher::Exact(s.to_string()))
    }

    fn pattern_registry(s: &str) -> RegistryMatcher {
        RegistryMatcher(StringMatcher::Pattern {
            pattern: WildMatch::new(s),
            raw: s.to_string(),
        })
    }

    #[rstest]
    #[case::empty_settings(Vec::new(), Vec::new(), true)]
    #[case::allow_only(vec![exact_registry("allowed-registry.com")], Vec::new(), true)]
    #[case::reject_only(Vec::new(), vec![exact_registry("forbidden-registry.com")], true)]
    #[case::allow_and_reject(
        vec![exact_registry("allowed-registry.com")],
        vec![exact_registry("forbidden-registry.com")],
        false
    )]
    #[case::allow_pattern(vec![pattern_registry("*.example.com")], Vec::new(), true)]
    fn validate_registries(
        #[case] allow: Vec<RegistryMatcher>,
        #[case] reject: Vec<RegistryMatcher>,
        #[case] is_valid: bool,
    ) {
        let registries = Registries {
            allow: allow.into_iter().collect(),
            reject: reject.into_iter().collect(),
        };

        let result = registries.validate();
        if is_valid {
            assert!(result.is_ok(), "{result:?}");
        } else {
            assert!(result.is_err(), "was supposed to be invalid");
        }
    }

    #[test]
    fn deserialize_registry_pattern() {
        let json = r#"{"allow": ["*.my-corp.com"], "reject": []}"#;
        let registries: Registries = serde_json::from_str(json).unwrap();
        assert_eq!(registries.allow.len(), 1);
        let matcher = registries.allow.iter().next().unwrap();
        assert!(matches!(matcher.0, StringMatcher::Pattern { .. }));
    }
}
