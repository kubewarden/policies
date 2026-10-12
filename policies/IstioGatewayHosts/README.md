# Istio Gateway Approved Hosts

### Istio Gateway Approved Hosts

This ensures you are only serving traffic for approved hostnames.

Ensure your domain name is the same as the one you are hosting.

# Settings

```yaml
settings:
  hostnames: []
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Gateway`
