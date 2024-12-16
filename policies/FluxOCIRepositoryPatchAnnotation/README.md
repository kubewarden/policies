# OCIRepository Patch Annotation

OCIRepository resources must have a patch annotation that matches the provider.

Ensure the OCIRepository has a patch annotation matching the provider.

# Settings
```yaml
  settings:
    parameters:
      - name: provider
        type: string
        required: true
        value:
      - name: exclude_namespaces
        type: array
        required: false
        value: []
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
`OCIRepository`
