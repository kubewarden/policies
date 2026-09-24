# Kustomization Decryption Provider

Spec.decryption.provider must be set to one of decryption_providers.

Set the Kustomization's Spec.decryption.provider to a valid value from the decryption_providers.

# Settings

```yaml
settings:
  decryption_providers: []
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Kustomization`
