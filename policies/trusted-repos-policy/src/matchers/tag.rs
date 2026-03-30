use std::{fmt, hash::Hash};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::matchers::string::StringMatcher;

#[derive(Debug, Clone)]
pub struct TagMatcher(pub(crate) StringMatcher);

impl TagMatcher {
    pub(crate) fn raw(&self) -> &str {
        self.0.raw()
    }

    pub(crate) fn matches(&self, s: &str) -> bool {
        self.0.matches(s)
    }
}

impl PartialEq for TagMatcher {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for TagMatcher {}

impl Hash for TagMatcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'de> Deserialize<'de> for TagMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(TagMatcher(StringMatcher::deserialize(deserializer)?))
    }
}

impl Serialize for TagMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl fmt::Display for TagMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::*;
    use wildmatch::WildMatch;

    use crate::settings::Tags;

    fn exact_tag(s: &str) -> TagMatcher {
        TagMatcher(StringMatcher::Exact(s.to_string()))
    }

    fn pattern_tag(s: &str) -> TagMatcher {
        TagMatcher(StringMatcher::Pattern {
            pattern: WildMatch::new(s),
            raw: s.to_string(),
        })
    }

    #[rstest]
    #[case::empty_settings(Vec::new(), true)]
    #[case::valid_tags(vec![exact_tag("latest")], true)]
    #[case::invalid_tags(vec![exact_tag("latest"), exact_tag("1.0.0+rc3")], false)]
    #[case::valid_pattern_tag(vec![pattern_tag("*-rc*")], true)]
    fn validate_tags(#[case] tags: Vec<TagMatcher>, #[case] is_valid: bool) {
        let tags = Tags {
            reject: tags.into_iter().collect(),
        };

        let result = tags.validate();
        if is_valid {
            assert!(result.is_ok(), "{result:?}");
        } else {
            assert!(result.is_err(), "was supposed to be invalid");
        }
    }

    #[test]
    fn deserialize_tag_pattern() {
        let json = r#"{"reject": ["*-rc*"]}"#;
        let tags: Tags = serde_json::from_str(json).unwrap();
        assert_eq!(tags.reject.len(), 1);
        let matcher = tags.reject.iter().next().unwrap();
        assert!(matches!(matcher.0, StringMatcher::Pattern { .. }));
    }
}
