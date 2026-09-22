[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

This policy validates Kubernetes workloads that are protected by
[Runtime Enforcer](https://github.com/kubewarden/runtime-enforcer).

Runtime Enforcer protects a Pod when the Pod has the
`runtimeenforcer.kubewarden.io/policy` label. The label value is the name
of a `WorkloadPolicy` (the Runtime Enforcer resource that holds the rules).
The `WorkloadPolicy` must exist in the same namespace as the workload.

This policy rejects a workload when the label points to a `WorkloadPolicy`
that does not exist. With the `requireProtection` setting, this policy also
rejects workloads that do not have the label.

## Configuration

```yaml
# Reject workloads that do not have the
# `runtimeenforcer.kubewarden.io/policy` label.
# Default: false
requireProtection: false
```

- `requireProtection` (boolean, default `false`): when `true`, this policy
  rejects workloads whose Pod template does not have the
  `runtimeenforcer.kubewarden.io/policy` label. When `false`, this policy
  accepts those workloads and only validates the workloads that have the
  label.

When `requireProtection` is `true`, this policy also rejects system
workloads that do not have the label. Examples are the workloads in
`kube-system`, in `kubewarden`, and in the namespace of Runtime Enforcer.
Use the `namespaceSelector` of the `ClusterAdmissionPolicy` to exclude
these namespaces.

## Behavior

This policy inspects the Pod template of the workload and reads the
`runtimeenforcer.kubewarden.io/policy` label:

- If the label is missing and `requireProtection` is `false`, the policy accepts the workload.
- If the label is missing and `requireProtection` is `true`, the policy rejects the workload.
- If the label is present and the `WorkloadPolicy` exists in the same namespace, the policy accepts the workload.
- If the label is present and the `WorkloadPolicy` does not exist, the policy rejects the workload.

This policy validates the following resource types:

- CronJob
- DaemonSet
- Deployment
- Job
- Pod
- ReplicaSet
- StatefulSet

## Deployment

This policy reads `WorkloadPolicy` resources from the cluster. The Service
Account of the Policy Server that hosts this policy must have the `get`,
`list`, and `watch` permissions on the `workloadpolicies` resource. If the
permissions are missing, this policy rejects every labeled workload with an
error.

Create the following RBAC resources to grant the permissions:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: runtime-enforcer-workloadpolicy-viewer
rules:
  - apiGroups:
      - runtimeenforcer.kubewarden.io
    resources:
      - workloadpolicies
    verbs:
      - get
      - list
      - watch
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: runtime-enforcer-workloadpolicy-viewer-policy-server
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: runtime-enforcer-workloadpolicy-viewer
subjects:
  - kind: ServiceAccount
    name: policy-server
    namespace: kubewarden
```
