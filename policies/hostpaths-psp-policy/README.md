[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# psp-hostpaths-policy

Replacement for the Kubernetes Pod Security Policy that controls the usage of
`hostPath` volumes. The policy inspects both the containers and the init
containers that are using `hostPath` volumes.

## Settings

```yaml
allowedHostPaths:
- pathPrefix: "/foo"
  readOnly: true
- pathPrefix: "/bar"
  readOnly: false
```

`allowedHostPaths` is a list of host paths that are allowed to be used by
`hostPath` volumes.

An empty `allowedHostPaths` list means there is no restriction on host paths
used.

Each entry of `allowedHostPaths` must have:
- A `pathPrefix` field, which allows `hostPath` volumes to mount a path that
  begins with an allowed prefix.
- a `readOnly` field indicating it must be mounted read-only.

### Special behaviour

It's possible to have host paths sharing part of the prefix. In that case, the
`readOnly` attribute of the most specific path takes precedence.

For example, given the following configuration:

```yaml
allowedHostPaths:
- pathPrefix: "/foo"
  readOnly: false
- pathPrefix: "/foo/bar"
  readOnly: true
```

Paths such as `/foo/bar/dir1`, `/foo/bar` must be read only.
