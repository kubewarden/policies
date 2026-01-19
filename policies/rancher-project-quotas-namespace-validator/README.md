[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

> **WARNING:** this policy requires Kubewarden 1.6.0 or later

This policy is meant to be used together with [Rancher Manager](https://ranchermanager.docs.rancher.com/).

Rancher Manager introduces the concept of `Project`. Projects group different
Kubernetes Namespace and can enforce resource quotas across all of them.
To learn more about Rancher Projects, checkout the [official documentation](https://ranchermanager.docs.rancher.com/v2.6/how-to-guides/new-user-guides/manage-clusters/projects-and-namespaces).

Rancher Manager UI prevents the creation of Namespace under a given Project once
its `ResourceQuota` is exceeded.

This policy complements Rancher Manager by introducing the same set of checks
for all the requests issued against the Kubernetes API server (like via `kubectl`).

## Settings

This policy does not have any configuration value.

## Example

Create a project under the Rancher Manager UI:

- Cluster
- Project/Namespaces
- Create Project
- Resource Quota Tab
- Select "CPU Reservation" from the dropdown
- Set Project Limit as `500` and Namespace as Limit as `100`
- Create

Get the cluster id(e.g., local) combined with Project ID(e.g., `p-sd7dh`) and enter in below yaml to create namespace with `requestsCpu` as 400m under the project.

Create a new Namespace using a definition like the following one:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: one
  annotations:
    field.cattle.io/projectId: local:p-sd7dh
    field.cattle.io/resourceQuota: '{"limit":{"requestsCpu":"400m"}}'
  labels:
    field.cattle.io/projectId: p-sd7dh
```

Create another Namespace which allocates all the remaining quota of `requestsCpu`:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: two
  annotations:
    field.cattle.io/projectId: local:p-sd7dh
    field.cattle.io/resourceQuota: '{"limit":{"requestsCpu":"100m"}}'
  labels:
    field.cattle.io/projectId: p-sd7dh
```

Now, all the quota of `requestsCpu` is exhausted inside of the Project.

This policy will prevent the creation of other Namespace under the project:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: three
  annotations:
    field.cattle.io/projectId: local:p-sd7dh
    field.cattle.io/resourceQuota: '{"limit":{"requestsCpu":"100m"}}'
  labels:
    field.cattle.io/projectId: p-sd7dh 
```

This time the project creation will be rejected.
