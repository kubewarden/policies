# Kustomization Target Namespace

Kustomization targetNamespace must be one of the allowed targetNamespace list.

Set the targetNamespace of the Kustomization to one of the allowed namespaces.

# Settings

```yaml
settings:
  target_namespaces: []
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional, default: ""
  exclude_label_value: "" # optional, default: ""
```

# Resources

Policy applies to resources kinds:
`Kustomization`
