# GitRepository Untrusted Domains

GitRepository resources must not use targets from untrusted domains.

Ensure the GitRepository URL does not belong to an untrusted domain.

# Settings

```yaml
settings:
  untrusted_domains: []
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`GitRepository`
