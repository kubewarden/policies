# Bucket Insecure Connections

Ensure that Bucket objects do not use insecure connections

Set 'spec.insecure' to 'false' or remove the field from the Bucket object.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Bucket`
