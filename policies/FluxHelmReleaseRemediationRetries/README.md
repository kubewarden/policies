# Helm Release Remediation Retries

HelmRelease remediation retries must be within the specified lower and upper bounds.

Set the remediation retries value of the HelmRelease to a value within the allowed range.

# Settings
```yaml
  settings:
    parameters:
      - name: lower_bound
        type: integer
        required: true
        value: 1
      - name: upper_bound
        type: integer
        required: true
        value: 10
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
