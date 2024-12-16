# Helm Release Service Account Name

HelmRelease serviceAccountName must contain a value from parameters.service_account_names

Set the serviceAccountName of the HelmRelease to one of the allowed service accounts.

# Settings
```yaml
  settings:
    parameters:
      - name: service_account_names
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
`HelmRelease`
