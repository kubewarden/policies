[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Kubewarden policy to control sysctls in pods

## Description

Replacement for the Kubernetes Pod Security Policy that controls the usage of
sysctls.

Linux Kernel sysctls are grouped into safe and unsafe sets. A safe sysctl must
be properly isolated between pods on the same node, and are properly namespaced
by the kernel. A (possibly outdated) list can be seen
[here](https://kubernetes.io/docs/concepts/security/pod-security-standards/#baseline).

All safe sysctls are enabled by default in Kubernetes.
All unsafe sysctls are disabled by default and must be explicitly allowed on a
per-node or per-pod basis.

As the deprecated analogous Kubernetes PSP, this policy validates which sysctls
can get set in pods by specifying lists of sysctls or sysctl patterns to be
allowed or forbidden. One can then modify the `securityContext` of Pods to make
use of the Sysctls as permitted by this policy.

## Settings

The following settings are accepted:

* `forbiddenSysctls`: List of plain sysctl names or sysctl patterns (which end
  with `*`) to be forbidden. You can forbid a combination of safe and unsafe
  sysctls in the list. To forbid setting any sysctls, use `*` on its own.
* `allowedUnsafeSysctls`: List of plain sysctl names that can be used in Pods.
  `*` cannot be used. `allowedUnsafeSysctls` has precedence over
  `forbiddenSysctls`.

A sysctl cannot be both forbidden and allowed at the same time.

### Example

With this policy deployed and configured as:

``` yaml
forbiddenSysctls:
- net.ipv6.conf.lo.*
allowedUnsafeSysctls:
- net.ipv6.conf.lo.max_addresses
```


A pod specifying the following Sysctls would get permitted, as they are
allowedUnsafe or on the default safe set:

``` yaml
apiVersion: v1
kind: Pod
metadata:
  name: sysctl-example
spec:
  securityContext:
    sysctls:
    - name: net.ipv6.conf.lo.max_addresses
      value: "1024"
    - name: kernel.shm_rmid_forced
      value: "0"
...
```


Yet the following pod will get rejected, as `net.ipv6.conf.lo.mtu` is forbidden,
even if `kernel.shm_rmid_forced` is part of the default safe set:

``` yaml
apiVersion: v1
kind: Pod
metadata:
  name: sysctl-example
spec:
  securityContext:
    sysctls:
    - name: kernel.shm_rmid_forced
      value: "0"
    - name: net.ipv6.conf.lo.mtu
      value: "32768"
...
```
