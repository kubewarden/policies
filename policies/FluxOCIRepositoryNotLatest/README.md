# OCIRepository Not Latest Tag

OCIRepository resources must not use 'latest' as a tag reference.

Specify a versioned tag for the OCIRepository instead of using 'latest'.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`OCIRepository`
