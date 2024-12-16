# Kustomization Decryption Provider

Spec.decryption.provider must be set to one of decryption_providers.

Set the Kustomization's Spec.decryption.provider to a valid value from the decryption_providers.

# Settings
```yaml
  settings:
    parameters:
      - name: decryption_providers
        type: array
        required: true
        value:
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
