[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

This policy ensures Kubernetes workloads that make use of
[Runtime Enforcer](https://github.com/rancher-sandbox/runtime-enforcer)
to protect themselves are referring an existing `WorkloadPolicy`.

## Configuration

There is no configurable options at this time.

## Behavior

Kubernetes Pods can be protected by Runtime Enforcer by adding the
`security.rancher.io/policy` label to them. This policy ensures that
the `WorkloadPolicy` referenced by that label exists.
The policy can validate Pods and higher order Kubernetes workload types,
such as:

- CronJob
- DaemonSet
- Deployment
- Job
- ReplicaSet
- StatefulSet

## Deployment

This policy requires extra permission to read `WorkloadPolicy` resource
defined by Runtime Enforcer.


The Service Account used by the Policy Server hosting this policy
must be granted `GET`, `LIST` and `WATCH` privileges to the
`WorkloadPolicy` resource:

This can be achieved by creating the following RBAC resources:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: runtime-enforcer-workloadpolicy-viewer
rules:
  - apiGroups:
      - security.rancher.io
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

