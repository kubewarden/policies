[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Kubewarden policy psp-selinux

## Description

Replacement for the Kubernetes Pod Security Policy that controls the usage of SELinux in the pod
security context and on containers, init containers and ephemeral containers. This policy will
inspect the `.spec.securityContext.seLinuxOptions` of the pod  if the container has no specific
`.spec.securityContext.seLinuxOptions`. In other words, the `seLinuxOptions` of the container, init
container and ephemeral containers take precendence over the pod `seLinuxOptions`, if any.

## Settings

This policy works by defining what `seLinuxOptions` can be set at the pod level and at the container
level.

One of the following setting keys are accepted for this policy:

* `MustRunAs`: contains the desired value for the `seLinuxOptions` parameter. If the pod does not
  contain a `.securityContext`, or a `.securityContext.seLinuxOptions`, then this policy acts as
  mutating and defaults the `seLinuxOptions` attribute to the one provided in the configuration. In
  all cases, pod containers, init container and ephemeral containers `.seLinuxOptions` are checked
  for compatibility if they override the Pod Security Context `seLinuxOptions` value.
* `RunAsAny`: always accepts the request.

Configuration examples:

```yaml
rule: RunAsAny
```

```yaml
rule: MustRunAs
user: user
role: role
type: type
level: s0:c0,c6
```
