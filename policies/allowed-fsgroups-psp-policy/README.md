[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Kubewarden policy psp-allowed-fsgroups

Replacement for the Kubernetes Pod Security Policy that controls the
usage of `fsGroup` in the pod security context.

## Settings

This policy works by defining what `fsGroup` is allowed in the pod security context.

One of the following setting keys are accepted for this policy:

* `MustRunAs`: contains a list of ranges that define valid ranges for the `fsGroup` value. At least
  one range must contain the provided `.securityContext.fsGroup`. If the pod does not contain a
  `.securityContext`, or a `.securityContext.fsGroup`, then this policy acts as mutating and
  defaults the `fsGroup` attribute to the first `min` value of the list of provided ranges.
* `MayRunAs`: contains a list of ranges that define valid ranges for the `fsGroup` value. At least
  one range must contain the provided `.securityContext.fsGroup`. If the pod does not contain a
  `.securityContext` or a `.securityContext.fsGroup`, then this policy will accept the request.
* `RunAsAny`: always accepts the request.

Configuration examples:

```yaml
rule: RunAsAny
```

```yaml
rule: MayRunAs
ranges:
  - min: 1000
    max: 2000
  - min: 3000
    max: 4000
```

```yaml
rule: MustRunAs
ranges:
  - min: 1000 # If no fsGroup is set for the pod, the
              # policy will default it to this value
    max: 2000
  - min: 3000
    max: 4000
```
