[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

## Container resources policy

This policy is designed to enforce constraints on the resource requirements of
Kubernetes containers. It follows a two-step verification process: initially
checking whether the container has defined resource requirements, and
subsequently ensuring that these resources fall within the permissible range
set by the minimum and maximum resource requirements configured in the policy
settings.

## Configuration

Users can configure the policy using the following parameters in YAML format:

```yaml
# optional
memory:
  defaultRequest: "100M"
  defaultLimit: "500M"
  maxRequest: "2G"
  minRequest: "50M"
  maxLimit: "4G"
  minLimit: "200M"
# optional
cpu:
  defaultRequest: 100m
  defaultLimit: 200m
  maxRequest: 300m
  minRequest: 50m
  maxLimit: 500m
  minLimit: 100m
# optional
ignoreImages: ["ghcr.io/foo/bar:1.23", "myimage", "otherimages:v1"]
```

### `memory` and `cpu`

You must define at least one of `cpu` or `memory`, you cannot leave both empty.
All CPU and memory quantities should be expressed in the [Kubernetes quantity
format](https://kubernetes.io/docs/reference/kubernetes-api/common-definitions/quantity/).

If you don't care about specific quantities of a resource, but you want to enforce
that requests and limits are set for that resource, set `ignoreValues` to `true` for that
resource (default is `false`).

For example, here we enforce that memory has limits or requests without
enforcing specific quantities, but we do it for CPU:

```yaml
# optional
memory:
  ignoreValues: true
# optional
cpu:
  defaultRequest: 100m
  defaultLimit: 200m
  maxRequest: 300m
  minRequest: 50m
  maxLimit: 500m
  minLimit: 100m
# optional
ignoreImages: ["ghcr.io/foo/bar:1.23", "myimage", "otherimages:v1"]
```

> [!NOTE]
> The admission request review evaluated by the policy could be mutated by
> another admission controller, like the LimitRange admission controller. This
> means that the policy could accept a resource that at first looks invalid, but
> that is later mutated by another admission controller to be valid. For example,
> LimitRange will set the default request values if they are not set.

### `ignoreImages`

The `ignoreImages` configuration can be used to exclude containers from
enforcement. Any container image that matches an entry in the list will be
skipped. Prefix-matching can be signified with `*`. For example: `my-image-*`.
It is recommended that users use the fully-qualified Docker image name (e.g. start with a domain name)
in order to avoid unexpectedly exempting images from an untrusted repository.

### Policy settings verification

The policy verifies the consistency of the values provided.

When all values are configured (non‑zero), they must satisfy the following
overall ordering:

- `minRequest` ≤ `defaultRequest` ≤ `maxRequest` ≤ `minLimit` ≤ `defaultLimit` ≤ `maxLimit`

Only comparisons between values that are **both** configured (non‑zero) are
enforced. If a value is left at zero, it is treated as "not configured" for the
purposes of these checks.

Full example of policy definition:

```yaml
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  name: container-resources-policy
spec:
  policyServer: container-resources-policy
  module: registry://ghcr.io/kubewarden/policies/container-resources:latest
  rules:
    - apiGroups: [""]
      apiVersions: ["v1"]
      resources: ["pods"]
      operations:
        - CREATE
        - UPDATE
  mutating: true
  settings:
    memory:
      defaultRequest: "1G"
      defaultLimit: "1G"
      maxLimit: "4G"
      maxRequest: "2G"
      minRequest: "500M"
      minLimit: "500M"
    cpu:
      defaultRequest: 1
      defaultLimit: 1
      maxLimit: 2
      maxRequest: 1.5
      minRequest: 0.5
      minLimit: 0.5
    ignoreImages: ["myimage:latest", "registry.k8s.io/pause"]
```

## Behavior

The policy skips all the containers that are using an image that is part of the
`ignoreImages` list. These containers are always considered valid and are never
mutated.

For containers that are **not** skipped:
- **Limits**:
  - If the limit is missing and `defaultLimit` is configured, the limit is mutated to `defaultLimit`.
  - If the limit is present, it must fall within the configured range (`minLimit`/`maxLimit`), when these are set.
- **Requests**:
  - If the request is missing and `defaultRequest` is configured, the request is mutated to `defaultRequest`.
  - If the request is present, it must fall within the configured range (`minRequest`/`maxRequest`), when these are set.
- **Consistency between request and limit**:
  - After any mutation, the effective limit must be **greater than or equal to** the effective request for each resource.
  - If this is not the case, the request is rejected so that Kubernetes will not receive an inconsistent Pod spec.
