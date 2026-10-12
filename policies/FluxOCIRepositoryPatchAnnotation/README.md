# OCIRepository Patch Annotation

OCIRepository resources must have a patch annotation that matches the provider.

Ensure the OCIRepository has a patch annotation matching the provider.

# Settings

```yaml
settings:
  provider: "provider"
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`OCIRepository`
