# OCIRepository Cosign Verification

OCIRepository resources must provide Cosign verification and reference a specific key.

Ensure the OCIRepository verification provider is 'cosign' and references a specific key.

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
`OCIRepository`
