# Kustomization Patches

Kustomization patches should be enabled or disabled based on input.

Update the 'spec.patches' field in the Kustomization object according to the input.

# Settings
```yaml
  settings:
    parameters:
      - name: patches_required
        type: boolean
        required: true
        value:
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
