[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# psp-volumes-policy

Replacement for the Kubernetes Pod Security Policy that controls the usage of
volumes in pods.

## Settings

The policy takes the list of the allowed volume types using the `allowedTypes`
setting. Example:

```yaml
allowedTypes:
- configMap
- downwardAPI
- emptyDir
- persistentVolumeClaim
- secret
- projected
```

The default value of allowedTypes is `[ ]`. The special value `*` can be used
to allow all kind of volumes.

No other value can be specified together with `*`. For example,
`allowedTypes: ['*', 'configMap']` is not a valid configuration setting.

The policy also takes an optional `ignoreInitContainersVolumes` setting. This setting defaults to `false`.
When set to `true`, volumes that are exclusively used by `initContainers` (and not by regular `containers`) are ignored during the validation process.

