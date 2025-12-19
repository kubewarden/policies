This is a helper crate that allows to easily scaffold a policy that validates a
list of strings using a well defined list of criteria.

The allowed criteria are:

- `containsAnyOf`
- `doesNotContainAllOf`
- `containsAllOf`
- `doesNotContainAllOf`
- `containsOtherThan`
- `doesNotContainOtherThan`

Right now we expect this library to be consumed by policies that are validating
environment variables, labels and annotations.

## Rules operators logic tables

These are some tables to help you understand the logic of the operators:

### `containsAnyOf`

Given these `values` settings: `[a, b]`

| Resource values entries | Evaluation result |
| ----------------------- | ----------------- |
| a                       | Accepted          |
| b                       | Accepted          |
| a,b                     | Accepted          |
| a,b,c                   | Accepted          |
| c                       | Rejected          |
| a, c                    | Accepted          |
| b, c                    | Accepted          |
| empty                   | Rejected          |

### `doesNotContainAnyOf` (denylist)

Given these `values` settings: `[a, b]`

| Resource values entries | Evaluation result |
| ----------------------- | ----------------- |
| a                       | Rejected          |
| b                       | Rejected          |
| a,b                     | Rejected          |
| a,b,c                   | Rejected          |
| c                       | Accepted          |
| a, c                    | Rejected          |
| b, c                    | Rejected          |
| empty                   | Accepted          |

### `containsAllOf`

Given these `values` settings: `[a, b]`

| Resource values entries | Evaluation result |
| ----------------------- | ----------------- |
| a                       | Rejected          |
| b                       | Rejected          |
| a,b                     | Accepted          |
| a,b,c                   | Accepted          |
| c                       | Rejected          |
| a, c                    | Rejected          |
| b, c                    | Rejected          |
| empty                   | Rejected          |

### `doesNotContainAllOf`

Given these `values` settings: `[a, b]`

| Resource values entries | Evaluation result |
| ----------------------- | ----------------- |
| a                       | Accepted          |
| b                       | Accepted          |
| a,b                     | Rejected          |
| a,b,c                   | Rejected          |
| c                       | Accepted          |
| a, c                    | Accepted          |
| b, c                    | Accepted          |
| empty                   | Accepted          |

### `containsOtherThan`

Given these `values` settings: `[a, b]`

| Resource values entries | Evaluation result |
| ----------------------- | ----------------- |
| a                       | rejected          |
| b                       | rejected          |
| a,b                     | rejected          |
| a,b,c                   | accepted          |
| c                       | accepted          |
| a, c                    | accepted          |
| b, c                    | accepted          |
| empty                   | rejected          |

### `doesNotContainOtherThan` (allowlist)

Given these `values` settings: `[a, b]`

| Resource values entries | Evaluation result |
| ----------------------- | ----------------- |
| a                       | accepted          |
| b                       | accepted          |
| a,b                     | accepted          |
| a,b,c                   | rejected          |
| c                       | rejected          |
| a, c                    | rejected          |
| b, c                    | rejected          |
| empty                   | accepted          |

## How to use the library

The errors messages contain the name of the resource the final policy
evaluates.

The name of the resource being evaluated is set inside of `lib.rs` and
is set by looking at the presence of specific feature flags.

The settings of the real policy is driven by the structure of the `settings::BaseSettings`
enumeration:

```yaml
settings:
  criteria: "containsAnyOf"
  values:
    - MARIADB_USER
    - MARIADB_PASSWORD
```

More work would be needed if the real policy needs other setting values.

The actual validation is then done by the `validate::validate_values` function
that must be called by the real policy.
