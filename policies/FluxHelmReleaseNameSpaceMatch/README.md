# Helm Release Namespace Match

HelmRelease storageNamespace and targetNamespace must match.

Set the storageNamespace and targetNamespace of the HelmRelease to the same value.

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
