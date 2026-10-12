# Kustomization Patches

Kustomization patches should be enabled or disabled based on input.

Update the 'spec.patches' field in the Kustomization object according to the input.

# Settings

```yaml
settings:
  patches_required: false
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Kustomization`
