[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

This policy validates the annotations of generic Kubernetes objects.

The policy rejects all the resources that use one or more annotations on the
deny list. The deny list is provided by at runtime via the policy configuration.

The policy allows users to put constraints on specific annotations. The constraints
are expressed as regular expression and are provided via the policy settings.

The policy settings look like that:

```yaml
# List of annotations that cannot be used
denied_annotations:
  - foo
  - bar

# List of annotations that must be defined
mandatory_annotations:
  - cost-center

# Annotations that are validate with user-defined RegExp
# Failing to comply with the RegExp resuls in the object
# being rejected
constrained_annotations:
  priority: "[123]"
  cost-center: "^cc-\\d+$"
```

> **Note well:** the regular expression must be expressed
> using [Go's syntax](https://golang.org/pkg/regexp/syntax/).

Given the configuration from above, the policy would reject the creation
of this Pod:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: nginx
  annotations:
    foo: hello world
spec:
  containers:
    - name: nginx
      image: nginx:latest
```

The policy would also reject the creation of this Ingress resource:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: minimal-ingress
  annotations:
    cost-center: cc-marketing
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
    - http:
        paths:
          - path: /testpath
            pathType: Prefix
            backend:
              service:
                name: test
                port:
                  number: 80
```

Policy's settings can also be used to force certain annotations to be specified,
regardless of their contents:

```yaml
# Policy's settings

constrained_annotations:
  mandatory-annotation: ".*" # <- this annotation must be present, we don't care about its value
```
