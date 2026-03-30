use std::{collections::HashSet, fmt, hash::Hash, str::FromStr};

use kubewarden_policy_sdk::settings::Validatable;
use oci_spec::distribution::Reference;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wildmatch::WildMatch;

fn is_glob_pattern(s: &str) -> bool {
    s.contains('*') || s.contains('?')
}

// --- RegistryMatcher ---

#[derive(Debug, Clone)]
pub enum RegistryMatcher {
    Exact(String),
    Pattern { pattern: WildMatch, raw: String },
}

impl RegistryMatcher {
    pub fn raw(&self) -> &str {
        match self {
            RegistryMatcher::Exact(s) => s,
            RegistryMatcher::Pattern { raw, .. } => raw,
        }
    }
}

impl PartialEq for RegistryMatcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RegistryMatcher::Exact(a), RegistryMatcher::Exact(b)) => a == b,
            (RegistryMatcher::Pattern { raw: a, .. }, RegistryMatcher::Pattern { raw: b, .. }) => {
                a == b
            }
            _ => false,
        }
    }
}

impl Eq for RegistryMatcher {}

impl Hash for RegistryMatcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        self.raw().hash(state);
    }
}

impl<'de> Deserialize<'de> for RegistryMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if is_glob_pattern(&s) {
            Ok(RegistryMatcher::Pattern {
                pattern: WildMatch::new(&s),
                raw: s,
            })
        } else {
            Ok(RegistryMatcher::Exact(s))
        }
    }
}

impl Serialize for RegistryMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.raw())
    }
}

impl fmt::Display for RegistryMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw())
    }
}

// --- TagMatcher ---

#[derive(Debug, Clone)]
pub enum TagMatcher {
    Exact(String),
    Pattern { pattern: WildMatch, raw: String },
}

impl TagMatcher {
    pub fn raw(&self) -> &str {
        match self {
            TagMatcher::Exact(s) => s,
            TagMatcher::Pattern { raw, .. } => raw,
        }
    }
}

impl PartialEq for TagMatcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TagMatcher::Exact(a), TagMatcher::Exact(b)) => a == b,
            (TagMatcher::Pattern { raw: a, .. }, TagMatcher::Pattern { raw: b, .. }) => a == b,
            _ => false,
        }
    }
}

impl Eq for TagMatcher {}

impl Hash for TagMatcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        self.raw().hash(state);
    }
}

impl<'de> Deserialize<'de> for TagMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if is_glob_pattern(&s) {
            Ok(TagMatcher::Pattern {
                pattern: WildMatch::new(&s),
                raw: s,
            })
        } else {
            Ok(TagMatcher::Exact(s))
        }
    }
}

impl Serialize for TagMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.raw())
    }
}

impl fmt::Display for TagMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw())
    }
}

// --- ImageMatcher ---

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
    Pattern { pattern: WildMatch, raw: String },
}

impl ImageMatcher {
    pub fn raw(&self) -> String {
        match self {
            ImageMatcher::Exact(image_ref) => image_ref.whole(),
            ImageMatcher::Pattern { raw, .. } => raw.clone(),
        }
    }
}

impl PartialEq for ImageMatcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ImageMatcher::Exact(a), ImageMatcher::Exact(b)) => a == b,
            (ImageMatcher::Pattern { raw: a, .. }, ImageMatcher::Pattern { raw: b, .. }) => a == b,
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
            ImageMatcher::Pattern { raw, .. } => raw.hash(state),
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
            Ok(ImageMatcher::Pattern {
                pattern: WildMatch::new(&s),
                raw: s,
            })
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

// --- Structs ---

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub(crate) struct Registries {
    pub allow: HashSet<RegistryMatcher>,
    pub reject: HashSet<RegistryMatcher>,
}

impl Registries {
    fn validate(&self) -> Result<(), String> {
        if !self.allow.is_empty() && !self.reject.is_empty() {
            return Err("only one of registries allow or reject can be provided".to_string());
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub(crate) struct Tags {
    pub reject: HashSet<TagMatcher>,
}

impl Tags {
    /// Validate the tags against the OCI spec
    fn validate(&self) -> Result<(), String> {
        let invalid_tags: Vec<String> = self
            .reject
            .iter()
            .filter(|tag| match tag {
                TagMatcher::Exact(t) => Reference::from_str(format!("hello:{t}").as_str()).is_err(),
                TagMatcher::Pattern { raw, .. } => raw.is_empty(),
            })
            .map(|tag| tag.raw().to_string())
            .collect();

        if !invalid_tags.is_empty() {
            return Err(format!(
                "tags {invalid_tags:?} are invalid, they must be valid OCI tags or wildcard patterns",
            ));
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub(crate) struct Images {
    pub allow: HashSet<ImageMatcher>,
    pub reject: HashSet<ImageMatcher>,
}

impl Images {
    /// An image cannot be present in both allow and reject lists
    fn validate(&self) -> Result<(), String> {
        if !self.allow.is_empty() && !self.reject.is_empty() {
            return Err("only one of images allow or reject can be provided".to_string());
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub(crate) struct Settings {
    pub registries: Registries,
    pub tags: Tags,
    pub images: Images,
}

impl Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        let errors = vec![
            self.registries.validate(),
            self.images.validate(),
            self.tags.validate(),
        ]
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<String>>();

        if !errors.is_empty() {
            return Err(errors.join(", "));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    fn exact_registry(s: &str) -> RegistryMatcher {
        RegistryMatcher::Exact(s.to_string())
    }

    fn pattern_registry(s: &str) -> RegistryMatcher {
        RegistryMatcher::Pattern {
            pattern: WildMatch::new(s),
            raw: s.to_string(),
        }
    }

    fn exact_tag(s: &str) -> TagMatcher {
        TagMatcher::Exact(s.to_string())
    }

    fn pattern_tag(s: &str) -> TagMatcher {
        TagMatcher::Pattern {
            pattern: WildMatch::new(s),
            raw: s.to_string(),
        }
    }

    fn exact_image(s: &str) -> ImageMatcher {
        ImageMatcher::Exact(ImageRef(Reference::from_str(s).unwrap()))
    }

    fn pattern_image(s: &str) -> ImageMatcher {
        ImageMatcher::Pattern {
            pattern: WildMatch::new(s),
            raw: s.to_string(),
        }
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

    #[rstest]
    #[case::empty_settings(Settings::default(), true)]
    #[case::valid_settings(
        Settings {
            registries: Registries {
                allow: vec![exact_registry("registry.com")].into_iter().collect(),
                ..Registries::default()
            },
            tags: Tags {
                reject: vec![exact_tag("latest")].into_iter().collect(),
            },
            images: Images {
                reject: vec!["busybox".to_string()].into_iter().map(|image| Reference::from_str(&image).unwrap().into()).collect(),
                ..Images::default()
            },
        },
        true
    )]
    #[case::bad_registries(
        Settings {
            registries: Registries {
                allow: vec![exact_registry("registry.com")].into_iter().collect(),
                reject: vec![exact_registry("registry2.com")].into_iter().collect(),
            },
            tags: Tags {
                reject: vec![exact_tag("latest")].into_iter().collect(),
            },
            images: Images {
                reject: vec!["busybox".to_string()].into_iter().map(|image| Reference::from_str(&image).unwrap().into()).collect(),
                ..Images::default()
            },
        },
        false
    )]
    fn validate_settings(#[case] settings: Settings, #[case] is_valid: bool) {
        let result = settings.validate();
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
        assert!(matches!(matcher, RegistryMatcher::Pattern { .. }));
    }

    #[test]
    fn deserialize_tag_pattern() {
        let json = r#"{"reject": ["*-rc*"]}"#;
        let tags: Tags = serde_json::from_str(json).unwrap();
        assert_eq!(tags.reject.len(), 1);
        let matcher = tags.reject.iter().next().unwrap();
        assert!(matches!(matcher, TagMatcher::Pattern { .. }));
    }

    #[test]
    fn deserialize_image_pattern() {
        let json = r#"{"allow": ["docker.io/bitnami/*"], "reject": []}"#;
        let images: Images = serde_json::from_str(json).unwrap();
        assert_eq!(images.allow.len(), 1);
        let matcher = images.allow.iter().next().unwrap();
        assert!(matches!(matcher, ImageMatcher::Pattern { .. }));
    }

    #[test]
    fn deserialize_settings_with_patterns() {
        let json = r#"{
            "registries": {"allow": ["*.my-corp.com"]},
            "tags": {"reject": ["*-rc*"]},
            "images": {"allow": ["docker.io/bitnami/*"]}
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.registries.allow.len(), 1);
        assert_eq!(settings.tags.reject.len(), 1);
        assert_eq!(settings.images.allow.len(), 1);

        let result = settings.validate();
        assert!(result.is_ok(), "{result:?}");
    }
}
