# Helm Release Post Renderer

HelmRelease must not have post-renderers enabled.

Remove the post-renderers configuration from the HelmRelease.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRelease`
