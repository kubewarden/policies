use std::{collections::HashSet, str::FromStr};

use kubewarden_policy_sdk::settings::Validatable;
use oci_spec::distribution::Reference;
use serde::{Deserialize, Serialize};

use crate::matchers::{image::ImageMatcher, registry::RegistryMatcher, tag::TagMatcher};

// --- Structs ---

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub(crate) struct Registries {
    pub allow: HashSet<RegistryMatcher>,
    pub reject: HashSet<RegistryMatcher>,
}

impl Registries {
    pub(crate) fn validate(&self) -> Result<(), String> {
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
    pub(crate) fn validate(&self) -> Result<(), String> {
        use crate::matchers::string::StringMatcher;
        let invalid_tags: Vec<String> = self
            .reject
            .iter()
            .filter(|tag: &&TagMatcher| match &tag.0 {
                StringMatcher::Exact(t) => {
                    Reference::from_str(format!("hello:{t}").as_str()).is_err()
                }
                StringMatcher::Pattern { raw, .. } => raw.is_empty(),
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
    pub(crate) fn validate(&self) -> Result<(), String> {
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

    use crate::matchers::string::StringMatcher;

    fn exact_registry(s: &str) -> RegistryMatcher {
        RegistryMatcher(StringMatcher::Exact(s.to_string()))
    }

    fn exact_tag(s: &str) -> TagMatcher {
        TagMatcher(StringMatcher::Exact(s.to_string()))
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
