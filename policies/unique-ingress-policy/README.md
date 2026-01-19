[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Unique ingress host

This policy prevents the creation of Ingress resources that have host rules conflicting with the Ingress objects already defined inside of the cluster.

**Note:** this policy does not handle hostname wildcards.

## Access to Kubernetes resources

This policy requires access to `networking.k8s.io/Ingress` objects.
Access has to be granted at deployment time by setting the `contextAwareResources`
attribute of the `ClusterAdmissionPolicy`.

Note: context aware policies cannot be deployed using the `AdmissionPolicy`
custom resource.

Refer to the [context aware documentation](https://docs.kubewarden.io/explanations/context-aware-policies)
for more details.

## Settings

This policy does not take any configuration value.

This is a Gatekeeper policy that prevents the creation of Ingress resources
with duplicated hosts.

## Example

Assume the following Ingress object already exists inside of the cluster:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: already-exists
spec:
  rules:
    - host: "example.com"
      http:
        paths:
          - pathType: Prefix
            path: "/foo"
            backend:
              service:
                name: service1
                port:
                  number: 80
```

The policy would allow the creation of this new Ingress object:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: new-one
spec:
  rules:
    - host: "esempio.it"
      http:
        paths:
          - pathType: Prefix
            path: "/foo"
            backend:
              service:
                name: service2
                port:
                  number: 80
```

While it would deny the creation of this one:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: not-valid
spec:
  rules:
    - host: "example.com"
      http:
        paths:
          - pathType: Prefix
            path: "/foo"
            backend:
              service:
                name: service3
                port:
                  number: 80
```

The latter object is not allowed because its `host` rule overlaps with the
one of the `already-exists` Ingress.

## Implementation details

The policy is a 1:1 copy of [this one](https://open-policy-agent.github.io/gatekeeper-library/website/validation/uniqueingresshost/),
it's meant to show how Kubewarden supports Gatekeeper policies that make use of
context aware data (also called
["replicating data"](https://open-policy-agent.github.io/gatekeeper/website/docs/sync/)
by Gatekeeper).
