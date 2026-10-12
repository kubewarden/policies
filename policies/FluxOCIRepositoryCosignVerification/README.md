# OCIRepository Cosign Verification

OCIRepository resources must provide Cosign verification and reference a specific key.

Ensure the OCIRepository verification provider is 'cosign' and references a specific key.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`OCIRepository`
