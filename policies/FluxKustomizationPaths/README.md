# Kustomization Excluded Paths

spec.path cannot be one of excludedPathsList[]

Update the spec.path to a value not in the excluded paths list.

# Settings
```yaml
  settings:
    parameters:
      - name: exclude_paths
        type: array
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
