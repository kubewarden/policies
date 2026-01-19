# priority-class-policy

Kubernetes allows users to define `PriorityClasses` for their pods. This helps
the control plane properly schedule workloads based on their priorities and
determine which pods should be preempted to ensure critical workloads are
always scheduled. However, users can misuse the available `PriorityClasses` by
setting improper classes.

To avoid this kind of situation, this policy ensures that the
`priorityClassName` defined in a pod is only one of the expected ones.
Therefore, cluster administrators can use this policy to enforce a set of
allowed `PriorityClass` names, rejecting deployments if users try to create a
Pod with an unexpected `priorityClassName`.

This policy can inspect `Pod` resources, but can also operate against
"higher-order" Kubernetes resources like `Deployment`, `ReplicaSet`,
`DaemonSet`, `ReplicationController`, `Job`, and `CronJob`.

## Settings

The policy settings accept a list of the `PriorityClass` names that would match
the Pod's `priorityClassName` field, under `allowed_priority_classes` for an
allow list, or `denied_priority_classes` for a deny list.

The user **must** provide a list of priority classes names when deploying the
policy. Both `allowed_priority_classes` and `denied_priority_classes` cannot
be used at the same time.

If the `PodSpec` of the resource does not have the `priorityClassName` field
defined, it's possible to configure the policy to mutate the resource adding a
default value. This is possible by defining the `default_priority_class`
settings. This field is optional. But when defined, it should be listed in the
`allowed_priority_classes` and not present in the `denied_priority_classes`.
When the resource has a `priorityClassName` defined, no mutation is performed.
And the validations against the settings provided are done as usual.

> [!WARNING]
> It's important that the `PriorityClass` name defined in the
> `default_priority_class` setting actually exists in the cluster. Otherwise,
> the policy will mutate resources to use a non-existent class, which will
> cause Kubernetes to reject them.

### Allow list

Here's an example where any `Pod` that defines a `priorityClassName` that
is not present in the list will be rejected. If the `priorityClassName` is not
defined, the resource will be accepted:

```yaml
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  annotations:
    io.kubewarden.policy.category: Resource validation
    io.kubewarden.policy.severity: medium
  name: priority-class-policy
spec:
  module: ghcr.io/kubewarden/policies/priority-class-policy:latest
  rules:
    - apiGroups:
        - ""
      apiVersions:
        - v1
      resources:
        - pods
      operations:
        - CREATE
        - UPDATE
  mutating: false
  settings:
    allowed_priority_classes:
      - low-priority
      - med-priority
      - high-priority
    default_priority_class: "med-priority"
```

### Deny list

Here's an example where any `Pod` that defines a `priorityClassName` that
is present in the list will be rejected. If the `priorityClassName` is not
defined, the resource will be accepted:

```yaml
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  annotations:
    io.kubewarden.policy.category: Resource validation
    io.kubewarden.policy.severity: medium
  name: priority-class-policy
spec:
  module: ghcr.io/kubewarden/policies/priority-class-policy:latest
  rules:
    - apiGroups:
        - ""
      apiVersions:
        - v1
      resources:
        - pods
      operations:
        - CREATE
        - UPDATE
  mutating: false
  settings:
    denied_priority_classes:
      - low-priority
      - med-priority
      - high-priority
    default_priority_class: "other-priority"
```

### Using namespaceSelector

Furthermore, the cluster operator can use this policy to enforce specific
classes based on namespace or object selectors. For example, to apply the
policy to specific namespaces, the `namespaceSelector` can be used.

In this example, the same `priorityClassName` configuration validation
will be applied to `Pods` only deployed in the `team1` and `team2` namespaces:

```yaml
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  annotations:
    io.kubewarden.policy.category: Resource validation
    io.kubewarden.policy.severity: medium
  name: priority-class-policy
spec:
  module: ghcr.io/kubewarden/policies/priority-class-policy:latest
  rules:
    - apiGroups:
        - ""
      apiVersions:
        - v1
      resources:
        - pods
      operations:
        - CREATE
        - UPDATE
  mutating: false
  namespaceSelector:
    matchExpressions:
      - key: "kubernetes.io/metadata.name"
        operator: In
        values: [team1, team2]
  settings:
    allowed_priority_classes:
      - low-priority
      - med-priority
      - high-priority
```
