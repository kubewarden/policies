[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# environment-variable-policy

The environment-variable-policy can be used to inspect environment variables
defined in the resources deployed in the cluster.

The policy can either target Pods, or workload resources (Deployments,
ReplicaSets, DaemonSets, ReplicationControllers, Jobs, CronJobs). Both have
trade-offs:

- Policy targets Pods: Different kind of resources (be them native or CRDs) can
  create Pods. By having the policy target Pods, we guarantee that all the Pods
  are going to be compliant, even those created from CRDs. However, this could
  lead to confusion among users, as high level Kubernetes resources would be
  successfully created, but they would stay in a non reconciled state. Example: a
  Deployment creating a non-compliant Pod would be created, but it would never
  have all its replicas running.

- Policy targets workload resources (e.g: Deployment): the policy inspect
  higher order resource (e.g. Deployment): users will get immediate feedback
  about rejections. However, non compliant pods created by another high level
  resource (be it native to Kubernetes, or a CRD), may not get rejected.

## Settings

> [!WARNING]
> If you are upgrading from version v1.x.x, please note the breaking changes
> introduced in v2.x.x:
>
> Environment variable values are no longer considered.
>
> The policy now focuses solely on the name of the environment variable, not
> its value.
>
> New settings syntax
>
> The settings no longer use a list of rules for validation. Instead, the
> policy validates a single type of condition. Refer to the "Current Settings
> Fields" section below for details on the new settings.

The policy settings has the `criteria` field which define the logic operatation
performed with the `values` defined in the settings and the environment variables
defined in the resource:

```yaml
settings:
  criteria: "containsAnyOf"
  values:
    - MARIADB_USER
    - MARIADB_PASSWORD
```

The `criteria` configuration can have the following values:

- `containsAnyOf`: enforces that the resource has at least one of the
  environment variables in `values`.
- `doesNotContainAnyOf`: enforces that the resource does not have any
  environment variable defined in `values` (denylist).
- `containsAllOf`: enforces that all of the environment variables in `values`
  are defined in the resource.
- `doesNotContainAllOf`: enforces that the environment variables defined in
  `values` are not all set together in the resource.
- `ContainsOtherThan`: enforces that the resource contains at least one
  environment varaible not in `values`.
- `DoesNotContainOtherThan`: enforces that the resource contains only
  environment variables from `values` (allowlist).

The `values` field must contain at least one environment variable name for
validation. Environment variable names should follow the C_IDENTIFIER standard.

> [!IMPORTANT]
> An empty list of environment variable names is not allowed.

If you require more complex environment variable validation, consider the use
of [Kubewarden policy groups](https://docs.kubewarden.io/howtos/policy-groups).
With policy groups, you can combine multiple validations using complex logical
operators to function as a single policy.

## Rules operators logic tables

These are some tables to help you understand the logic of the operators:

### `containsAnyOf`

Given these `environmentVariables` settings: `[a, b]`

| Resource environment variables | Evaluation result |
| ------------------------------ | ----------------- |
| a                              | Accepted          |
| b                              | Accepted          |
| a,b                            | Accepted          |
| a,b,c                          | Accepted          |
| c                              | Rejected          |
| a, c                           | Accepted          |
| b, c                           | Accepted          |
| empty                          | Rejected          |

### `doesNotContainAnyOf` (denylist)

Given these `environmentVariables` settings: `[a, b]`

| Resource environment variables | Evaluation result |
| ------------------------------ | ----------------- |
| a                              | Rejected          |
| b                              | Rejected          |
| a,b                            | Rejected          |
| a,b,c                          | Rejected          |
| c                              | Accepted          |
| a, c                           | Rejected          |
| b, c                           | Rejected          |
| empty                          | Accepted          |

### `containsAllOf`

Given these `environmentVariables` settings: `[a, b]`

| Resource environment variables | Evaluation result |
| ------------------------------ | ----------------- |
| a                              | Rejected          |
| b                              | Rejected          |
| a,b                            | Accepted          |
| a,b,c                          | Accepted          |
| c                              | Rejected          |
| a, c                           | Rejected          |
| b, c                           | Rejected          |
| empty                          | Rejected          |

### `doesNotContainAllOf`

Given these `environmentVariables` settings: `[a, b]`

| Resource environment variables | Evaluation result |
| ------------------------------ | ----------------- |
| a                              | Accepted          |
| b                              | Accepted          |
| a,b                            | Rejected          |
| a,b,c                          | Rejected          |
| c                              | Accepted          |
| a, c                           | Accepted          |
| b, c                           | Accepted          |
| empty                          | Accepted          |

### `containsOtherThan`

Given these `environmentVariables` settings: `[a, b]`

| Resource environment variables | Evaluation result |
| ------------------------------ | ----------------- |
| a                              | rejected          |
| b                              | rejected          |
| a,b                            | rejected          |
| a,b,c                          | accepted          |
| c                              | accepted          |
| a, c                           | accepted          |
| b, c                           | accepted          |
| empty                          | rejected          |

### `doesNotContainOtherThan` (allowlist)

Given these `environmentVariables` settings: `[a, b]`

| Resource environment variables | Evaluation result |
| ------------------------------ | ----------------- |
| a                              | accepted          |
| b                              | accepted          |
| a,b                            | accepted          |
| a,b,c                          | rejected          |
| c                              | rejected          |
| a, c                           | rejected          |
| b, c                           | rejected          |
| empty                          | accepted          |
