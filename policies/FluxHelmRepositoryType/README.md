# Helm Repo Type Should Be OCI

The type of a Helm repository should be OCI.

Change the type of the Helm repository to OCI.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional, default: ["kube-system"]
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRepository`
