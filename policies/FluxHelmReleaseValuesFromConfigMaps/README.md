# Helm Release Values From

HelmRelease valuesFrom must use correctly configured ConfigMaps.

Ensure that the HelmRelease uses allowed ConfigMaps in the valuesFrom field.

# Settings
```yaml
  settings:
    parameters:
      - name: configmaps
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
