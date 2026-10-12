# Kustomization Var Substitution

The property 'spec.postBuild.substitute.var_substitution_enabled' must be disabled.

Set the Kustomization's spec.postBuild.substitute.var_substitution_enabled to false.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Kustomization`
