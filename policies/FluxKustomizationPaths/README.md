# Kustomization Excluded Paths

spec.path cannot be one of excludedPathsList[]

Update the spec.path to a value not in the excluded paths list.

# Settings

```yaml
settings:
  exclude_paths: []
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Kustomization`
