# Kustomization Image Tag Standards

spec.Images must comply with image tag/semver reference standards.

Update the image tag to comply with semver reference standards.

# Settings
```yaml
  settings:
    parameters:
      - name: exclude_namespaces
        type: array
        required: false
        value:
      - name: exclude_label_key
        type: string
        required: false
        value:
      - name: exclude_label_value
        type: string
        required: false
        value:
```

# Resources
Policy applies to resources kinds:
`Kustomization`
