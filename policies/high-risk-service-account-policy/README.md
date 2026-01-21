[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# High-Risk Service Account Blocker

> [!IMPORTANT]
> This policy needs Kubewarden `>= 1.27`, as it depends on the new `can_i` host capability.
>
> On older Kubewarden versions it fails-closed on execution, rejecting all
> listened resources under rules, and logging an error in the policy-server.

The policy can be configured to define a list of resources and operations that
are considered privileged (for example, `LIST, GET` Secret resources).

When a resource workload (a Pod, Deployment, CronJob,...) is created or
updated, the policy will use the [Kubernetes Authorization
API](https://kubernetes.io/docs/reference/access-authn-authz/authorization/#checking-api-access)
to assess whether the ServiceAccount used by the workload has the rights to
perform any of the sensitive operations defined by the user.

Workloads that use a high-privileged ServiceAccount are rejected.

The policy rejects a workload as soon as it found a blocked operation to reduce
the load on the Kubernetes API server.

## How it works

Every time a workload resource that uses a service account is submitted to the cluster,
the policy will query the Kubernetes authorization API to check if the given
ServiceAccount has some permissions that it shouldn't. To perform this
verification, the policy will create a
[SubjectAccessReview](https://kubernetes.io/docs/reference/kubernetes-api/authorization-resources/subject-access-review-v1/)
and apply it to check the service account permissions. If the
service account can perform such operation, the request is rejected.

When the policy builds the `SubjectAccessReview`, the user set in in the
resource is defined by `system:serviceaccount:<namespace>:<service-account>`.
Here, the `namespace` is the namespace from the request and the
`service-account` is the service account from the resource being deployed or
the `default` one.

## Access to Kubernetes resources

The PolicyServer where the this policy runs must have RBAC permissions to
create `authorization.k8s.io/v1/SubjectAccessReviews`. This is already the case
for the Kubewarden default PolicyServer.

To do this for other PolicyServers, set their `spec.serviceAccountName` to
the name of ServiceAccount that is bound to a ClusterRole with a rule:

```yaml
- apiGroup: "authorization.k8s.io"
  resources:
    - "subjectaccessreviews"
  verbs:
    - create
```

## Settings

The policy settings consist of a list of rules that service accounts are
prohibited from having defined in any of their associated roles and cluster
roles.

```yaml
blockRules:
  # For listing secrets
  - apiGroups: [""] # mandatory
    resources: ["secrets"] # mandatory
    verbs: ["list"] # mandatory
    namespace: "default" # optional
```

The `apiGroups` and the `resources` fields are used to identify the Kubernetes
resources to protect. The verbs list the operations that have to be prevented.

The verbs field allow the following values: `create`, `update`, `delete`,
`get`, `list`, `watch`, `proxy`, `*`, `patch` and `deletecollection`.

### `namespace` Field Usage Guide

The `namespace` attribute is optional. When specified, it defines the Namespace
inside of which the operation has to be blocked. When not provided, the policy
assumes the operation should be blocked at the cluster level.

When protecting cluster-wide resources (e.g. `Node`, `ClusterRole`,
`PersistentVolume`, `Namespace`), the namespace field must be omitted.

When protecting a namespaced resource (e.g. `Secret`), you have to consider
whether you want to prevent operation from being done inside of all the
namespaces of the cluster or just some of them.

> [!IMPORTANT]
> Namespaced rules **never** implies cluster-wide access.
> Rules without `namespace` checks **exclusively** for cluster-scoped permissions.
>
> Namespace-bound access (granted via Role + RoleBinding) and cluster-wide access
> (via ClusterRole + ClusterRoleBinding) are mutually exclusive in authorization
> checks. A service account with access to `Secrets` in its own namespace will
> fail (`allowed: false`, which in this policy context means accepting the
> resource in the cluster) a cluster-wide `SubjectAccessReview` check (e.g.,
> listing Secrets across all namespaces). This occurs because Kubernetes enforces
> strict scope isolation: permissions granted within a namespace never implicitly
> extend to other namespaces or cluster-wide operations. `SubjectAccessReview`
> precisely reflects this design. A cluster-wide check verifies global access
> rights, while a namespace-scoped check validates local permissions.

## Example

Let's take a look in a example

```yaml
blockRules:
  # For listing secrets
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["list"]

  # For executing commands in containers
  - apiGroups: [""]
    resources: ["pods/exec"]
    verbs: ["create"]

  # Full access to all workload resources
  - apiGroups: [""]
    resources: ["pods", "pods/log"]
    verbs: ["*"]
  - apiGroups: ["apps"]
    resources: ["deployments", "statefulsets", "daemonsets", "replicasets"]
    verbs: ["*"]
  - apiGroups: ["batch"]
    resources: ["jobs", "cronjobs"]
    verbs: ["*"]
  - apiGroups: ["autoscaling"]
    resources: ["horizontalpodautoscalers"]
    verbs: ["*"]

  # Full access to RBAC resources within a namespace
  - apiGroups: ["rbac.authorization.k8s.io"]
    resources: ["roles", "rolebindings"]
    verbs: ["*"]
    namespace: "default"
```

For example, if the policy is deployed with the previous configuration in a
cluster that has a service account like this:

```yaml
---
# ServiceAccount
apiVersion: v1
kind: ServiceAccount
metadata:
  name: super-admin-sa
  namespace: default

---
# Powerful Role with all requested permissions
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: super-admin-role
  namespace: default
rules:
  # For listing secrets
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["list"]

  # For executing commands in containers
  - apiGroups: [""]
    resources: ["pods/exec"]
    verbs: ["create"]

  # Full access to all workload resources
  - apiGroups: [""]
    resources: ["pods", "pods/log"]
    verbs: ["*"]
  - apiGroups: ["apps"]
    resources: ["deployments", "statefulsets", "daemonsets", "replicasets"]
    verbs: ["*"]
  - apiGroups: ["batch"]
    resources: ["jobs", "cronjobs"]
    verbs: ["*"]
  - apiGroups: ["autoscaling"]
    resources: ["horizontalpodautoscalers"]
    verbs: ["*"]
  # Full access to RBAC resources within namespace
  - apiGroups: ["rbac.authorization.k8s.io"]
    resources: ["roles", "rolebindings"]
    verbs: ["*"]

---
# RoleBinding for namespace-scoped permissions
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: super-admin-rolebinding
  namespace: default
subjects:
  - kind: ServiceAccount
    name: super-admin-sa
    namespace: default
roleRef:
  kind: ClusterRole
  name: super-admin-role
  apiGroup: rbac.authorization.k8s.io
```

The following deployment would not be allowed in the cluster:

```console
# Deployment using the powerful ServiceAccount
kubectl create -f - << EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: super-admin-deployment
  namespace: default
spec:
  replicas: 1
  selector:
    matchLabels:
      app: super-admin-app
  template:
    metadata:
      labels:
        app: super-admin-app
    spec:
      serviceAccountName: super-admin-sa
      containers:
        - name: main
          image: nginx:alpine
          ports:
            - containerPort: 80
        - name: kubectl
          image: bitnami/kubectl:latest
          command: ["sleep", "infinity"]
EOF
Error from server: error when creating "STDIN": admission webhook "clusterwide-high-risk-service-account.kubewarden.admission" denied the request: Cannot use service account 'system:serviceaccount:default:super-admin-sa' with permissions to perform list /secrets in the cluster
```

In the previous example, the user set in the `SubjectAccessReview` resource
would be `system:serviceaccount:default:super-admin-sa`.

As mentioned in the settings section, it's important to pay attention to if
`blockRules` are performing namespaced or cluster wide checks. For example, in
the previous example, if the `super-admin-deployment` has the same permissions
defined in a `Role` resource instead of `ClusterRole`, the deployment will be
applied in the cluster.
