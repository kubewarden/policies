[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

This policy provides a replacement for the Pod Security Policy that deals with
seccomp profiles.

Prior to Kubernetes 1.19, seccomp profiles could be defined only via Pod
`annotations`. Starting from Kubernetes 1.19 the seccomp profiles can be managed
via the `securityContext` field of Pods and Containers.

Note well: the seccomp annotations are deprecated and will be dropped starting
from Kubernetes 1.25.

## Settings

This policy has some configurations:
- `allowed_profiles`: Define the accecpted profile values for the annotations
`container.seccomp.security.alpha.kubernetes.io/<container>` and
`seccomp.security.alpha.kubernetes.io/pod`.
- `profile_types`: Define the allowed values to be set in the seccomp type
in the security context of a container or of the Pod.
- `localhost_profiles`: Define the allowed localhost profiles. This is used only
when the "Localhost" type is allowed inside of the security context.

This policy can handle both seccomp policies expressed via `annotations` and
via `securityContext`. In later Kubernetes version, it will populate the
`securityContext` when the user define only the annotations. For this reason,
if the user does not define the `profile_types` and `localhost_profiles` settings,
the policy will fall-back to the `allowed_profiles`. Validating the containers
`securityContext` against it. The following two settings are equivalent:

```yaml
apiVersion: policies.kubewarden.io/v1alpha2
kind: ClusterAdmissionPolicy
metadata:
  name: psp-seccomp
spec:
  policyServer: default
  module: registry://ghcr.io/kubewarden/policies/seccomp-psp:v0.1.0
  rules:
  - apiGroups: [""]
    apiVersions: ["v1"]
    resources: ["pods"]
    operations:
    - CREATE
    - UPDATE
  mutating: false
  settings:
    allowed_profiles:
      - runtime/default
      - docker/default
      - localhost/test
    profile_types:
      - RuntimeDefault
      - Localhost
    localhost_profiles:
      - test
```


```yaml
apiVersion: policies.kubewarden.io/v1alpha2
kind: ClusterAdmissionPolicy
metadata:
  name: psp-seccomp
spec:
  policyServer: default
  module: registry://ghcr.io/kubewarden/policies/seccomp-psp:v0.1.0
  rules:
  - apiGroups: [""]
    apiVersions: ["v1"]
    resources: ["pods"]
    operations:
    - CREATE
    - UPDATE
  mutating: false
  settings:
    allowed_profiles:
      - runtime/default
      - docker/default
      - localhost/test
```

As said before, you do not need to declare the `profile_types` and
`localhost_profiles`. If you decided to use only the `allowed_profiles`
settings you can get the same results.


## Examples

With the yaml settings described in the settings section the following pods will
be accepted:


```yaml
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-allowed
  annotations:
    container.seccomp.security.alpha.kubernetes.io/nginx: runtime/default
  labels:
    app: nginx-seccomp
spec:
  containers:
  - name: nginx
    image: nginx
----
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-allowed2
  labels:
    app: nginx-seccomp
spec:
  securityContext:
    seccompProfile:
      type: RuntimeDefault
  containers:
  - name: nginx
    image: nginx
----
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-allowed3
  labels:
    app: nginx-seccomp
spec:
  securityContext:
    seccompProfile:
      type: Localhost
      localhostProfile: test
  containers:
  - name: nginx
    image: nginx
----
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-allowed4
  annotations:
    container.seccomp.security.alpha.kubernetes.io/nginx: localhost/test
  labels:
    app: nginx-seccomp
spec:
  containers:
  - name: nginx
    image: nginx
----
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-allowed5
  annotations:
    container.seccomp.security.alpha.kubernetes.io/nginx: docker/default
  labels:
    app: nginx-seccomp
spec:
  containers:
  - name: nginx
    image: nginx
```

While the following pods will be rejected:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-disallowed
  labels:
    app: nginx-seccomp
spec:
  securityContext:
    seccompProfile:
      type: Unconfined
  containers:
  - name: nginx
    image: nginx
  - name: nginx2
    image: nginx
----
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-disallowed2
  annotations:
    container.seccomp.security.alpha.kubernetes.io/nginx: localhost/test2
  labels:
    app: nginx-seccomp
spec:
  containers:
  - name: nginx
    image: nginx
  - name: nginx2
    image: nginx
----
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-disallowed3
  annotations:
    seccomp.security.alpha.kubernetes.io/pod: unconfined
  labels:
    app: nginx-seccomp
spec:
  containers:
  - name: nginx
    image: nginx
----
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-disallowed4
  annotations:
    container.seccomp.security.alpha.kubernetes.io/nginx: unconfined
  labels:
    app: nginx-seccomp
spec:
  containers:
  - name: nginx
    image: nginx
  - name: nginx2
    image: nginx
----
apiVersion: v1
kind: Pod
metadata:
  name: nginx-seccomp-disallowed5
  labels:
    app: nginx-seccomp
spec:
  securityContext:
    seccompProfile:
      type: Localhost
      localhostProfile: test2
  containers:
  - name: nginx
    image: nginx
  - name: nginx2
    image: nginx
```
