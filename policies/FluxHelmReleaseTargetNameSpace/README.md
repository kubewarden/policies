# Helm Release Target Namespace

HelmRelease targetNamespace must be one of the allowed targetNamespace list.

Set the targetNamespace of the HelmRelease to one of the allowed namespaces.

# Settings

```yaml
settings:
  target_namespaces: []
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRelease`
