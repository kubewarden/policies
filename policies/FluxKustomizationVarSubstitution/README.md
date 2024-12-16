# Kustomization Var Substitution

The property 'spec.postBuild.substitute.var_substitution_enabled' must be disabled.

Set the Kustomization's spec.postBuild.substitute.var_substitution_enabled to false.

# Settings
```yaml
  settings:
    parameters:
      - name: exclude_namespaces
        type: array
        required: false
        value:
      - name: exclude_label_key
        type: string
        required: false
        value:
      - name: exclude_label_value
        type: string
        required: false
        value:
```

# Resources
Policy applies to resources kinds:
`Kustomization`
