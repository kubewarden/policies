# Kustomization Prune

The 'spec.prune' field in the Kustomization object must be set according to the input parameter 'prune'.

Update the 'spec.prune' field in the Kustomization object to match the required value.

# Settings
```yaml
  settings:
    parameters:
      - name: prune
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
