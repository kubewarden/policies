[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Restrict StorageClasses in PersistentVolumeClaims

This Kubewarden policy is designed to enhance the security and manageability of
Kubernetes clusters by preventing the use of certain storage classes within
`PersistentVolumeClaim` (PVC) objects. The policy provides an option to
configure a fallback storage class, offering a seamless alternative when a
denied storage class is requested.

## Configuration

You can configure the policy's behavior using the parameters below:

```yaml
# A list of storage classes that are forbidden.
# This setting is mutually exclusive with allowedStorageClasses.
deniedStorageClasses:
  - fast
  - nvme

# A list of storage classes that are permitted.
# This setting is mutually exclusive with deniedStorageClasses.
allowedStorageClasses:
  - standard
  - slow

# Optional: Specifies a fallback storage class to use when the
# requested storage class is not allowed.
fallbackStorageClass: standard
```

You can configure the policy using either a denylist (`deniedStorageClasses`)
or an allowlist (`allowedStorageClasses`), but not both, as these settings are
mutually exclusive.

- `deniedStorageClasses`: Any `PersistentVolumeClaim` requesting a storage
  class on this list will be considered invalid.
- `allowedStorageClasses`: Any `PersistentVolumeClaim` requesting a storage
  class not on this list will be considered invalid.

The `fallbackStorageClass` is an optional parameter that specifies a storage
class to apply if the one requested is invalid. The value of the fallback must
be a valid storage class itself:

- If `deniedStorageClasses` is configured, the `fallbackStorageClass` must not be
  one of the denied classes.
- If `allowedStorageClasses` is configured, the `fallbackStorageClass` must be one
  of the allowed classes.

If a request uses an invalid storage class and `fallbackStorageClass` is not
defined, the policy will reject the resource.

## How It Works

The policy inspects the `spec.storageClassName` field of PVC under evaluation.
The policy's action depends on whether you are using an allowlist or a
denylist.

**Using `deniedStorageClasses`**

If a PVC requests a storage class that is on the `deniedStorageClasses` list:

- Without a `fallbackStorageClass`, the request is rejected.
- With a `fallbackStorageClass`, the PVC is mutated to use the fallback storage
  class, and the request is accepted.

**Using `allowedStorageClasses`**

If a PVC requests a storage class that is not on the `allowedStorageClasses`
list:

- Without a `fallbackStorageClass`, the request is rejected.
- With a `fallbackStorageClass`, the PVC is mutated to use the fallback storage
  class, and the request is accepted.

## Examples

### Example 1: Rejecting a Denied Storage Class

Given the configuration:

```yaml
deniedStorageClasses:
  - fast
```

A PVC with `storageClassName: fast` will be rejected:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: task-pv-claim
spec:
  storageClassName: fast
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
```

### Example 2: Accepting and Mutating to Fallback Storage Class

With the following configuration:

```yaml
deniedStorageClasses:
  - fast
fallbackStorageClass: cheap
```

A PVC requesting a denied storage class will be mutated to use the fallback class `cheap`, thus being accepted:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: task-pv-claim
spec:
  storageClassName: fast
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
```

Will be mutated to:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: task-pv-claim
spec:
  storageClassName: cheap
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
```

### Example 3: Rejecting a Disallowed Storage Class

Given the following configuration that only allows the `standard` storage class

```yaml
allowedStorageClasses:
  - standard
```

A PVC requesting the disallowed `fast` storage class will be rejected:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: task-pv-claim
spec:
  storageClassName: fast
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
```

### Example 4: Mutating a Disallowed Storage Class to the Fallback

With the following configuration:

```yaml
allowedStorageClasses:
  - standard
fallbackStorageClass: standard
```

A PVC requesting a disallowed storage class (`fast`) will be mutated to use the `fallbackStorageClass` (`standard`) and will be accepted:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: task-pv-claim
spec:
  storageClassName: fast
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
```

Will be mutated to:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: task-pv-claim
spec:
  storageClassName: standard
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
```
