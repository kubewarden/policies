# Kustomization Images Requirement

The 'spec.images' field in a Kustomization object must be enabled or disabled based on the policy input images_required.

Update the 'spec.images' field in the Kustomization object based on the policy input images_required.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional, default: ""
  exclude_label_value: "" # optional, default: ""
  images_required: true # default: true
```

# Resources

Policy applies to resources kinds:
`Kustomization`
