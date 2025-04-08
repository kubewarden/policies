# Helm Repo URL Should Be in Organisation Domain

The URL of a Helm repository should only be from the specified organisation domain.

Change the URL of the Helm repository to one that is from the organisation domain.

# Settings

```yaml
settings:
  domains: []
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRepository`
