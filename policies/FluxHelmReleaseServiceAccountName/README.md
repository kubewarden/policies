# Helm Release Service Account Name

HelmRelease serviceAccountName must contain a value from parameters.service_account_names

Set the serviceAccountName of the HelmRelease to one of the allowed service accounts.

# Settings

```yaml
settings:
  service_account_names: []
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRelease`
