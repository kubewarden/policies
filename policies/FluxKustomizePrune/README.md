# Kustomization Prune

The 'spec.prune' field in the Kustomization object must be set according to the input parameter 'prune'.

Update the 'spec.prune' field in the Kustomization object to match the required value.

# Settings

```yaml
settings:
  prune: false
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Kustomization`
