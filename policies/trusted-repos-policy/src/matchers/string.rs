use std::{fmt, hash::Hash};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wildmatch::WildMatch;

use crate::matchers::is_glob_pattern;

#[derive(Debug, Clone)]
pub(crate) enum StringMatcher {
    Exact(String),
    Pattern { pattern: WildMatch, raw: String },
}

impl StringMatcher {
    pub(crate) fn raw(&self) -> &str {
        match self {
            StringMatcher::Exact(s) => s,
            StringMatcher::Pattern { raw, .. } => raw,
        }
    }

    pub(crate) fn matches(&self, s: &str) -> bool {
        match self {
            StringMatcher::Exact(exact) => exact == s,
            StringMatcher::Pattern { pattern, .. } => pattern.matches(s),
        }
    }
}

impl PartialEq for StringMatcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StringMatcher::Exact(a), StringMatcher::Exact(b)) => a == b,
            (StringMatcher::Pattern { raw: a, .. }, StringMatcher::Pattern { raw: b, .. }) => {
                a == b
            }
            _ => false,
        }
    }
}

impl Eq for StringMatcher {}

impl Hash for StringMatcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw().hash(state);
    }
}

impl<'de> Deserialize<'de> for StringMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if is_glob_pattern(&s) {
            Ok(StringMatcher::Pattern {
                pattern: WildMatch::new(&s),
                raw: s,
            })
        } else {
            Ok(StringMatcher::Exact(s))
        }
    }
}

impl Serialize for StringMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.raw())
    }
}

impl fmt::Display for StringMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::*;
    use wildmatch::WildMatch;

    // --- matches ---

    #[rstest]
    #[case::exact_match("docker.io", "docker.io", true)]
    #[case::exact_no_match("docker.io", "ghcr.io", false)]
    #[case::pattern_match("*.my-corp.com", "registry.my-corp.com", true)]
    #[case::pattern_no_match("*.my-corp.com", "docker.io", false)]
    #[case::pattern_question_mark("v1.?.0", "v1.2.0", true)]
    #[case::pattern_wildcard("v1.*.0", "v1.20.0", true)]
    #[case::pattern_question_mark_no_match("v1.?.0", "v1.20.0", false)]
    fn matches(#[case] matcher_raw: &str, #[case] input: &str, #[case] expected: bool) {
        let m: StringMatcher = serde_json::from_str(&format!("{matcher_raw:?}")).unwrap();
        assert_eq!(m.matches(input), expected);
    }

    // --- raw ---

    #[rstest]
    #[case::exact("docker.io")]
    #[case::pattern("*.my-corp.com")]
    fn raw_returns_original_string(#[case] input: &str) {
        let m: StringMatcher = serde_json::from_str(&format!("{input:?}")).unwrap();
        assert_eq!(m.raw(), input);
    }

    // --- PartialEq ---

    #[rstest]
    #[case::exact_same("foo", "foo", true)]
    #[case::exact_different("foo", "bar", false)]
    #[case::pattern_same("foo*", "foo*", true)]
    #[case::pattern_different("foo*", "bar*", false)]
    fn eq(#[case] a: &str, #[case] b: &str, #[case] expected: bool) {
        let a: StringMatcher = serde_json::from_str(&format!("{a:?}")).unwrap();
        let b: StringMatcher = serde_json::from_str(&format!("{b:?}")).unwrap();
        assert_eq!(a == b, expected);
    }

    #[test]
    fn exact_and_pattern_never_equal() {
        // Construct both variants with the same raw string to isolate PartialEq
        let exact = StringMatcher::Exact("foo".to_string());
        let pattern = StringMatcher::Pattern {
            pattern: WildMatch::new("foo"),
            raw: "foo".to_string(),
        };
        assert_ne!(exact, pattern);
    }

    // --- Deserialize ---

    #[rstest]
    #[case::plain_string("docker.io", false)]
    #[case::glob_star("*.my-corp.com", true)]
    #[case::glob_question("v1.?.0", true)]
    fn deserialize_variant(#[case] input: &str, #[case] is_pattern: bool) {
        let m: StringMatcher = serde_json::from_str(&format!("{input:?}")).unwrap();
        assert_eq!(matches!(m, StringMatcher::Pattern { .. }), is_pattern);
        assert_eq!(m.raw(), input);
    }

    // --- Serialize ---

    #[rstest]
    #[case::exact("docker.io")]
    #[case::pattern("*.my-corp.com")]
    fn serialize_roundtrip(#[case] input: &str) {
        let m: StringMatcher = serde_json::from_str(&format!("{input:?}")).unwrap();
        let serialized = serde_json::to_string(&m).unwrap();
        assert_eq!(serialized, format!("{input:?}"));
    }

    // --- Display ---

    #[rstest]
    #[case::exact("docker.io")]
    #[case::pattern("*.my-corp.com")]
    fn display_shows_raw(#[case] input: &str) {
        let m: StringMatcher = serde_json::from_str(&format!("{input:?}")).unwrap();
        assert_eq!(format!("{m}"), input);
    }
}
