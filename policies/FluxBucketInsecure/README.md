# Bucket Insecure Connections

Ensure that Bucket objects do not use insecure connections

Set 'spec.insecure' to 'false' or remove the field from the Bucket object.

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
`Bucket`
