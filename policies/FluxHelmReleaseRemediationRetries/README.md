# Helm Release Remediation Retries

HelmRelease remediation retries must be within the specified lower and upper bounds.

Set the remediation retries value of the HelmRelease to a value within the allowed range.

# Settings

```yaml
settings:
  lower_bound: 1 # default: 1
  upper_bound: 10 # default: 10
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRelease`
