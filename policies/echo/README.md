[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Kubewarden policy echo

> **Warning: this policy is useful only to Policy Authors**

The purpose of this policy is to allow Policy Authors to retrieve samples
of Kubernetes' [`AdmissionReview`](https://kubernetes.io/docs/reference/access-authn-authz/extensible-admission-controllers/#request)
objects. This information is useful to understand what a policy
validation (or mutation) logic should expect as input.

The policy can be used against any kind of Kubernetes event and resource.

### Behavior

The policy automatically accepts all the incoming requests, except the ones
that have the annotation `io.kubewarden.policy.echo.<operation>` specified. The value of the
annotation is not relevant.

All the requests with this special annotation will be rejected. The rejection message
will contain the JSON representation of the the `AdmissionReview`.

These are the annotations being watched:

* `io.kubewarden.policy.echo.create`: rejects creation of objects with this annotations
* `io.kubewarden.policy.echo.updates`: rejects changes to objects with this annotations
* `io.kubewarden.policy.echo.delete`: rejects deletions of objects with this annotations
* `io.kubewarden.policy.echo.connect`: rejects CONNECT operations to objects with this annotations

We will cover these in depth inside of the *"examples"* section.

## Settings

The policy has no settings. It's behavior is driven by special annotations added
to the objects being evaluated.

## Examples

First of all lets deploy the policy. We will enforce it only inside of
the `default` Namespace:

```console
kubectl apply -f - <<EOF
apiVersion: policies.kubewarden.io/v1
kind: AdmissionPolicy
metadata:
  name: echo
  namespace: default
spec:
  module: "registry://ghcr.io/kubewarden/policies/echo:latest"
  settings: {}
  rules:
    - apiGroups:
        - "*"
      apiVersions:
        - "*"
      resources:
        - "*"
      operations:
        - CREATE
        - UPDATE
        - DELETE
        - CONNECT
  mutating: false
EOF
```

### Inspect `CREATE` events

Let's obtain the `AdmissionReview` that is generated when a new Deployment
object is created.

To do that, let's create a regular Deployment object making sure the
"io.kubewarden.policy.echo.create" annotation is present.

We start by creating a file called `nginx.yaml` with the following contents:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx
  namespace: default
  annotations:
    io.kubewarden.policy.echo.create: "true"
spec:
  selector:
    matchLabels:
      app: nginx
  replicas: 0
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:latest
        ports:
        - containerPort: 80
```

Attempting to create this object will fail because the `echo` policy rejects
the operation.

We can get the raw AdmissionReview object by using the following command:

```console
kubectl apply -f nginx.yaml 2>&1 | sed -e 's/Error from.*request: //' | jq
```

### Inspect `UPDATE` events

Let's obtain the `AdmissionReview` that is generated when an existing
Deployment object is changed.

To do that, we will start by creating a regular Deployment object that
has the "io.kubewarden.policy.echo.update" annotation set:

```console
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx
  namespace: default
  annotations:
    io.kubewarden.policy.echo.update: "true"
spec:
  selector:
    matchLabels:
      app: nginx
  replicas: 0
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:latest
        ports:
        - containerPort: 80
EOF
```

Let's trigger an `UPDATE` operation. We will do that by attempting to add a
new annotation to the Deployment object:

```console
kubectl patch deployment -n default nginx --patch '{"metadata": {"annotations": {"hello": "world" } } }'
```

The `echo` policy will reject this change. We can get the raw AdmissionReview
object by using the following command:

```console
kubectl patch deployment -n default nginx --patch '{"metadata": {"annotations": {"hello": "world" } } }' 2>&1 | sed -e 's/Error from.*request: //' | jq
```

> **Note:** don't forget to remove the Deployment object before moving to the next
> section:
>
> `kubectl delete -n default deployment nginx`

### Inspect `DELETE` events

Let's obtain the `AdmissionReview` that is generated when attempting to delete
an existing Deployment resource.

To do that, let's create a regular Deployment object making sure the
"io.kubewarden.policy.echo.delete" annotation is present:

```yaml
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx
  namespace: default
  annotations:
    io.kubewarden.policy.echo.delete: "true"
spec:
  selector:
    matchLabels:
      app: nginx
  replicas: 0
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:latest
        ports:
        - containerPort: 80
EOF
```

Let's trigger a DELETE operation:

```console
kubectl delete deployment -n default nginx
```

The `echo` policy will reject this change. We can get the raw AdmissionReview
object by using the following command:

```console
kubectl delete deployment -n default nginx 2>&1 | sed -e 's/Error from.*request: //' | jq
```
