# Helm Release Max History

HelmRelease maxHistory cannot exceed the specified value.

Set the maxHistory of the HelmRelease to a value less than or equal to the specified limit.

# Settings

```yaml
settings:
  max_history: 100 # default: 100
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRelease`
