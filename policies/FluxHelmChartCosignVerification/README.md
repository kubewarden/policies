# HelmChart Cosign Verification

HelmChart objects must provide cosign verification and reference a secret containing the Cosign public keys of trusted authors

Add cosign verification and reference a secret containing the Cosign public keys of trusted authors to the HelmChart object.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmChart`
