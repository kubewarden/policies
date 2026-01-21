> **Note:** this is a context aware policy that requires Kubewarden 1.6

Rancher Manager has the concept of Project. A Project can hold many
regular Kubernetes Namespaces.

This policy monitors the creation and update of of Namespace objects and,
when they belong to a Project, it ensures a list of labels defined on the
Project are propagated to the Namespace.

The labels defined on the Project have precedence over the ones defined
inside of the Namespace.

On the Project, only the labels that start with the `propagate.` prefix
are propagated to its Namespaces. The `propagate.` prefix is stripped when
the copy operation is performed.

Namespaces that do not belong to a Rancher Project are ignored by this policy.

## Cluster access

The policy requires access to the `management.cattle.io/projects`
custom resources.

The policy requires only `GET` access to Projects defined inside of the
`local` Namespace.

Given the following assumptions:

* Kubewarden is deployed inside of the `kubewarden` Namespace
* The policy is hosted by a PolicyServer that uses the `policy-server` ServiceAccount

The following snippet will allow the policy to operate:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: rancher-project-reader
  namespace: local
rules:
- apiGroups: ["management.cattle.io"]
  resources: ["projects"]
  verbs: ["get"]
---
# Allows the policy-server ServiceAccount defined inside of the
# kubewarden namespace to read Project resources defined inside
# of the `local` namespace.
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: read-rancher-projects-local
  namespace: local
subjects:
- kind: ServiceAccount
  name: policy-server
  namespace: kubewarden
roleRef:
  kind: Role
  name: rancher-project-reader
  apiGroup: rbac.authorization.k8s.io
```

## Examples

Given a Project that defines the following labels:

* `propagate.kubewarden-profile` with value `strict`
* `cost-center` with value `123`

The creation of a Namespace without labels would be changed to ensure
these labels are defined:

* `kubewarden-profile` with value `strict`

The creation of a Namespace that has the following labels:

* `kubewarden-profile` with value `low`
* `team` with value `hacking`

Would be changed by the policy to ensure these labels are defined:

* `kubewarden-profile` with value `strict`
* `team` with value `hacking`

### Use case

Let's assume we want to replace the now removed Kubernetes
Pod Security Policies with a list of Kubewarden policies.

We are going to define different levels of security compliance for our Namespaces,
then we will use Kubewarden policies to enforce them.

For example, let's assume we want to have this security levels:

| Security level | Policies |
|----------------|----------|
| moderate       | do not allow privilege escalation, do not allow privileged pods |
| strict | do not allow privilege escalation, do not allow privileged pods, restrict fsgroups |

We start by defining the Kubewarden policies that are required
by the `strict` and `moderate` profiles:

```yaml
# note: these policies can target also high order resources
# that consume Pods. These definitions are kept short
# on purpose.
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  name: do-not-allow-privilege-escalation-psp
spec:
  module: registry://ghcr.io/kubewarden/policies/allow-privilege-escalation-psp:v0.2.5
  settings:
    default_allow_privilege_escalation: false
  rules:
  - apiGroups:
    - ''
    apiVersions:
    - v1
    resources:
    - pods
    operations:
    - CREATE
  mutating: true
  namespaceSelector:
    matchExpressions:
    - key: "security-posture"
      operator: "In"
      values: [ "strict", "moderate" ]
---
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  name: pod-privileged-policy
spec:
  module: registry://ghcr.io/kubewarden/policies/pod-privileged:v0.2.5
  settings: {}
  rules:
  - apiGroups:
    - ''
    apiVersions:
    - v1
    resources:
    - pods
    operations:
    - CREATE
  mutating: false
  namespaceSelector:
    matchExpressions:
    - key: "security-posture"
      operator: "In"
      values: [ "strict", "moderate" ]
```

Then we define the policy that belongs only to the
`strict` profile:

```yaml
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  name: allowed-fsgroups-psp
spec:
  module: registry://ghcr.io/kubewarden/policies/allowed-fsgroups-psp:v0.1.9
  settings:
    rule: MustRunAs
    ranges:
    - min: 1000
      max: 2000
  rules:
  - apiGroups:
    - ''
    apiVersions:
    - v1
    resources:
    - pods
    operations:
    - CREATE
    - UPDATE
  mutating: true
  namespaceSelector:
    matchExpressions:
    - key: "security-posture"
      operator: "In"
      values: [ "strict" ]
```

Finally, we deploy the `rancher-project-propagate-labels` policy:

```yaml
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  name: rancher-project-propagate-labels
spec:
  module: registry://ghcr.io/kubewarden/policies/rancher-project-propagate-labels:latest
  settings: {}
  rules:
  - apiGroups:
    - ''
    apiVersions:
    - v1
    resources:
    - namespaces
    operations:
    - CREATE
    - UPDATE
  mutating: true
  contextAwareResources:
  - apiVersion: management.cattle.io/v3
    kind: Project
```

Now, we can define a new Rancher project that has the following label:

* `propagate.security-posture`: `moderate`

All the Namespaces created under this project will have the label
`{"security-posture": "moderate"}` set. Hence the
`do-not-allow-privilege-escalation-psp` and the
`pod-privileged-policy` policies will be enforced inside of it.


## Limitations

### Rancher multi-cluster support

Currently the policy works only when deployed inside of the `local` Rancher cluster.
That's because the Project resources are defined only inside of the cluster where
Rancher Manager is running.

When deployed on a downstream cluster, the policy isn't currently capable of
querying the upstream Project that is being referenced by the Namespace.

This limitation is going to be addressed by future releases.

### Changes to parent Project

A change done to the parent project (adding/removing/updating a label) will not
result in an update to all the namespaces that are associated with it.

The label propagation is done when the Namespace is created or updated.

## Settings

This policy has one configuration value `downstream_cluster_failure_mode`. This
defines what the policy should do when it's being deployed into a downstream
cluster.

As explained before, the policy cannot evaluate Namespace defined inside of a
downstream cluster. This configuration has two possible values:

* `ignore`: accept the Namespace CREATE/UPDATE event. This is the default value
* `fail`: reject the Namespace CREATE/UPDATE event

For example, given the following configuration:

```yaml
downstream_cluster_failure_mode: fail
```

The policy would reject the creation and update of Namespace when its being
deployed inside of a downstream cluster.

On the other hand, given this configuration:

```yaml
downstream_cluster_failure_mode: ignore
```

The creation/update of Namespace resources would always be allowed inside of
downstream clusters.
