# Helm Release Storage Namespace

HelmRelease storageNamespace must contain a value from storage_namespaces.

Set the storageNamespace of the HelmRelease to one of the allowed namespaces.

# Settings
```yaml
  settings:
    parameters:
      - name: storage_namespaces
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
