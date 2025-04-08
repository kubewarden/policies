# Helm Release Storage Namespace

HelmRelease storageNamespace must contain a value from storage_namespaces.

Set the storageNamespace of the HelmRelease to one of the allowed namespaces.

# Settings

```yaml
settings:
  storage_namespaces: []
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRelease`
