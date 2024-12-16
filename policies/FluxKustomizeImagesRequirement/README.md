# Kustomization Images Requirement

The 'spec.images' field in a Kustomization object must be enabled or disabled based on the policy input images_required.

Update the 'spec.images' field in the Kustomization object based on the policy input images_required.

# Settings
```yaml
  settings:
    parameters:
      - name: exclude_namespaces
        type: array
        required: false
        value: []
      - name: exclude_label_key
        type: string
        required: false
        value: ""
      - name: exclude_label_value
        type: string
        required: false
        value: ""
      - name: images_required
        type: boolean
        required: true
        value: true
```

# Resources
Policy applies to resources kinds:
`Kustomization`
