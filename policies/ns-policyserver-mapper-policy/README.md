# ns-policyserver-mapper

A Kubewarden `ClusterAdmissionPolicy` that mutates `AdmissionPolicy` and
`AdmissionPolicyGroup` resources, enforcing which `PolicyServer` they run on
based on a label placed on the `Namespace` where the policy is deployed.

## How it works

Label each Namespace with `admission.kubewarden.io/policy-server`, where its
value is the PolicyServer name that should handle its namespaced policies.
Example:

```yaml
kind: Namespace
metadata:
  labels:
    admission.kubewarden.io/policy-server: <desired-PolicyServer>
```

Namespaces with the label will get any `AdmissionPolicy`/
`AdmissionPolicyGroup` on them mutated, so those policies are scheduled in the
desired PolicyServer.

Namespaces without the label will have any `AdmissionPolicy` /
`AdmissionPolicyGroup` creation or update rejected until the label is added.
This prevents policies from being silently scheduled on an unintended PolicyServer.

## Settings

This policy has no required settings.

## Example

```yaml
apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  name: ns-policyserver-mapper
spec:
  module: registry://ghcr.io/kubewarden/policies/ns-policyserver-mapper:v0.1.0
  mode: protect
  mutating: true
  rules:
    - apiGroups: ["policies.kubewarden.io"]
      apiVersions: ["v1"]
      resources: ["admissionpolicies", "admissionpolicygroups"]
      operations:
        - CREATE
        - UPDATE
  contextAwareResources:
    - apiVersion: v1
      kind: Namespace
```
