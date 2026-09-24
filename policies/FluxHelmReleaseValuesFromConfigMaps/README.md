# Helm Release Values From

HelmRelease valuesFrom must use correctly configured ConfigMaps.

Ensure that the HelmRelease uses allowed ConfigMaps in the valuesFrom field.

# Settings

```yaml
settings:
  configmaps: []
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRelease`
