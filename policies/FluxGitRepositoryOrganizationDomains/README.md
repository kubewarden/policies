# GitRepository Organization Domains

GitRepository resources must only be from allowed organization domains.

Ensure the GitRepository URL belongs to an allowed organization domain.

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
`GitRepository`
