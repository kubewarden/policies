[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Kubewarden policy volumemounts-policy

## Description

Policy that inspects containers, init containers, and ephemeral containers, and
restricts their usage of volumes by  checking the `volume` name being used in
`volumeMounts[*].name`.

The policy can either target `Pods`, or [workload
resources](https://kubernetes.io/docs/concepts/workloads/) (`Deployments`,
`ReplicaSets`, `DaemonSets`, `ReplicationControllers`, `Jobs`, `CronJobs`) by
setting the policy's `spec.rules` accordingly.

Both have trade-offs:
* Policy targets Pods: Different kind of resources (be them native or CRDs) can
  create Pods. By having the policy target Pods, we guarantee that all the Pods
  are going to be compliant, even those created from CRDs.
  However, this could lead to confusion among users, as high level Kubernetes
  resources would be successfully created, but they would stay in a non
  reconciled state. Example: a Deployment creating a non-compliant Pod would be
  created, but it would never have all its replicas running.
* Policy targets workload resources (e.g: Deployment): the policy inspect higher
  order resource (e.g. Deployment): users will get immediate feedback about
  rejections.
  However, non compliant pods created by another high level resource (be it
  native to Kubernetes, or a CRD), may not get rejected.


## Settings
```yaml
reject: anyIn # one of anyIn (default, denylist), anyNotIn (allowlist), allAreUsed, notAllAreUsed
volumeMountsNames:  # list of volumeMounts.name to match using the defined reject operator
  - foo
  - bar
  - baz
```

- `anyIn` (default): checks if any of the volumeMountsNames are in the Pod/Workload resource
- `anyNotIn`: checks if any of the volumeMountsNames are not in the Pod/Workload resource
- `allAreUsed`: checks if all of the volumeMountsNames are in the Pod/Workload resource
- `notAllAreUsed`: checks if all of the volumeMountsNames are not in the Pod/Workload resource

## Examples

```yaml
# denylist, reject volumeMounts named `my-volume` or `my-volume2`
reject: anyIn
volumeMountsNames:
  - my-volume
  - my-volume2
```

```yaml
# allowlist, only allow volumeMounts named `my-volume3` or `my-volume4`
reject: anyNotIn
volumeMountsNames:
  - my-volume3
  - my-volume4
```

```yaml
# container cannot use both volumes at once, only one or the other
reject: allAreUsed
volumeMountsNames:
  - my-volume5
  - my-volume6
```

```yaml
# container can use both volumes at once, but not only one of them
reject: notAllAreUsed
volumeMountsNames:
  - my-volume5
  - my-volume6
```
