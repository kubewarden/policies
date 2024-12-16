# HelmChart Cosign Verification

HelmChart objects must provide cosign verification and reference a secret containing the Cosign public keys of trusted authors

Add cosign verification and reference a secret containing the Cosign public keys of trusted authors to the HelmChart object.

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
`HelmChart`
