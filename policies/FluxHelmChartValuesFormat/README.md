# HelmChart Values File Format

HelmChart must reference values files in the following format: 'xxx=values.yaml'.

Update the HelmChart valuesFrom field to use the correct format.

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
`HelmChart`
