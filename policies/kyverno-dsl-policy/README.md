[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Sandbox](https://img.shields.io/badge/status-sandbox-red?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#sandbox)

This policy aims to provide a way to reuse [Kyverno policies](https://kyverno.io/docs/writing-policies/) with Kubewarden.

## Policy goals

The policy aims to allow the creation of validating and mutating policies.
It does not intend to provide a way to use Kyverno's generating policies.

## Known limitations

### Go compiler

Kyverno evaluation code is too complex for TinyGo. Because of that, the official Go compiler
must be used to compile this policy.
Building the policy policy requires Go 1.21 or later, which introduces support for the [WASI](https://wasi.dev/) target.

### Execution mode

Currently the WASI modules produce by the Go compiler cannot export functions.
This is a [known issue](https://github.com/golang/go/issues/42372) that is going
to be addressed by future releases of Go. In the meantime, this policy leverages the new
[WASI excution mode](https://docs.kubewarden.io/writing-policies/wasi) offered starting from
the Kubewarden 1.7.0 release. Once the linked Go issue is fixed, the policy can be rewritten
to make use of the traditional [waPC execution mode](https://docs.kubewarden.io/writing-policies/spec/intro-spec).

### Size and startup time

The amount of code, and dependencies, required to evaluate Kyverno policies is significant.
The size of this policy is huge, it's approximatively 62 Mb. For comparison, a traditional
Kubewarden policy size ranges from 100 Kb up to 2 Mb.

This not only increases the download time, but it takes a long time to be optimized by the
JIT compiler of Wasmtime. This means the first run of the policy is going to be slow, while
the subsequent ones are going to be fast.

When used inside of a Kubernetes cluster, it's highly recommended to deploy this policy to a
dedicated Policy Server. In this way, specific Policy Servers are going to be impacted by the
slow startup times and the higher memory usage caused by this policy.

To do that, start by deploy a dedicated Policy Server:

```yaml
apiVersion: policies.kubewarden.io/v1
kind: PolicyServer
metadata:
  name: kyverno-only
spec:
  image: ghcr.io/kubewarden/policy-server:v1.7.0
  replicas: 2
  serviceAccountName: ~
```

Then deploy the policy to this specific Policy Server by using the
`spec.policyServer` attribute. This is going to be shown in the example below.

### Limitations due to the maturity of this codebase

- Policy settings are not supported
- Mutation support is not yet done
- Context aware information are not provided to the policy

## Usage

The policy provides the Kyverno evaluation engine, the actual policy is specified
via the policy settings.

This is done using the `policy` key, that has as value a string holding the actual
YAML definition of the Kyverno policy.

For example, to deploy [this](https://kyverno.io/policies/flux/verify-flux-sources/verify-flux-sources/) Kyverno policy,
the settings of the policy should look like that:

```yaml
policy: |
  apiVersion: kyverno.io/v1
  kind: ClusterPolicy
  metadata:
    name: allowed-annotations
    annotations:
      policies.kyverno.io/title: Allowed Annotations
      policies.kyverno.io/category: Other
      policies.kyverno.io/severity: medium
      kyverno.io/kyverno-version: 1.6.0
      policies.kyverno.io/minversion: 1.6.0
      kyverno.io/kubernetes-version: "1.23"
      policies.kyverno.io/subject: Pod, Annotation
      policies.kyverno.io/description: >-
        Rather than creating a deny list of annotations, it may be more useful
        to invert that list and create an allow list which then denies any others.
        This policy demonstrates how to allow two annotations with a specific key
        name of fluxcd.io/ while denying others that do not meet the pattern.      
  spec:
    validationFailureAction: audit
    background: true
    rules:
    - name: allowed-fluxcd-annotations
      match:
        any:
        - resources:
            kinds:
              - Pod
      validate:
        message: The only approved FluxCD annotations are `fluxcd.io/cow` and `fluxcd.io/dog`.
        deny:
          conditions:
            all:
            - key: "{{ request.object.metadata.annotations.keys(@)[?contains(@, 'fluxcd.io/')] }}"
              operator: AnyNotIn
              value:
              - fluxcd.io/cow
              - fluxcd.io/dog
```

> **Note:** this is just a copy & paste of the official Kyverno policy

This means the actual Kubewarden policy deployment would look like that:

```yaml
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  name: allowed-flux-annotations
spec:
  policyServer: kyverno-only
  module: registry://ghcr.io/kubewarden/policies/kyverno-dsl:v0.1.0
  rules:
    - apiGroups: [""]
      apiVersions: ["v1"]
      resources: ["pods"]
      operations:
        - CREATE
        - UPDATE
  mutating: false
  settings:
    policy: |
      apiVersion: kyverno.io/v1
      kind: ClusterPolicy
      metadata:
        name: allowed-annotations
        annotations:
          policies.kyverno.io/title: Allowed Annotations
          policies.kyverno.io/category: Other
          policies.kyverno.io/severity: medium
          kyverno.io/kyverno-version: 1.6.0
          policies.kyverno.io/minversion: 1.6.0
          kyverno.io/kubernetes-version: "1.23"
          policies.kyverno.io/subject: Pod, Annotation
          policies.kyverno.io/description: >-
            Rather than creating a deny list of annotations, it may be more useful
            to invert that list and create an allow list which then denies any others.
            This policy demonstrates how to allow two annotations with a specific key
            name of fluxcd.io/ while denying others that do not meet the pattern.      
      spec:
        validationFailureAction: audit
        background: true
        rules:
        - name: allowed-fluxcd-annotations
          match:
            any:
            - resources:
                kinds:
                  - Pod
          validate:
            message: The only approved FluxCD annotations are `fluxcd.io/cow` and `fluxcd.io/dog`.
            deny:
              conditions:
                all:
                - key: "{{ request.object.metadata.annotations.keys(@)[?contains(@, 'fluxcd.io/')] }}"
                  operator: AnyNotIn
                  value:
                  - fluxcd.io/cow
                  - fluxcd.io/dog
```

> **Note:** this is policy is going to be deployed on a Policy Server named `kyverno-only`
