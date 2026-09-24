# OCIRepository Ignore Suffixes

OCIRepository resources must include specific suffixes in the spec.ignore field which determines the files to be ignored before making a commit..

Ensure the spec.ignore field in the OCIRepository resource includes the required suffixes. The specified suffixes correspond to the file types that should not be included in the repository commit.

# Settings

```yaml
settings:
  suffixes: []
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`OCIRepository`
