# Helm Release Rollback Should Be Disabled

The rollback feature of a HelmRelease should be disabled.

Set the rollback feature of the HelmRelease to false.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRelease`
