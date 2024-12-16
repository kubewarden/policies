# Helm Release Post Renderer

HelmRelease must not have post-renderers enabled.

Remove the post-renderers configuration from the HelmRelease.

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
`HelmRelease`
