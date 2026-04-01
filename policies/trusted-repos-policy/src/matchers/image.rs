use std::{hash::Hash, str::FromStr};

use oci_spec::distribution::Reference;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wildmatch::WildMatch;

use crate::matchers::{is_glob_pattern, string::StringMatcher};

/// Custom type to represent an image reference. It's required to implement
/// the `Deserialize` trait to be able to use it in the `Settings` struct.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ImageRef(oci_spec::distribution::Reference);

impl ImageRef {
    pub fn new(reference: oci_spec::distribution::Reference) -> Self {
        ImageRef(reference)
    }

    pub fn whole(&self) -> String {
        self.0.whole()
    }

    pub fn repository(&self) -> &str {
        self.0.repository()
    }

    pub fn registry(&self) -> &str {
        self.0.registry()
    }
}

impl From<Reference> for ImageRef {
    fn from(reference: Reference) -> Self {
        ImageRef(reference)
    }
}

#[derive(Debug, Clone)]
pub enum ImageMatcher {
    Exact(ImageRef),
    Pattern(StringMatcher),
}

impl ImageMatcher {
    pub fn raw(&self) -> String {
        match self {
            ImageMatcher::Exact(image_ref) => image_ref.whole(),
            ImageMatcher::Pattern(sm) => sm.raw().to_string(),
        }
    }
}

impl PartialEq for ImageMatcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ImageMatcher::Exact(a), ImageMatcher::Exact(b)) => a == b,
            (ImageMatcher::Pattern(a), ImageMatcher::Pattern(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ImageMatcher {}

impl Hash for ImageMatcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            ImageMatcher::Exact(image_ref) => image_ref.hash(state),
            ImageMatcher::Pattern(sm) => sm.hash(state),
        }
    }
}

impl<'de> Deserialize<'de> for ImageMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if is_glob_pattern(&s) {
            Ok(ImageMatcher::Pattern(StringMatcher::Pattern {
                pattern: WildMatch::new(&s),
                raw: s,
            }))
        } else {
            let reference = Reference::from_str(&s).map_err(serde::de::Error::custom)?;
            Ok(ImageMatcher::Exact(ImageRef(reference)))
        }
    }
}

impl Serialize for ImageMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.raw())
    }
}

impl From<Reference> for ImageMatcher {
    fn from(reference: Reference) -> Self {
        ImageMatcher::Exact(ImageRef(reference))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::*;

    use crate::settings::Images;

    fn exact_image(s: &str) -> ImageMatcher {
        ImageMatcher::Exact(ImageRef(Reference::from_str(s).unwrap()))
    }

    fn pattern_image(s: &str) -> ImageMatcher {
        ImageMatcher::Pattern(StringMatcher::Pattern {
            pattern: WildMatch::new(s),
            raw: s.to_string(),
        })
    }

    #[rstest]
    #[case::empty_settings(Vec::new(), Vec::new(), true)]
    #[case::allow_only(vec![exact_image("allowed-image")], Vec::new(), true)]
    #[case::reject_only(Vec::new(), vec![exact_image("forbidden-image")], true)]
    #[case::allow_and_reject(
        vec![exact_image("allowed-image.com")],
        vec![exact_image("forbidden-image.com")],
        false
    )]
    #[case::allow_pattern(vec![pattern_image("docker.io/bitnami/*")], Vec::new(), true)]
    fn validate_images(
        #[case] allow: Vec<ImageMatcher>,
        #[case] reject: Vec<ImageMatcher>,
        #[case] is_valid: bool,
    ) {
        let images = Images {
            allow: allow.into_iter().collect(),
            reject: reject.into_iter().collect(),
        };

        let result = images.validate();
        if is_valid {
            assert!(result.is_ok(), "{result:?}");
        } else {
            assert!(result.is_err(), "was supposed to be invalid");
        }
    }

    #[rstest]
    #[case::good_input(
        r#"{
            "allow": [],
            "reject": [
                "busybox",
                "busybox:latest",
                "registry.com/image@sha256:3fc9b689459d738f8c88a3a48aa9e33542016b7a4052e001aaa536fca74813cb",
                "quay.io/etcd/etcd:1.1.1@sha256:3fc9b689459d738f8c88a3a48aa9e33542016b7a4052e001aaa536fca74813cb"
            ]
        }"#,
        true
    )]
    #[case::bad_input(
        r#"{
            "allow": [],
            "reject": [
                "busybox",
                "registry.com/image@sha256",
            ]
        }"#,
        false
    )]
    #[case::pattern_input(
        r#"{
            "allow": ["docker.io/bitnami/*"],
            "reject": []
        }"#,
        true
    )]
    fn deserialize_images(#[case] input: &str, #[case] valid: bool) {
        let image: Result<Images, _> = serde_json::from_str(input);
        if valid {
            assert!(image.is_ok(), "{image:?}");
        } else {
            assert!(image.is_err(), "was supposed to be invalid");
        }
    }

    #[test]
    fn deserialize_image_pattern() {
        let json = r#"{"allow": ["docker.io/bitnami/*"], "reject": []}"#;
        let images: Images = serde_json::from_str(json).unwrap();
        assert_eq!(images.allow.len(), 1);
        let matcher = images.allow.iter().next().unwrap();
        assert!(matches!(
            matcher,
            ImageMatcher::Pattern(StringMatcher::Pattern { .. })
        ));
    }
}
