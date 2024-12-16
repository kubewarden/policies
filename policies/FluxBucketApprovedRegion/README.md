# Bucket Approved Region

Bucket region must be one of the approved regions.

Set the region of the Bucket to one of the approved regions.

# Settings
```yaml
  settings:
    parameters:
      - name: regions
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
