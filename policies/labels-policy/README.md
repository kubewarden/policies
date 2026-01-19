[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# labels-policy

This policy validates the labels of generic Kubernetes objects.

> [!NOTE]  
> By default, the policy metadata targets only workload resources (Deployments,
> Pods, Replicasets, Jobs..).
>
> Deploy the policy with your desired rules to target other resources.
> Note that the audit scanner feature does [not support policies targeting `"*"`](https://docs.kubewarden.io/explanations/audit-scanner/limitations#usage-of--by-policies).

## Settings

The policy settings has the `criteria` field which define the logic operation
performed with the `values` defined in the settings and the labels
defined in the resource:

```yaml
settings:
  criteria: "containsAnyOf"
  values:
    - cost-center
```

The `criteria` configuration can have the following values:

- `containsAnyOf`: enforces that the resource has at least one of the
  labels in `values`.
- `doesNotContainAnyOf`: enforces that the resource does not have any label
  defined in `values` (denylist).
- `containsAllOf`: enforces that all of the labels in `values` are defined in
  the resource.
- `doesNotContainAllOf`: enforces that the labels defined in `values` are
  not all set together in the resource.
- `ContainsOtherThan`: enforces that the resource contains at least one label not in `values`.
- `DoesNotContainOtherThan`: enforces that the resource contains only
  labels from `values` (allowlist).

The `values` field must contain at least one label name for
validation. label names should be [valid label
names](https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#syntax-and-character-set)
per Kubernetes docs.

> [!IMPORTANT]  
> An empty list of label names is not allowed.

If you require more complex labels validation, consider the use
of [Kubewarden policy groups](https://docs.kubewarden.io/howtos/policy-groups).
With policy groups, you can combine multiple validations using complex logical
operators to function as a single policy.

## Rules operators logic tables

These are some tables to help you understand the logic of the operators:

### `containsAnyOf`

Given these `values` settings: `[a, b]`

| Resource labels | Evaluation result |
| --------------- | ----------------- |
| a               | Accepted          |
| b               | Accepted          |
| a,b             | Accepted          |
| a,b,c           | Accepted          |
| c               | Rejected          |
| a, c            | Accepted          |
| b, c            | Accepted          |
| empty           | Rejected          |

### `doesNotContainAnyOf` (denylist)

Given these `values` settings: `[a, b]`

| Resource labels | Evaluation result |
| --------------- | ----------------- |
| a               | Rejected          |
| b               | Rejected          |
| a,b             | Rejected          |
| a,b,c           | Rejected          |
| c               | Accepted          |
| a, c            | Rejected          |
| b, c            | Rejected          |
| empty           | Accepted          |

### `containsAllOf`

Given these `values` settings: `[a, b]`

| Resource labels | Evaluation result |
| --------------- | ----------------- |
| a               | Rejected          |
| b               | Rejected          |
| a,b             | Accepted          |
| a,b,c           | Accepted          |
| c               | Rejected          |
| a, c            | Rejected          |
| b, c            | Rejected          |
| empty           | Rejected          |

### `doesNotContainAllOf`

Given these `values` settings: `[a, b]`

| Resource labels | Evaluation result |
| --------------- | ----------------- |
| a               | Accepted          |
| b               | Accepted          |
| a,b             | Rejected          |
| a,b,c           | Rejected          |
| c               | Accepted          |
| a, c            | Accepted          |
| b, c            | Accepted          |
| empty           | Accepted          |

### `containsOtherThan`

Given these `values` settings: `[a, b]`

| Resource labels | Evaluation result |
| --------------- | ----------------- |
| a               | rejected          |
| b               | rejected          |
| a,b             | rejected          |
| a,b,c           | accepted          |
| c               | accepted          |
| a, c            | accepted          |
| b, c            | accepted          |
| empty           | rejected          |

### `doesNotContainOtherThan` (allowlist)

Given these `values` settings: `[a, b]`

| Resource labels | Evaluation result |
| --------------- | ----------------- |
| a               | accepted          |
| b               | accepted          |
| a,b             | accepted          |
| a,b,c           | rejected          |
| c               | rejected          |
| a, c            | rejected          |
| b, c            | rejected          |
| empty           | accepted          |
