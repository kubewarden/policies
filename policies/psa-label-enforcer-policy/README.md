[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Kubewarden Policy: PSA Label Enforcer

This policy ensures that namespaces have the required
[PSA labels](https://kubernetes.io/docs/concepts/security/pod-security-admission/)
configuration for deployment in the cluster.

The policy validates whether the PSA labels defined in the namespace comply with
the configuration specified in the policy settings. If the labels do not match
the configuration, the policy will modify the required labels to meet the requirements.

## Settings

> Note: The configuration format of this policy intentionally resembles that of the
[PSA admission controller](https://kubernetes.io/docs/tasks/configure-pod-container/enforce-standards-admission-controller/#configure-the-admission-controller).

The policy settings consist of fields where the user can define the pod security
levels and versions to be used in the deployed namespace. For example, consider
the following policy settings:

```yaml
modes:
  enforce: "baseline"
  enforce-version: "latest"
  audit: "restricted"
  audit-version: "v1.27"
  warn: "privileged"
  warn-version: "v1.25"
```

The above configuration ensures that all namespaces deployed in the cluster will
have the following set of PSA labels:

```
pod-security.kubernetes.io/enforce: baseline
pod-security.kubernetes.io/enforce-version: latest
pod-security.kubernetes.io/audit: restricted
pod-security.kubernetes.io/audit-version: v1.27
pod-security.kubernetes.io/warn: privileged
pod-security.kubernetes.io/warn-version: v1.25
```

> Note: If you want to know what these labels mean, please refer to the
[Kubernetes documentation](https://kubernetes.io/docs/concepts/security/pod-security-admission/).

While it is not necessary to define all the modes in the policy settings, at least
one mode must be defined, with or without the mode version. For example, the
following is a valid configuration:

```yaml
modes:
  enforce: "baseline"
  enforce-version: "latest"
```

In the above configuration, if a deployed namespace already has labels for the `audit`
and `warn` modes in its definition, the policy will not modify them. The policy only
updates the labels for the modes defined in the policy settings.

The mode level fields allow three values: `baseline`, `restricted`, and `privileged`.

It is not permitted to define the mode version without specifying the mode level.
The mode version must follow the `v<major>.<minor>` version pattern or use the `latest` value.
