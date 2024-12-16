# Helm Release Max History

HelmRelease maxHistory cannot exceed the specified value.

Set the maxHistory of the HelmRelease to a value less than or equal to the specified limit.

# Settings
```yaml
  settings:
    parameters:
      - name: max_history
        type: integer
        required: true
        value: 100
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
`HelmRelease`
