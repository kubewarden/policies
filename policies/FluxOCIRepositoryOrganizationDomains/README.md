# OCIRepository Organization Domains

OCIRepository resources must only be from allowed organization domains.

Ensure the OCIRepository URL belongs to an allowed organization domain.

# Settings

```yaml
settings:
  domains: []
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`OCIRepository`
