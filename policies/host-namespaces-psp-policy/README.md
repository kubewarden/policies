[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Kubewarden policy psp-host-namespaces

## Description

Replacement for the Kubernetes Pod Security Policy that controls the
usage of host namespaces

The policy can either target `Pods`, or [workload
resources](https://kubernetes.io/docs/concepts/workloads/) (`Deployments`,
`ReplicaSets`, `DaemonSets`, `ReplicationControllers`, `Jobs`, `CronJobs`) by
setting the policy's `spec.rules` accordingly.

Both have trade-offs:

- Policy targets Pods: Different kind of resources (be them native or CRDs) can
  create Pods. By having the policy target Pods, we guarantee that all the Pods
  are going to be compliant, even those created from CRDs.
  However, this could lead to confusion among users, as high level Kubernetes
  resources would be successfully created, but they would stay in a non
  reconciled state. Example: a Deployment creating a non-compliant Pod would be
  created, but it would never have all its replicas running.
- Policy targets workload resources (e.g: Deployment): the policy inspect higher
  order resource (e.g. Deployment): users will get immediate feedback about
  rejections.
  However, non compliant pods created by another high level resource (be it
  native to Kubernetes, or a CRD), may not get rejected.

## Settings

This policy works by defining what host namespaces can be used by a Pod.

The following setting keys are accepted for this policy:

- `allow_host_ipc`: allows the pod to set `.spec.HostIPC` to true.
- `allow_host_network`: allows the pod to set `.spec.HostNetwork` to true.
- `allow_host_pid`: allows the pod to set `.spec.HostPID` to true.
- `allow_host_ports`: is a range of ports of the form:

  ```yaml
  allow_host_ports:
    - min: 80
      max: 80
    - min: 443
      max: 443
    - min: 8000
      max: 9000
  ```

  This example would allow host ports `80`, `443` and the range `8000-9000`.

`allow_host_ipc`, `allow_host_network` and `allow_host_pid` are
`false` by default. `allow_host_ports` is an empty list by
default. This means that by default host IPC, network, pid and all
host ports are disabled when this policy is loaded with no
configuration.
