[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Kubewarden policy psp-flexvolume-drivers

## Description

Replacement for the Kubernetes Pod Security Policy that controls Flex Volume drivers.

## Settings

This policy allows to provide a list of allowed Flex Volume drivers.

The configuration supports a list of allowed flex volume drivers. An example follows:

```yaml
allowedFlexVolumes:
  - driver: example/lvm
  - driver: example/cifs
```

If the pod to be evaluated has a different driver on any `flexVolume` volume, it will be rejected.
