[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

Kubewarden policy that enforces the DNS lookup configuration of a Pod to have a specific `ndots` value.

This is done by mutating the Pod's `.spec.dnsConfig.options` field to have the desired `ndots` value.

# Configuration

The number of `ndots` to enforce can be configured using the `ndots` field.

```yaml
ndots: 2
```

When no configuration is provided, the default value is `1`.

## Examples

Assuming the no configuration is provided, the policy will enforce the `ndots` value to be `1`.

This will change the following Pod:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: nginx
spec:
  containers:
    - name: nginx
      image: nginx
```

To the following Pod:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: nginx
spec:
  dnsConfig:
    options:
      - name: ndots
        value: "1"
  containers:
    - name: nginx
      image: nginx
```
