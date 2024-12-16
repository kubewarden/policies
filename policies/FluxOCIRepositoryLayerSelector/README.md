# OCIRepository Layer Selector

OCIRepository layer selector must only reference predefined media/operation type.

Ensure the OCIRepository layer selector refers to a predefined media and operation type.

# Settings
```yaml
  settings:
    parameters:
      - name: media_types
        type: array
        required: true
        value:
      - name: operations
        type: array
        required: true
        value: ["extract"]
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
