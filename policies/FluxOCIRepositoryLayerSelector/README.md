# OCIRepository Layer Selector

OCIRepository layer selector must only reference predefined media/operation type.

Ensure the OCIRepository layer selector refers to a predefined media and operation type.

# Settings

```yaml
settings:
  media_types: []
  operations: [] # default: ["extract"]
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`OCIRepository`
