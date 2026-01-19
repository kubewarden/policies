[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

This Kubewarden Policy is a replacement for the Kubernetes Pod Security Policy
that enforces the usage of [`ReadOnlyRootFilesystems`](https://kubernetes.io/docs/concepts/policy/pod-security-policy/#volumes-and-file-systems).

# How the policy works

The policy inspects the `securityContext` of each container defined inside of
a Pod and ensures all the containers have the `readOnlyRootFilesystem` attribute
set to `true`.

The policy checks the both the `pod.spec.containers` and the init containers
too.

Containers that do not have a `securityContext` defined are rejected too.
That happens because, by default, the root filesystem of a container is
considered to be writable.

Ephemeral containers are not checked because, by Kubernetes definition, they
cannot have a `securityContext`.

# Configuration

The policy doesn't have any configuration.
