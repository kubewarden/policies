use std::{collections::HashSet, str::FromStr};

use k8s_openapi::api::core::v1 as apicore;
use oci_spec::distribution::Reference;

use crate::{
    matchers::image::{ImageMatcher, ImageRef},
    matchers::registry::RegistryMatcher,
    matchers::tag::TagMatcher,
    settings::Settings,
    validation_result::{PodRejectionReasons, PodSpecValidationResult},
};

pub(crate) fn validate_pod_spec(
    pod_spec: &apicore::PodSpec,
    settings: &Settings,
) -> PodSpecValidationResult {
    let images = discover_images(pod_spec);

    validate_images(&images, settings)
}

fn validate_images(images: &HashSet<&str>, settings: &Settings) -> PodSpecValidationResult {
    let mut rejection_reasons = PodRejectionReasons::default();

    for image in images {
        let image_ref = Reference::from_str(image);
        if let Ok(image_ref) = image_ref {
            if !is_allowed_registry(image_ref.registry(), settings) {
                rejection_reasons
                    .registries_not_allowed
                    .insert(image_ref.registry().to_owned());
            }

            let tag = image_ref.tag().unwrap_or("latest");
            if !is_allowed_tag(tag, settings) {
                rejection_reasons.tags_not_allowed.insert(tag.to_owned());
            }

            if !is_allowed_image(&image_ref.into(), settings) {
                rejection_reasons
                    .images_not_allowed
                    .insert(image.to_string());
            }
        }
    }

    if rejection_reasons.is_empty() {
        PodSpecValidationResult::Allowed
    } else {
        PodSpecValidationResult::NotAllowed(rejection_reasons)
    }
}

fn discover_images(pod_spec: &apicore::PodSpec) -> HashSet<&str> {
    let init_containers_images: Vec<&str> = pod_spec
        .init_containers
        .as_ref()
        .and_then(|containers| {
            containers
                .iter()
                .map(|container| container.image.as_deref())
                .collect()
        })
        .unwrap_or_default();

    let ephemeral_containers_images: Vec<&str> = pod_spec
        .ephemeral_containers
        .as_ref()
        .and_then(|containers| {
            containers
                .iter()
                .map(|container| container.image.as_deref())
                .collect()
        })
        .unwrap_or_default();

    let container_images: Vec<&str> = pod_spec
        .containers
        .iter()
        .filter_map(|container| container.image.as_deref())
        .collect();

    init_containers_images
        .into_iter()
        .chain(ephemeral_containers_images)
        .chain(container_images)
        .collect()
}

fn registry_matches_any(registry: &str, matchers: &HashSet<RegistryMatcher>) -> bool {
    matchers.iter().any(|m| m.matches(registry))
}

fn is_allowed_registry(registry: &str, settings: &Settings) -> bool {
    // Keep in mind the settings are validate to prevent both allow and reject
    // lists to be populated at the same time

    // if no configuration has been given for registries, we allow all
    if settings.registries.allow.is_empty() && settings.registries.reject.is_empty() {
        return true;
    }

    // if the registry is explicitly rejected, it is not allowed
    if !settings.registries.reject.is_empty()
        && registry_matches_any(registry, &settings.registries.reject)
    {
        return false;
    }

    if !settings.registries.allow.is_empty() {
        return registry_matches_any(registry, &settings.registries.allow);
    }

    true
}

fn tag_matches_any(tag: &str, matchers: &HashSet<TagMatcher>) -> bool {
    matchers.iter().any(|m| m.matches(tag))
}

fn is_allowed_tag(tag: &str, settings: &Settings) -> bool {
    if settings.tags.reject.is_empty() {
        return true;
    }

    !tag_matches_any(tag, &settings.tags.reject)
}

fn image_matches_any(image_ref: &ImageRef, matchers: &HashSet<ImageMatcher>) -> bool {
    matchers.iter().any(|m| match m {
        ImageMatcher::Exact(exact_ref) => {
            // Exact match
            if exact_ref == image_ref {
                return true;
            }

            // Loose match: repository only (without registry, tag, or digest)
            let contained_in_set_with_same_repo = Reference::from_str(image_ref.repository())
                .ok()
                .map(|r| &ImageRef::new(r) == exact_ref)
                .unwrap_or(false);
            if contained_in_set_with_same_repo {
                return true;
            }

            // Loose match: registry + repository (without tag or digest)
            let registry_repo = format!("{}/{}", image_ref.registry(), image_ref.repository());
            Reference::from_str(&registry_repo)
                .ok()
                .map(|r| &ImageRef::new(r) == exact_ref)
                .unwrap_or(false)
        }
        ImageMatcher::Pattern(sm) => {
            let whole = image_ref.whole();

            // Match against fully normalized image reference
            if sm.matches(&whole) {
                return true;
            }

            // If pattern has no tag/digest component, also match against registry/repository
            let has_tag_or_digest = sm.raw().contains(':') || sm.raw().contains('@');
            if !has_tag_or_digest {
                let registry_repo = format!("{}/{}", image_ref.registry(), image_ref.repository());
                if sm.matches(&registry_repo) {
                    return true;
                }

                // Also try matching against just the repository for Docker Hub shorthand
                if sm.matches(image_ref.repository()) {
                    return true;
                }
            }

            false
        }
    })
}

fn is_allowed_image(image_ref: &ImageRef, settings: &Settings) -> bool {
    // Keep in mind the settings are validate to prevent both allow and reject
    // lists to be populated at the same time

    // Accept/Reject if the allow/reject list contains either:
    // - The full image ref (exact match)
    //
    // - The image repository, without registry, nor tag, nor digest:
    //   allow "nginx" matches "nginx:1.21", "nginx:latest", "docker.io/library/nginx:1.21"
    //
    // - The image registry+repository, without tag nor digest:
    //   allow "quay.io/coreos/etcd" matches "quay.io/coreos/etcd:1.21", "quay.io/coreos/etcd:latest"
    //   allow "nginx" matches "nginx:1.21", "nginx:latest", "docker.io/library:nginx:1.21"
    //
    // - A wildcard pattern matching the normalized image reference

    // If no configuration has been given for images, we allow all
    if settings.images.allow.is_empty() && settings.images.reject.is_empty() {
        return true;
    }

    if !settings.images.reject.is_empty() && image_matches_any(image_ref, &settings.images.reject) {
        return false;
    }

    if !settings.images.allow.is_empty() {
        if image_matches_any(image_ref, &settings.images.allow) {
            return true;
        }
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::*;
    use wildmatch::WildMatch;

    use crate::{
        matchers::string::StringMatcher,
        settings::{Images, Registries, Tags},
    };

    fn exact_registry(s: &str) -> RegistryMatcher {
        RegistryMatcher(StringMatcher::Exact(s.to_string()))
    }

    fn pattern_registry(s: &str) -> RegistryMatcher {
        RegistryMatcher(StringMatcher::Pattern {
            pattern: WildMatch::new(s),
            raw: s.to_string(),
        })
    }

    fn exact_tag(s: &str) -> TagMatcher {
        TagMatcher(StringMatcher::Exact(s.to_string()))
    }

    fn pattern_tag(s: &str) -> TagMatcher {
        TagMatcher(StringMatcher::Pattern {
            pattern: WildMatch::new(s),
            raw: s.to_string(),
        })
    }

    fn exact_image(s: &str) -> ImageMatcher {
        ImageMatcher::Exact(ImageRef::new(Reference::from_str(s).unwrap()))
    }

    fn pattern_image(s: &str) -> ImageMatcher {
        ImageMatcher::Pattern(StringMatcher::Pattern {
            pattern: WildMatch::new(s),
            raw: s.to_string(),
        })
    }

    #[rstest]
    #[case::empty_pod_spec(
        apicore::PodSpec {
            containers: Vec::new(),
            init_containers: None,
            ephemeral_containers: None,
            ..apicore::PodSpec::default()
        },
        Vec::new(),
    )]
    #[case::main_containers(
        apicore::PodSpec {
            containers: vec![
                apicore::Container {
                    image: Some("busybox:1.0.0".to_string()),
                    ..apicore::Container::default()
                },
                apicore::Container {
                    image: Some("alpine:3.12".to_string()),
                    ..apicore::Container::default()
                },
            ],
            init_containers: None,
            ephemeral_containers: None,
            ..apicore::PodSpec::default()
        },
        vec!["busybox:1.0.0", "alpine:3.12"],
    )]
    #[case::init_containers(
        apicore::PodSpec {
            containers: vec![
                apicore::Container {
                    image: Some("busybox:1.0.0".to_string()),
                    ..apicore::Container::default()
                },
            ],
            init_containers: Some(vec![
                apicore::Container {
                    image: Some("busybox:1.0.0".to_string()),
                    ..apicore::Container::default()
                },
                apicore::Container {
                    image: Some("alpine:3.12".to_string()),
                    ..apicore::Container::default()
                },
            ]),
            ephemeral_containers: None,
            ..apicore::PodSpec::default()
        },
        vec!["busybox:1.0.0", "alpine:3.12"],
    )]
    #[case::ephemeral_containers(
        apicore::PodSpec {
            containers: vec![
                apicore::Container {
                    image: Some("busybox:1.0.0".to_string()),
                    ..apicore::Container::default()
                },
            ],
            init_containers: None,
            ephemeral_containers: Some(vec![
                apicore::EphemeralContainer {
                    image: Some("busybox:1.0.0".to_string()),
                    ..apicore::EphemeralContainer::default()
                },
                apicore::EphemeralContainer {
                    image: Some("alpine:3.12".to_string()),
                    ..apicore::EphemeralContainer::default()
                },
            ]),
            ..apicore::PodSpec::default()
        },
        vec!["busybox:1.0.0", "alpine:3.12"],
    )]
    fn discover_images_from_pod_spec(
        #[case] pod_spec: apicore::PodSpec,
        #[case] expected_images: Vec<&str>,
    ) {
        let images: HashSet<&str> = discover_images(&pod_spec);
        let expected_images: HashSet<&str> = expected_images.into_iter().collect();
        assert_eq!(
            images, expected_images,
            "got {images:?} instead of {expected_images:?}"
        );
    }

    #[rstest]
    #[case::block_implicit_latest(
        vec!["busybox"],
        vec![exact_tag("latest")],
        Err(vec!["latest"]),
    )]
    #[case::tag_part_of_reject_list(
        vec!["busybox:latest"],
        vec![exact_tag("latest")],
        Err(vec!["latest"]),
    )]
    #[case::tag_not_part_of_reject_list(
        vec!["busybox:1.0.0"],
        vec![exact_tag("latest")],
        Ok(()),
    )]
    #[case::tag_pattern_reject_rc(
        vec!["busybox:1.0.0-rc1"],
        vec![pattern_tag("*-rc*")],
        Err(vec!["1.0.0-rc1"]),
    )]
    #[case::tag_pattern_no_match(
        vec!["busybox:1.0.0"],
        vec![pattern_tag("*-rc*")],
        Ok(()),
    )]
    fn validation_with_rejected_tags_constraint(
        #[case] images: Vec<&str>,
        #[case] settings_tags_rejected: Vec<TagMatcher>,
        #[case] expected_result: Result<(), Vec<&str>>,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let settings = Settings {
            tags: Tags {
                reject: settings_tags_rejected.into_iter().collect(),
            },
            ..Settings::default()
        };
        let expected_result = if let Err(tags_not_allowed) = expected_result {
            let tags_not_allowed = tags_not_allowed
                .into_iter()
                .map(|image| image.to_string())
                .collect();
            PodSpecValidationResult::NotAllowed(PodRejectionReasons {
                tags_not_allowed,
                ..PodRejectionReasons::default()
            })
        } else {
            PodSpecValidationResult::Allowed
        };

        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            "got: {result:?} instead of {expected_result:?}"
        );
    }

    #[rstest]
    #[case::image_from_registry_part_of_the_reject_list(
        vec!["busybox:1.0.0", "ghcr.io/kubewarden/policy-server:1.0.0"],
        vec![exact_registry("docker.io"), exact_registry("ghcr.io")],
        Err(vec!["docker.io", "ghcr.io"]),
    )]
    #[case::image_from_registry_not_part_of_the_reject_list(
        vec!["ghcr.io/kubewarden/policy-server:1.0.0"],
        vec![exact_registry("docker.io")],
        Ok(()),
    )]
    #[case::registry_pattern_reject(
        vec!["ghcr.io/kubewarden/policy-server:1.0.0"],
        vec![pattern_registry("ghcr.*")],
        Err(vec!["ghcr.io"]),
    )]
    #[case::registry_pattern_no_match(
        vec!["ghcr.io/kubewarden/policy-server:1.0.0"],
        vec![pattern_registry("*.example.com")],
        Ok(()),
    )]
    fn validation_with_registry_reject_constraint(
        #[case] images: Vec<&str>,
        #[case] settings_registries_to_reject: Vec<RegistryMatcher>,
        #[case] expected_result: Result<(), Vec<&str>>,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let settings = Settings {
            registries: Registries {
                reject: settings_registries_to_reject.into_iter().collect(),
                ..Registries::default()
            },
            ..Settings::default()
        };
        let expected_result = if let Err(registries_not_allowed) = expected_result {
            let registries_not_allowed = registries_not_allowed
                .into_iter()
                .map(|image| image.to_string())
                .collect();
            PodSpecValidationResult::NotAllowed(PodRejectionReasons {
                registries_not_allowed,
                ..PodRejectionReasons::default()
            })
        } else {
            PodSpecValidationResult::Allowed
        };

        let images: HashSet<&str> = images.into_iter().collect();
        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            "got: {result:?} instead of {expected_result:?}"
        );
    }

    #[rstest]
    #[case::image_from_registry_not_part_of_the_allow_list(
        vec!["busybox:1.0.0", "docker.io/alpine:1.0.0", "ghcr.io/kubewarden/policy-server:1.0.0"],
        vec![exact_registry("ghcr.io")],
        Err(vec!["docker.io"]),
    )]
    #[case::image_from_registry_part_of_the_allow_list(
        vec!["busybox:1.0.0", "docker.io/alpine:1.0.0", "ghcr.io/kubewarden/policy-server:1.0.0"],
        vec![exact_registry("ghcr.io"), exact_registry("docker.io")],
        Ok(()),
    )]
    #[case::registry_pattern_allow(
        vec!["busybox:1.0.0", "docker.io/alpine:1.0.0", "ghcr.io/kubewarden/policy-server:1.0.0"],
        vec![pattern_registry("*.io")],
        Ok(()),
    )]
    #[case::registry_pattern_allow_partial(
        vec!["busybox:1.0.0", "registry.my-corp.com/app:1.0.0"],
        vec![pattern_registry("*.my-corp.com")],
        Err(vec!["docker.io"]),
    )]
    fn validation_with_registry_allow_constraint(
        #[case] images: Vec<&str>,
        #[case] settings_registries_to_allow: Vec<RegistryMatcher>,
        #[case] expected_result: Result<(), Vec<&str>>,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let settings = Settings {
            registries: Registries {
                allow: settings_registries_to_allow.into_iter().collect(),
                ..Registries::default()
            },
            ..Settings::default()
        };
        let expected_result = if let Err(registries_not_allowed) = expected_result {
            let registries_not_allowed = registries_not_allowed
                .into_iter()
                .map(|image| image.to_string())
                .collect();
            PodSpecValidationResult::NotAllowed(PodRejectionReasons {
                registries_not_allowed,
                ..PodRejectionReasons::default()
            })
        } else {
            PodSpecValidationResult::Allowed
        };

        let images: HashSet<&str> = images.into_iter().collect();
        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            "got: {result:?} instead of {expected_result:?}"
        );
    }

    #[rstest]
    #[case::image_not_part_of_the_allow_list(
        vec![
            "busybox:1.0.0",
            "docker.io/alpine:1.0.0",
            "ghcr.io/kubewarden/policy-server:1.0.0",
            "quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079",
            "quay.io/coreos/etcd:v3.4.12",
        ],
        vec![
            exact_image("ghcr.io/kubewarden/policy-server:1.0.0"),
            exact_image("quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079"),
            exact_image("quay.io/coreos/etcd:v3.4.12@sha256:7ed2739c96eb16de3d7169e2a0aa4ccf3a1f44af24f2bb6cad826935a51bcb3d"),
        ],
        Err(
            vec![
                "busybox:1.0.0",
                "docker.io/alpine:1.0.0",
                "quay.io/coreos/etcd:v3.4.12",
            ]),
    )]
    #[case::image_part_of_the_allow_list(
        vec![
            "ghcr.io/kubewarden/policy-server:1.0.0",
            "quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079",
        ],
        vec![
            exact_image("ghcr.io/kubewarden/policy-server:1.0.0"),
            exact_image("quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079"),
            exact_image("quay.io/coreos/etcd:v3.4.12@sha256:7ed2739c96eb16de3d7169e2a0aa4ccf3a1f44af24f2bb6cad826935a51bcb3d"),
        ],
        Ok(()),
    )]
    #[case::image_from_dockerio_with_any_tag_part_of_the_allow_list(
        vec![
            "nginx:1.21",
            "docker.io/library/nginx:1.21",
        ],
        vec![exact_image("nginx")],
        Ok(()),
    )]
    #[case::image_with_any_tag_part_of_the_allow_list(
        vec!["quay.io/coreos/etcd:v3.4.12"],
        vec![exact_image("quay.io/coreos/etcd")],
        Ok(()),
    )]
    #[case::image_with_implicit_tag_latest_part_of_the_allow_list(
        vec!["nginx", "quay.io/coreos/etcd"],
        vec![exact_image("nginx"), exact_image("quay.io/coreos/etcd")],
        Ok(()),
    )]
    #[case::image_with_implicit_tag_latest_not_part_of_the_allow_list(
        vec!["coreos/etcd", "coreos/etcd:v3.4.12"],
        vec![exact_image("quay.io/coreos/etcd")],
        Err(
            vec![
                "coreos/etcd",
                "coreos/etcd:v3.4.12",
            ]),
    )]
    fn validation_with_image_allow_constraint(
        #[case] images: Vec<&str>,
        #[case] settings_images_to_allow: Vec<ImageMatcher>,
        #[case] expected_result: Result<(), Vec<&str>>,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let settings = Settings {
            images: Images {
                allow: settings_images_to_allow.into_iter().collect(),
                ..Images::default()
            },
            ..Settings::default()
        };
        let expected_result = if let Err(images_not_allowed) = expected_result {
            let images_not_allowed = images_not_allowed
                .into_iter()
                .map(|image| image.to_string())
                .collect();
            PodSpecValidationResult::NotAllowed(PodRejectionReasons {
                images_not_allowed,
                ..PodRejectionReasons::default()
            })
        } else {
            PodSpecValidationResult::Allowed
        };

        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            r#"got: {result:?} instead of {expected_result:?}"#
        );
    }

    #[rstest]
    #[case::image_not_part_of_the_reject_list(
        vec![
            "busybox:1.0.0",
            "docker.io/alpine:1.0.0",
            "ghcr.io/kubewarden/policy-server:1.0.0",
            "quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079",
            "quay.io/coreos/etcd:v3.4.12",
        ],
        vec![
            exact_image("ghcr.io/kubewarden/policy-server:1.0.0"),
            exact_image("quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079"),
            exact_image("quay.io/coreos/etcd:v3.4.12@sha256:7ed2739c96eb16de3d7169e2a0aa4ccf3a1f44af24f2bb6cad826935a51bcb3d"),
        ],
        Err(
          vec![
            "ghcr.io/kubewarden/policy-server:1.0.0",
            "quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079",
        ]),
    )]
    #[case::image_part_of_the_reject_list(
        vec![
            "ghcr.io/kubewarden/policy-server:1.0.0",
            "quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079",
        ],
        vec![
            exact_image("ghcr.io/kubewarden/policy-server:1.0.0"),
            exact_image("quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079"),
            exact_image("quay.io/coreos/etcd"),
        ],
        Err(
          vec![
            "ghcr.io/kubewarden/policy-server:1.0.0",
            "quay.io/bitnami/redis:6.0@sha256:82dfd9ac433eacb5f89e5bf2601659bbc78893c1a9e3e830c5ef4eb489fde079",
        ]),
    )]
    #[case::image_from_dockerio_with_any_tag_part_of_the_reject_list(
        vec![
            "nginx:1.21",
            "docker.io/library/nginx:1.21",
        ],
        vec![exact_image("nginx")],
        Err(
          vec![
            "nginx:1.21",
            "docker.io/library/nginx:1.21",
        ]),
    )]
    #[case::image_from_dockerio_no_match_part_of_the_reject_list(
        vec!["quay.io/coreos/etcd"],
        vec![exact_image("etcd")], // this is actually docker.io/library/etcd
        Ok(()),
    )]
    #[case::image_with_any_tag_part_of_the_reject_list(
        vec!["quay.io/coreos/etcd:v3.4.12"],
        vec![exact_image("quay.io/coreos/etcd")],
        Err(vec!["quay.io/coreos/etcd:v3.4.12"]),
    )]
    #[case::image_with_implicit_tag_latest_part_of_the_reject_list(
        vec!["nginx", "quay.io/coreos/etcd"],
        vec![exact_image("nginx"), exact_image("quay.io/coreos/etcd")],
        Err(
          vec![
            "nginx",
            "quay.io/coreos/etcd",
        ]),
    )]
    #[case::image_with_implicit_tag_latest_not_part_of_the_reject_list(
        vec!["coreos/etcd", "coreos/etcd:v3.4.12"], // these actually are docker.io/library/coreos/etcd
        vec![exact_image("quay.io/coreos/etcd")],
        Ok(()),
    )]
    fn validation_with_image_reject_constraint(
        #[case] images: Vec<&str>,
        #[case] settings_images_to_reject: Vec<ImageMatcher>,
        #[case] expected_result: Result<(), Vec<&str>>,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let settings = Settings {
            images: Images {
                reject: settings_images_to_reject.into_iter().collect(),
                ..Images::default()
            },
            ..Settings::default()
        };
        let expected_result = if let Err(images_not_allowed) = expected_result {
            let images_not_allowed = images_not_allowed
                .into_iter()
                .map(|image| image.to_string())
                .collect();
            PodSpecValidationResult::NotAllowed(PodRejectionReasons {
                images_not_allowed,
                ..PodRejectionReasons::default()
            })
        } else {
            PodSpecValidationResult::Allowed
        };

        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            r#"got: {result:?} instead of {expected_result:?}"#
        );
    }

    #[rstest]
    #[case::empty_settings(
        vec!["busybox"],
        Settings::default(),
        PodSpecValidationResult::Allowed)]
    #[case::registry_allowed_but_tag_rejected(
        vec!["busybox"],
        Settings{
            registries: Registries {
                allow: vec![exact_registry("docker.io")].into_iter().collect(),
                ..Registries::default()
            },
            tags: Tags {
                reject: vec![exact_tag("latest")].into_iter().collect(),
            },
            ..Settings::default()
        },
        PodSpecValidationResult::NotAllowed(PodRejectionReasons {
            tags_not_allowed: vec!["latest".to_string()].into_iter().collect(),
            ..PodRejectionReasons::default()
        }),
    )]
    #[case::registry_allowed_but_image_rejected(
        vec!["busybox:1.0.0"],
        Settings{
            registries: Registries {
                allow: vec![exact_registry("docker.io")].into_iter().collect(),
                ..Registries::default()
            },
            images: Images {
                reject: vec![exact_image("busybox:1.0.0")].into_iter().collect(),
                ..Images::default()
            },
            ..Settings::default()
        },
        PodSpecValidationResult::NotAllowed(PodRejectionReasons {
            images_not_allowed: vec!["busybox:1.0.0".to_string()].into_iter().collect(),
            ..PodRejectionReasons::default()
        }),
    )]
    #[case::registry_allowed_and_image_not_rejected(
        vec!["busybox:2.0.0"],
        Settings{
            registries: Registries {
                allow: vec![exact_registry("docker.io")].into_iter().collect(),
                ..Registries::default()
            },
            images: Images {
                reject: vec![exact_image("busybox:1.0.0")].into_iter().collect(),
                ..Images::default()
            },
            ..Settings::default()
        },
        PodSpecValidationResult::Allowed,
    )]
    fn validation_with_special_settings(
        #[case] images: Vec<&str>,
        #[case] settings: Settings,
        #[case] expected_result: PodSpecValidationResult,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            "got: {result:?} instead of {expected_result:?}"
        );
    }

    // --- Wildcard/glob pattern tests ---

    #[rstest]
    #[case::image_wildcard_allow_bitnami(
        vec!["docker.io/bitnami/redis:6.0"],
        vec![pattern_image("docker.io/bitnami/*")],
        Ok(()),
    )]
    #[case::image_wildcard_allow_no_match(
        vec!["quay.io/coreos/etcd:v3.4.12"],
        vec![pattern_image("docker.io/bitnami/*")],
        Err(vec!["quay.io/coreos/etcd:v3.4.12"]),
    )]
    #[case::image_wildcard_with_tag_pattern(
        vec!["ghcr.io/my-org/app:v1.2.3"],
        vec![pattern_image("ghcr.io/my-org/*:v1.*")],
        Ok(()),
    )]
    #[case::image_wildcard_with_tag_pattern_no_match(
        vec!["ghcr.io/my-org/app:v2.0"],
        vec![pattern_image("ghcr.io/my-org/*:v1.*")],
        Err(vec!["ghcr.io/my-org/app:v2.0"]),
    )]
    #[case::image_wildcard_docker_library(
        vec!["busybox:1.0.0"],
        vec![pattern_image("docker.io/library/*")],
        Ok(()),
    )]
    #[case::image_wildcard_mixed_with_exact(
        vec!["docker.io/bitnami/redis:6.0", "ghcr.io/kubewarden/policy-server:1.0.0"],
        vec![
            pattern_image("docker.io/bitnami/*"),
            exact_image("ghcr.io/kubewarden/policy-server:1.0.0"),
        ],
        Ok(()),
    )]
    #[case::image_wildcard_mixed_partial_match(
        vec!["docker.io/bitnami/redis:6.0", "quay.io/coreos/etcd:v3.4.12"],
        vec![pattern_image("docker.io/bitnami/*")],
        Err(vec!["quay.io/coreos/etcd:v3.4.12"]),
    )]
    fn validation_with_image_allow_wildcard(
        #[case] images: Vec<&str>,
        #[case] settings_images_to_allow: Vec<ImageMatcher>,
        #[case] expected_result: Result<(), Vec<&str>>,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let settings = Settings {
            images: Images {
                allow: settings_images_to_allow.into_iter().collect(),
                ..Images::default()
            },
            ..Settings::default()
        };
        let expected_result = if let Err(images_not_allowed) = expected_result {
            let images_not_allowed = images_not_allowed
                .into_iter()
                .map(|image| image.to_string())
                .collect();
            PodSpecValidationResult::NotAllowed(PodRejectionReasons {
                images_not_allowed,
                ..PodRejectionReasons::default()
            })
        } else {
            PodSpecValidationResult::Allowed
        };

        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            r#"got: {result:?} instead of {expected_result:?}"#
        );
    }

    #[rstest]
    #[case::image_wildcard_reject(
        vec!["docker.io/bitnami/redis:6.0"],
        vec![pattern_image("docker.io/bitnami/*")],
        Err(vec!["docker.io/bitnami/redis:6.0"]),
    )]
    #[case::image_wildcard_reject_no_match(
        vec!["ghcr.io/kubewarden/policy-server:1.0.0"],
        vec![pattern_image("docker.io/bitnami/*")],
        Ok(()),
    )]
    fn validation_with_image_reject_wildcard(
        #[case] images: Vec<&str>,
        #[case] settings_images_to_reject: Vec<ImageMatcher>,
        #[case] expected_result: Result<(), Vec<&str>>,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let settings = Settings {
            images: Images {
                reject: settings_images_to_reject.into_iter().collect(),
                ..Images::default()
            },
            ..Settings::default()
        };
        let expected_result = if let Err(images_not_allowed) = expected_result {
            let images_not_allowed = images_not_allowed
                .into_iter()
                .map(|image| image.to_string())
                .collect();
            PodSpecValidationResult::NotAllowed(PodRejectionReasons {
                images_not_allowed,
                ..PodRejectionReasons::default()
            })
        } else {
            PodSpecValidationResult::Allowed
        };

        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            r#"got: {result:?} instead of {expected_result:?}"#
        );
    }

    #[rstest]
    #[case::registry_wildcard_allow(
        vec!["registry.my-corp.com/app:1.0.0"],
        vec![pattern_registry("*.my-corp.com")],
        Ok(()),
    )]
    #[case::registry_wildcard_allow_no_match(
        vec!["docker.io/library/busybox:1.0.0"],
        vec![pattern_registry("*.my-corp.com")],
        Err(vec!["docker.io"]),
    )]
    fn validation_with_registry_wildcard(
        #[case] images: Vec<&str>,
        #[case] settings_registries_to_allow: Vec<RegistryMatcher>,
        #[case] expected_result: Result<(), Vec<&str>>,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let settings = Settings {
            registries: Registries {
                allow: settings_registries_to_allow.into_iter().collect(),
                ..Registries::default()
            },
            ..Settings::default()
        };
        let expected_result = if let Err(registries_not_allowed) = expected_result {
            let registries_not_allowed = registries_not_allowed
                .into_iter()
                .map(|r| r.to_string())
                .collect();
            PodSpecValidationResult::NotAllowed(PodRejectionReasons {
                registries_not_allowed,
                ..PodRejectionReasons::default()
            })
        } else {
            PodSpecValidationResult::Allowed
        };

        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            "got: {result:?} instead of {expected_result:?}"
        );
    }

    #[rstest]
    #[case::tag_wildcard_reject(
        vec!["busybox:1.0.0-rc1"],
        vec![pattern_tag("*-rc*")],
        Err(vec!["1.0.0-rc1"]),
    )]
    #[case::tag_wildcard_no_match(
        vec!["busybox:1.0.0"],
        vec![pattern_tag("*-rc*")],
        Ok(()),
    )]
    fn validation_with_tag_wildcard(
        #[case] images: Vec<&str>,
        #[case] settings_tags_to_reject: Vec<TagMatcher>,
        #[case] expected_result: Result<(), Vec<&str>>,
    ) {
        let images: HashSet<&str> = images.into_iter().collect();
        let settings = Settings {
            tags: Tags {
                reject: settings_tags_to_reject.into_iter().collect(),
            },
            ..Settings::default()
        };
        let expected_result = if let Err(tags_not_allowed) = expected_result {
            let tags_not_allowed = tags_not_allowed
                .into_iter()
                .map(|t| t.to_string())
                .collect();
            PodSpecValidationResult::NotAllowed(PodRejectionReasons {
                tags_not_allowed,
                ..PodRejectionReasons::default()
            })
        } else {
            PodSpecValidationResult::Allowed
        };

        let result = validate_images(&images, &settings);
        assert_eq!(
            result, expected_result,
            "got: {result:?} instead of {expected_result:?}"
        );
    }
}
