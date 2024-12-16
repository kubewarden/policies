# Bucket Endpoint Domain

Bucket endpoint domain must be one of the trusted domains.

Update the Bucket endpoint domain to one of the trusted domains.

# Settings
```yaml
  settings:
    parameters:
      - name: domains
        type: array
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
`Bucket`
