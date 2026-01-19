[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)


# unique-service-selector-policy

This policy requires Services to have unique selectors within a namespace. Selectors are considered the same if they have identical keys and values.
Selectors may share a key/value pair so long as there is at least one distinct key/value pair between them.

## Settings

This policy has no configurable settings. 

## Example

Assuming the following service is already defined:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: my-service
  namespace: default
spec:
  selector:
    app.kubernetes.io/name: MyApp
  ports:
    - protocol: TCP
      port: 80
      targetPort: 9376
```

The creation of the following service is **not** going to be allowed:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: another-service
  namespace: default
spec:
  selector:
    app.kubernetes.io/name: MyApp
  ports:
    - protocol: TCP
      port: 80
      targetPort: 9376
```

The creation of this service is going to be allowed:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: another-service
  namespace: default
spec:
  selector:
    app.kubernetes.io/name: MyApp
    env: production
  ports:
    - protocol: TCP
      port: 80
      targetPort: 9376
```

This is allowed because the service defines a new selector `env` that is not
defined by the already existing service.

The creation of the following service is going to be allowed:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: another-service
  namespace: default
spec:
  selector:
    app.kubernetes.io/name: AnotherApp
  ports:
    - protocol: TCP
      port: 80
      targetPort: 9376
```

This is allowed because the value of the `app.kubernetes.io/name` key is different.

The creation of the following service is going to be allowed:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: another-service
  namespace: staging
spec:
  selector:
    app.kubernetes.io/name: MyApp
  ports:
    - protocol: TCP
      port: 80
      targetPort: 9376
```

This is allowed despite the selector being equal to the one of the already
existing service. This is accepted because the new service is declared in a
completely different namespace (`staging` instead of `default`).

